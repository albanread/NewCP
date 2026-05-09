use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use newcp_llvm::{CodegenOptions, CompiledModule, OwnedJitModule};
use newcp_parser::{parse_source_module, SourceExportKind, SourceModuleSpec};
use newcp_runtime::{
    BootstrapReport, CompilerService, ExportEntry, ExportDirectory, ExportKind,
    HostedModuleArtifact, KernelState, ModuleKind, ResidentCompiler, RustCommandHandlerSpec, CommandInvocation,
};
use newcp_sema::{analyze_module, SemanticDiagnostic, SemanticModule};

#[derive(Debug, Clone)]
pub struct SourceModuleRecord {
    pub path: PathBuf,
    pub spec: SourceModuleSpec,
}

#[derive(Debug, Clone)]
pub struct SourceModuleGraph {
    pub root_path: PathBuf,
    pub modules: Vec<SourceModuleRecord>,
    pub dependency_edges: Vec<(String, String)>,
    pub runtime_imports: Vec<(String, String)>,
}

impl SourceModuleGraph {
    pub fn root_spec(&self) -> &SourceModuleSpec {
        &self
            .modules
            .last()
            .expect("source module graph should always contain the root module")
            .spec
    }

    pub fn initialization_order(&self) -> Vec<String> {
        self.modules.iter().map(|module| module.spec.name.clone()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct SourceLoadResult {
    pub graph: SourceModuleGraph,
    pub load_log: Vec<String>,
    pub materialized_modules: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoaderCommandLoadResult {
    pub load: SourceLoadResult,
    pub command: CommandInvocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoaderCommandInvokeResult {
    pub command: CommandInvocation,
    pub load_log: Vec<String>,
    pub execution_log: Vec<String>,
    pub materialized_modules: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirtyModuleState {
    Clean,
    Modified,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyModuleRecord {
    pub name: String,
    pub path: PathBuf,
    pub state: DirtyModuleState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetirementReason {
    SourceChanged,
    DependencyChanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFileStamp {
    pub size_bytes: u64,
    pub modified_unix_ms: u128,
}

#[derive(Debug, Clone)]
struct CachedSourceGraph {
    graph: SourceModuleGraph,
    file_stamps: Vec<(PathBuf, SourceFileStamp)>,
}

impl CachedSourceGraph {
    fn is_fresh(&self) -> bool {
        self.file_stamps
            .iter()
            .all(|(path, stamp)| source_file_stamp(path).is_ok_and(|current| current == *stamp))
    }
}

#[derive(Debug, Clone)]
pub struct MaterializedModuleRecord {
    pub name: String,
    pub generation: u64,
    pub path: PathBuf,
    pub stamp: SourceFileStamp,
    pub imports: Vec<String>,
    pub has_executable_image: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetiredMaterialization {
    pub name: String,
    pub generation: u64,
    pub path: PathBuf,
    pub reason: RetirementReason,
    pub collect_after_quiescent_epoch: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExecutionScopeId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedGeneration {
    pub module_name: String,
    pub generation: u64,
}

#[derive(Debug, Clone)]
pub struct ExecutionScopeRecord {
    pub id: ExecutionScopeId,
    pub root_module: String,
    pub pinned_generations: Vec<PinnedGeneration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderFailurePhase {
    DiscoverGraph,
    ReadModuleSource,
    ParseModule,
    AnalyzeModule,
    CodegenModule,
    MaterializeModule,
    RegisterRootModule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoaderFailureRecord {
    pub module_name: Option<String>,
    pub phase: LoaderFailurePhase,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderInvalidationState {
    Clean,
    Dirty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderRecoveryState {
    Ready,
    RecoverableFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoaderSessionStatus {
    pub cached_graph_count: usize,
    pub active_modules: Vec<LoaderModuleStatus>,
    pub dirty_modules: Vec<DirtyModuleRecord>,
    pub invalidation_state: LoaderInvalidationState,
    pub recovery_state: LoaderRecoveryState,
    pub retired_generations: Vec<RetiredMaterialization>,
    pub active_execution_scopes: Vec<LoaderExecutionScopeStatus>,
    pub retired_executable_generation_count: usize,
    pub last_failure: Option<LoaderFailureRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoaderModuleStatus {
    pub name: String,
    pub generation: u64,
    pub path: PathBuf,
    pub imports: Vec<String>,
    pub has_executable_image: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoaderExecutionScopeStatus {
    pub id: ExecutionScopeId,
    pub root_module: String,
    pub pinned_generations: Vec<PinnedGeneration>,
}

#[derive(Debug)]
struct ActiveExecutableImage {
    module_name: String,
    generation: u64,
    export_addresses: HashMap<String, usize>,
    /// `llvm_name -> address` for every method body emitted by this module.
    /// Importers read this when patching their cross-module vtable slots.
    method_addresses: HashMap<String, usize>,
    image: OwnedJitModule,
}

#[derive(Debug)]
struct RetiredExecutableImage {
    module_name: String,
    generation: u64,
    collect_after_quiescent_epoch: u64,
    image: OwnedJitModule,
}

struct StagedModuleUpdate {
    artifact: newcp_runtime::CompiledModuleArtifact,
    compiled_module_entry: String,
    materialized_record: MaterializedModuleRecord,
    /// `(image, export_addresses, method_addresses)`.
    executable_image: Option<(
        OwnedJitModule,
        HashMap<String, usize>,
        HashMap<String, usize>,
    )>,
    retirement_reason: RetirementReason,
}

/// Predicate that vetoes dropping a retired executable image. Receives the
/// module name and generation; returns `true` if the loader is allowed to
/// release the JIT memory, `false` to keep the image retained for the next
/// quiescent pass.
///
/// This is the integration point for heap-side liveness checks: the GC's
/// `BlockHeader.tag` and `ModuleDesc.var_base` both point into a JIT image's
/// memory. Dropping an image whose `TypeDesc`s or `var_base` are still
/// reachable from any live block would dangle those pointers. Until the
/// `heap_introspection` module lands, no probe is set by default and image
/// drops follow stack-quiescence alone (see `garbage_collect_retired`).
pub struct RetiredImageDropPredicate(Box<dyn Fn(&str, u64) -> bool + 'static>);

impl RetiredImageDropPredicate {
    pub fn new(predicate: impl Fn(&str, u64) -> bool + 'static) -> Self {
        Self(Box::new(predicate))
    }

    fn call(&self, module_name: &str, generation: u64) -> bool {
        (self.0)(module_name, generation)
    }
}

impl std::fmt::Debug for RetiredImageDropPredicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RetiredImageDropPredicate(<fn>)")
    }
}

#[derive(Debug)]
pub struct LoaderSession {
    report: BootstrapReport,
    compiler: ResidentCompiler,
    graph_cache: HashMap<PathBuf, CachedSourceGraph>,
    materialized_modules: HashMap<String, MaterializedModuleRecord>,
    retired_materializations: Vec<RetiredMaterialization>,
    active_executable_images: HashMap<String, ActiveExecutableImage>,
    retired_executable_images: Vec<RetiredExecutableImage>,
    active_execution_scopes: HashMap<ExecutionScopeId, ExecutionScopeRecord>,
    last_failure: Option<LoaderFailureRecord>,
    next_generation: u64,
    next_scope_id: u64,
    /// Monotonic counter advanced by `note_quiescent_point` once no execution
    /// scope is open. Retired records are reclaimed only when their
    /// `collect_after_quiescent_epoch` is `<=` this value: the `+1` offset
    /// applied at retirement time means "wait for the next quiescent
    /// boundary", which is the earliest moment we can be sure the previous
    /// generation's stack frames are gone.
    quiescent_epoch: u64,
    /// Optional veto hook over executable-image drops. See
    /// [`RetiredImageDropPredicate`].
    retired_image_drop_predicate: Option<RetiredImageDropPredicate>,
}

impl LoaderSession {
    pub fn new() -> Self {
        Self {
            report: BootstrapReport::new(),
            compiler: ResidentCompiler::bootstrap(),
            graph_cache: HashMap::new(),
            materialized_modules: HashMap::new(),
            retired_materializations: Vec::new(),
            active_executable_images: HashMap::new(),
            retired_executable_images: Vec::new(),
            active_execution_scopes: HashMap::new(),
            last_failure: None,
            next_generation: 1,
            next_scope_id: 1,
            quiescent_epoch: 0,
            retired_image_drop_predicate: None,
        }
    }

    /// Install a predicate that gets the final say on whether a retired
    /// executable image may be dropped. See [`RetiredImageDropPredicate`].
    pub fn set_retired_image_drop_predicate(&mut self, predicate: RetiredImageDropPredicate) {
        self.retired_image_drop_predicate = Some(predicate);
    }

    pub fn clear_retired_image_drop_predicate(&mut self) {
        self.retired_image_drop_predicate = None;
    }

    /// Advance quiescence and reclaim retired generations in a single step.
    ///
    /// Returns `Some(epoch)` if the cycle ran, `None` if any execution scope
    /// is currently open (in which case nothing is reclaimed). Safe to call
    /// after every command invocation; without it, retired `OwnedJitModule`s
    /// accumulate forever.
    pub fn drive_quiescent_collection(&mut self) -> Option<u64> {
        if !self.can_observe_quiescent_point() {
            return None;
        }
        let epoch = self.note_quiescent_point().ok()?;
        let _ = self.garbage_collect_retired();
        Some(epoch)
    }

    pub fn report(&self) -> &BootstrapReport {
        &self.report
    }

    pub fn retired_materializations(&self) -> &[RetiredMaterialization] {
        &self.retired_materializations
    }

    pub fn active_execution_scopes(&self) -> Vec<&ExecutionScopeRecord> {
        let mut scopes = self.active_execution_scopes.values().collect::<Vec<_>>();
        scopes.sort_by_key(|scope| scope.id.0);
        scopes
    }

    pub fn status(&self) -> LoaderSessionStatus {
        let mut active_modules = self
            .materialized_modules
            .values()
            .map(|module| LoaderModuleStatus {
                name: module.name.clone(),
                generation: module.generation,
                path: module.path.clone(),
                imports: module.imports.clone(),
                has_executable_image: module.has_executable_image,
            })
            .collect::<Vec<_>>();
        active_modules.sort_by(|left, right| left.name.cmp(&right.name));

        let mut dirty_modules = self
            .materialized_modules
            .values()
            .map(|module| DirtyModuleRecord {
                name: module.name.clone(),
                path: module.path.clone(),
                state: materialized_module_state(module),
            })
            .collect::<Vec<_>>();
        dirty_modules.sort_by(|left, right| left.name.cmp(&right.name));

        let mut active_execution_scopes = self
            .active_execution_scopes
            .values()
            .map(|scope| LoaderExecutionScopeStatus {
                id: scope.id,
                root_module: scope.root_module.clone(),
                pinned_generations: scope.pinned_generations.clone(),
            })
            .collect::<Vec<_>>();
        active_execution_scopes.sort_by_key(|scope| scope.id.0);

        LoaderSessionStatus {
            cached_graph_count: self.graph_cache.len(),
            active_modules,
            invalidation_state: if dirty_modules.iter().any(|module| module.state != DirtyModuleState::Clean) {
                LoaderInvalidationState::Dirty
            } else {
                LoaderInvalidationState::Clean
            },
            recovery_state: if self.last_failure.is_some() {
                LoaderRecoveryState::RecoverableFailure
            } else {
                LoaderRecoveryState::Ready
            },
            dirty_modules,
            retired_generations: self.retired_materializations.clone(),
            active_execution_scopes,
            retired_executable_generation_count: self.retired_executable_images.len(),
            last_failure: self.last_failure.clone(),
        }
    }

    pub fn last_failure(&self) -> Option<&LoaderFailureRecord> {
        self.last_failure.as_ref()
    }

    pub fn has_active_executable_image(&self, module_name: &str) -> bool {
        self.active_executable_images.contains_key(module_name)
    }

    pub fn active_export_address(&self, module_name: &str, public_name: &str) -> Option<usize> {
        self.active_executable_images
            .get(module_name)
            .and_then(|image| image.export_addresses.get(public_name).copied())
    }

    pub fn ensure_command_loaded(&mut self, command_path: &str) -> Result<LoaderCommandLoadResult, String> {
        let (module_name, command_name) = parse_command_path(command_path)?;
        let load = self.ensure_module_loaded(&module_name)?;
        let command = self
            .report
            .kernel
            .this_command(&module_name_from_ref(&module_name), &command_name)
            .ok_or_else(|| format!("command not found: {command_path}"))?;

        Ok(LoaderCommandLoadResult { load, command })
    }

    pub fn invoke_command(&mut self, command_path: &str) -> Result<LoaderCommandInvokeResult, String> {
        let loaded = self.ensure_command_loaded(command_path)?;
        let root_ref = loaded.load.graph.root_path.display().to_string();
        let scope_id = self.begin_execution_scope(&root_ref)?;

        // Resolve the export address up front; failure here returns through
        // the same end-of-scope cleanup as a successful run.
        let address_result = {
            let export_path = format!("{}.{}", loaded.command.module_name, loaded.command.command_name);
            self.active_export_address(&loaded.command.module_name, &export_path)
                .ok_or_else(|| format!("command executable not materialized: {command_path}"))
        };

        // Outcome of the actual JIT call: success, error, or a pending panic
        // payload that we re-raise after cleanup.
        enum Outcome {
            Ok,
            Err(String),
            Panic(Box<dyn std::any::Any + Send>),
        }

        let outcome = match address_result {
            Err(error) => Outcome::Err(error),
            Ok(address) => {
                let command_fn: unsafe extern "C" fn() = unsafe { std::mem::transmute(address) };
                // catch_unwind: a Rust-side panic crossing the JIT boundary
                // (codegen bug, debug assertion in a runtime intrinsic) must
                // not skip end_execution_scope; otherwise the scope set jams
                // and every subsequent note_quiescent_point fails forever.
                // `command_fn` is `extern "C" fn()`, so its execution is
                // UnwindSafe; we assert that for the surrounding closure.
                let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
                    command_fn();
                }));
                match panic_result {
                    Ok(()) => Outcome::Ok,
                    Err(payload) => Outcome::Panic(payload),
                }
            }
        };

        // Always close the scope. Always.
        let _ = self.end_execution_scope(scope_id);

        // Drive a quiescent collection now that the scope set is empty. This
        // is the *only* place retired generations get reclaimed in normal
        // operation; without it, every recompile leaks one OwnedJitModule.
        // No-op when another scope remains open (re-entrant call paths).
        let _ = self.drive_quiescent_collection();

        match outcome {
            Outcome::Panic(payload) => std::panic::resume_unwind(payload),
            Outcome::Err(error) => Err(error),
            Outcome::Ok => {
                let execution_log = vec![loaded.command.render()];
                Ok(LoaderCommandInvokeResult {
                    command: loaded.command,
                    load_log: loaded.load.load_log,
                    execution_log,
                    materialized_modules: loaded.load.materialized_modules,
                })
            }
        }
    }

    pub fn retired_executable_image_count(&self) -> usize {
        self.retired_executable_images.len()
    }

    pub fn begin_execution_scope(&mut self, module_ref: &str) -> Result<ExecutionScopeId, String> {
        let graph = self.ensure_import_graph_loaded(module_ref)?.graph;
        let pinned_generations = graph
            .modules
            .iter()
            .map(|module| {
                let active = self
                    .materialized_modules
                    .get(&module.spec.name)
                    .ok_or_else(|| format!("module not materialized for scope: {}", module.spec.name))?;
                Ok(PinnedGeneration {
                    module_name: active.name.clone(),
                    generation: active.generation,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let scope_id = ExecutionScopeId(self.next_scope_id);
        self.next_scope_id += 1;
        self.active_execution_scopes.insert(
            scope_id,
            ExecutionScopeRecord {
                id: scope_id,
                root_module: graph.root_spec().name.clone(),
                pinned_generations,
            },
        );
        Ok(scope_id)
    }

    pub fn end_execution_scope(&mut self, scope_id: ExecutionScopeId) -> bool {
        self.active_execution_scopes.remove(&scope_id).is_some()
    }

    pub fn can_observe_quiescent_point(&self) -> bool {
        self.active_execution_scopes.is_empty()
    }

    pub fn note_quiescent_point(&mut self) -> Result<u64, String> {
        if !self.can_observe_quiescent_point() {
            return Err("cannot mark quiescent point while execution scopes are active".to_string());
        }
        self.quiescent_epoch += 1;
        Ok(self.quiescent_epoch)
    }

    pub fn garbage_collect_retired(&mut self) -> Vec<RetiredMaterialization> {
        let pinned_generations = self
            .active_execution_scopes
            .values()
            .flat_map(|scope| scope.pinned_generations.iter())
            .map(|pinned| (pinned.module_name.clone(), pinned.generation))
            .collect::<HashSet<_>>();

        // Phase 1 — pick the (name, generation) pairs that are eligible for
        // reclamation. A pair is eligible iff:
        //   1. its retire-epoch has been reached (stack-quiescence), and
        //   2. no open execution scope is pinning it, and
        //   3. the optional drop predicate doesn't veto it (heap-quiescence
        //      hook for a future heap-introspection observer; defaults to
        //      "no veto" when no probe is registered).
        // Metadata (`RetiredMaterialization`) and image
        // (`RetiredExecutableImage`) share the same eligibility — keeping
        // them in lock-step prevents bookkeeping/image divergence on veto.
        let probe = self.retired_image_drop_predicate.as_ref();
        let mut drop_keys: HashSet<(String, u64)> = HashSet::new();
        for retired in &self.retired_materializations {
            let key = (retired.name.clone(), retired.generation);
            let stack_clear = retired.collect_after_quiescent_epoch <= self.quiescent_epoch;
            let unpinned = !pinned_generations.contains(&key);
            let probe_allows = probe.map_or(true, |p| p.call(&retired.name, retired.generation));
            if stack_clear && unpinned && probe_allows {
                drop_keys.insert(key);
            }
        }
        // Probe also gates pure-image entries that no longer have matching
        // metadata (defensive — should not happen with current code paths).
        for retired in &self.retired_executable_images {
            let key = (retired.module_name.clone(), retired.generation);
            if drop_keys.contains(&key) {
                continue;
            }
            let stack_clear = retired.collect_after_quiescent_epoch <= self.quiescent_epoch;
            let unpinned = !pinned_generations.contains(&key);
            let probe_allows =
                probe.map_or(true, |p| p.call(&retired.module_name, retired.generation));
            if stack_clear && unpinned && probe_allows && !self
                .retired_materializations
                .iter()
                .any(|r| (r.name == key.0) && r.generation == key.1)
            {
                drop_keys.insert(key);
            }
        }

        // Phase 2 — partition the metadata vector by eligibility.
        let (collected, retained_meta): (Vec<_>, Vec<_>) = self
            .retired_materializations
            .drain(..)
            .partition(|r| drop_keys.contains(&(r.name.clone(), r.generation)));
        self.retired_materializations = retained_meta;

        // Phase 3 — drop or retain images using the same key set. Drop
        // releases the OwnedJitModule, which tears down its LLVM context and
        // executable memory. After this point any pointer into that image
        // (TypeDesc addresses in heap block tags, ModuleDesc.var_base in GC
        // module roots) is dangling — see RetiredImageDropPredicate.
        let mut retained_executables = Vec::new();
        for retired in self.retired_executable_images.drain(..) {
            let key = (retired.module_name.clone(), retired.generation);
            if drop_keys.contains(&key) {
                drop(retired);
            } else {
                retained_executables.push(retired);
            }
        }
        self.retired_executable_images = retained_executables;

        collected
    }

    pub fn ensure_source_graph(&mut self, module_ref: &str) -> Result<SourceModuleGraph, String> {
        let root_path = resolve_module_source(Path::new(module_ref));

        if let Some(cached) = self.graph_cache.get(&root_path) {
            if cached.is_fresh() {
                return Ok(cached.graph.clone());
            }
        }

        let graph = discover_source_module_graph_detailed(&root_path).map_err(|failure| {
            self.record_failure_record(failure.clone());
            failure.detail
        })?;
        let file_stamps = collect_graph_file_stamps(&graph)?;
        self.graph_cache.insert(
            root_path,
            CachedSourceGraph {
                graph: graph.clone(),
                file_stamps,
            },
        );
        self.clear_failure();
        Ok(graph)
    }

    pub fn ensure_module_loaded(&mut self, module_ref: &str) -> Result<SourceLoadResult, String> {
        let graph = self.ensure_source_graph(module_ref)?;
        let materialized_modules = self.materialize_graph(&graph)?;

        let root_name = graph.root_spec().name.clone();
        let load_log = self
            .report
            .kernel
            .load_mod(&root_name)
            .map_err(|error| {
                let rendered = error.render();
                self.record_failure(
                    Some(root_name.clone()),
                    LoaderFailurePhase::RegisterRootModule,
                    rendered.clone(),
                );
                rendered
            })?;

        for entry in &load_log {
            if !self.report.init_log.iter().any(|existing| existing == entry) {
                self.report.init_log.push(entry.clone());
            }
        }

        self.clear_failure();

        Ok(SourceLoadResult {
            graph,
            load_log,
            materialized_modules,
        })
    }

    pub fn ensure_import_graph_loaded(&mut self, module_ref: &str) -> Result<SourceLoadResult, String> {
        let graph = self.ensure_source_graph(module_ref)?;
        let materialized_modules = self.materialize_graph(&graph)?;

        self.clear_failure();

        Ok(SourceLoadResult {
            graph,
            load_log: Vec::new(),
            materialized_modules,
        })
    }

    pub fn dirty_modules(&self) -> Vec<DirtyModuleRecord> {
        self.status().dirty_modules
    }

    pub fn materialization_summary(&self) -> String {
        let status = self.status();
        let modules = status
            .active_modules
            .iter()
            .map(|module| format!("{} [{}]", module.name, module.path.display()))
            .collect::<Vec<_>>();

        format!(
            concat!(
                "loader-session\n",
                "cached-graphs: {}\n",
                "active-generations: {}\n",
                "active-executable-generations: {}\n",
                "active-execution-scopes: {}\n",
                "retired-generations: {}\n",
                "retired-executable-generations: {}"
            ),
            status.cached_graph_count,
            if modules.is_empty() {
                "<none>".to_string()
            } else {
                modules.join(", ")
            },
            status.active_modules.iter().filter(|module| module.has_executable_image).count(),
            status.active_execution_scopes.len(),
            status.retired_generations.len(),
            status.retired_executable_generation_count
        )
    }

    fn materialize_graph(&mut self, graph: &SourceModuleGraph) -> Result<Vec<String>, String> {
        let mut rebuilt_modules = HashSet::new();
        let mut materialized_modules = Vec::new();
        let mut staged_updates = Vec::new();
        let mut staged_export_addresses: HashMap<String, HashMap<String, usize>> = HashMap::new();
        // Method-body addresses staged in this pass, keyed
        // module → llvm_name → address. Importers compiled later in the
        // same pass read this for cross-module vtable patching.
        let mut staged_method_addresses: HashMap<String, HashMap<String, usize>> = HashMap::new();
        let mut next_generation = self.next_generation;

        for module in &graph.modules {
            let stamp = source_file_stamp(&module.path)?;
            // Transitive rebuild propagation depends on `graph.modules` being
            // in topological / DFS post-order: any module rebuilt earlier in
            // this loop (direct or transitive import) is already in
            // `rebuilt_modules`, so checking the *direct* import list is
            // sufficient. If the discovery order ever stops being topological,
            // this loses transitivity silently — guard at the source if so.
            let dependency_changed = module
                .spec
                .imports
                .iter()
                .any(|import| rebuilt_modules.contains(import));
            let retirement_reason = if dependency_changed {
                RetirementReason::DependencyChanged
            } else {
                RetirementReason::SourceChanged
            };
            let needs_compile = self
                .materialized_modules
                .get(&module.spec.name)
                .is_none_or(|existing| {
                    existing.path != module.path || existing.stamp != stamp || dependency_changed
                });

            if !needs_compile {
                continue;
            }

            analyze_source_module(&module.path).map_err(|failure| {
                self.record_failure_record(failure.clone());
                failure.detail
            })?;

            let import_refs = module.spec.imports.iter().map(String::as_str).collect::<Vec<_>>();
            let exports = module
                .spec
                .exports
                .iter()
                .map(|export| ExportEntry {
                    name: export.name.clone(),
                    kind: match export.kind {
                        SourceExportKind::Constant => ExportKind::Constant,
                        SourceExportKind::Type => ExportKind::Type,
                        SourceExportKind::Variable => ExportKind::Variable,
                        SourceExportKind::Procedure => ExportKind::Procedure,
                        SourceExportKind::Command => ExportKind::Command,
                    },
                })
                .collect::<Vec<_>>();

            let artifact = self.compiler.compile_module(
                &module.spec.name,
                &import_refs,
                exports,
                &format!("{}.body", module.spec.name),
                &module.path.display().to_string(),
            );

            let generation = next_generation;
            next_generation += 1;
            let import_symbol_mappings =
                self.collect_import_symbol_mappings(
                    module,
                    &staged_export_addresses,
                    &staged_method_addresses,
                );
            let compiled = compile_executable_image(&module.path, generation).map_err(|error| {
                self.record_failure(
                    Some(module.spec.name.clone()),
                    LoaderFailurePhase::CodegenModule,
                    error.clone(),
                );
                error
            })?;
            let executable_image = Some(
                materialize_compiled_image(
                    &module.path,
                    generation,
                    compiled,
                    &import_symbol_mappings,
                )
                .map_err(|error| {
                    self.record_failure(
                        Some(module.spec.name.clone()),
                        LoaderFailurePhase::MaterializeModule,
                        error.clone(),
                    );
                    error
                })?,
            );
            let has_executable_image = executable_image.is_some();

            let compiled_module_entry = format!(
                "{} [compiled by {} from {}]",
                module.spec.name,
                self.compiler.service_name(),
                module.path.display()
            );

            if let Some((_, export_addresses, method_addresses)) = &executable_image {
                staged_export_addresses.insert(module.spec.name.clone(), export_addresses.clone());
                staged_method_addresses
                    .insert(module.spec.name.clone(), method_addresses.clone());
            }

            staged_updates.push(StagedModuleUpdate {
                artifact,
                compiled_module_entry,
                materialized_record: MaterializedModuleRecord {
                    name: module.spec.name.clone(),
                    generation,
                    path: module.path.clone(),
                    stamp,
                    imports: module.spec.imports.clone(),
                    has_executable_image,
                },
                executable_image,
                retirement_reason,
            });

            rebuilt_modules.insert(module.spec.name.clone());
            materialized_modules.push(module.spec.name.clone());
        }

        self.next_generation = next_generation;

        for update in staged_updates {
            let module_name = update.materialized_record.name.clone();

            if let Some(existing) = self.materialized_modules.get(&module_name).cloned() {
                self.retired_materializations.push(RetiredMaterialization {
                    name: existing.name,
                    generation: existing.generation,
                    path: existing.path,
                    reason: update.retirement_reason,
                    collect_after_quiescent_epoch: self.quiescent_epoch + 1,
                });
            }

            if let Some(existing) = self.active_executable_images.remove(&module_name) {
                self.retired_executable_images.push(RetiredExecutableImage {
                    module_name: existing.module_name,
                    generation: existing.generation,
                    collect_after_quiescent_epoch: self.quiescent_epoch + 1,
                    image: existing.image,
                });
            }

            self.report.kernel.register_compiled_module(update.artifact);
            self.report
                .hosted_modules
                .retain(|entry| !entry.starts_with(&format!("{} [", module_name)));
            self.report
                .compiled_modules
                .retain(|entry| !entry.starts_with(&format!("{} [", module_name)));
            self.report.compiled_modules.push(update.compiled_module_entry);

            if let Some((image, export_addresses, method_addresses)) = update.executable_image {
                self.active_executable_images.insert(
                    module_name.clone(),
                    ActiveExecutableImage {
                        module_name: module_name.clone(),
                        generation: update.materialized_record.generation,
                        export_addresses,
                        method_addresses,
                        image,
                    },
                );
            }

            self.materialized_modules
                .insert(module_name, update.materialized_record);
        }

        // Run each newly-materialized module's CP `BEGIN..END` body once, in
        // dependency order (graph.modules is already DFS post-order). Without
        // this, module-level VARs initialised in the body stay at their zero
        // default, breaking every module that relies on them.
        //
        // Compiler emits the body as an exported procedure named "body";
        // its symbol in the export-address map is `<ModName>.body`.
        for module in &graph.modules {
            if !materialized_modules.contains(&module.spec.name) {
                continue; // skipped above (no recompile needed)
            }
            let Some(image) = self.active_executable_images.get(&module.spec.name) else {
                continue;
            };
            let body_symbol = format!("{}.body", module.spec.name);
            let Some(&addr) = image.export_addresses.get(&body_symbol) else {
                continue; // module has no body (e.g. DEFINITION MODULE)
            };
            // Safety: the JIT-emitted body has signature `extern "C" fn()`.
            // It's been linked into the active executable image whose lifetime
            // outlives this call.
            unsafe {
                let body_fn: extern "C" fn() = std::mem::transmute(addr);
                body_fn();
            }
        }

        Ok(materialized_modules)
    }
}

fn parse_command_path(path: &str) -> Result<(String, String), String> {
    if let Some((module_ref, command_name)) = path.rsplit_once("::") {
        if module_ref.is_empty() || command_name.is_empty() {
            return Err(format!("invalid command path: {path}"));
        }
        return Ok((module_ref.to_string(), command_name.to_string()));
    }

    let mut parts = path.split('.');
    let Some(module_name) = parts.next() else {
        return Err(format!("invalid command path: {path}"));
    };
    let Some(command_name) = parts.next() else {
        return Err(format!("invalid command path: {path}"));
    };

    if parts.next().is_some() || module_name.is_empty() || command_name.is_empty() {
        return Err(format!("invalid command path: {path}"));
    }

    Ok((module_name.to_string(), command_name.to_string()))
}

impl Default for LoaderSession {
    fn default() -> Self {
        Self::new()
    }
}

impl LoaderSession {
    fn collect_import_symbol_mappings(
        &self,
        module: &SourceModuleRecord,
        staged_export_addresses: &HashMap<String, HashMap<String, usize>>,
        staged_method_addresses: &HashMap<String, HashMap<String, usize>>,
    ) -> HashMap<String, usize> {
        let mut symbol_mappings = HashMap::new();
        let debug = std::env::var("NEWCP_JIT_DEBUG").is_ok();
        if debug {
            eprintln!(
                "[loader] collect_import_symbol_mappings for {} (imports: {:?})",
                module.spec.name, module.spec.imports
            );
        }

        for import in &module.spec.imports {
            // Native (Rust-hosted) modules MUST win over any CP stub body that
            // happens to share the name.  Definition-style CP modules like
            // `WinSpec.cp` / `HostWindows.cp` exist purely to give sema/IR the
            // procedure signatures; their empty `BEGIN END` bodies must never
            // shadow the real Rust shims.
            if let Some(mod_record) = self.report.kernel.this_mod(import) {
                if matches!(mod_record.kind, ModuleKind::ResidentRust) {
                    if debug {
                        eprintln!(
                            "[loader]   import {} is native (resident Rust), exports: {:?}",
                            import,
                            mod_record.exports.names()
                        );
                    }
                    let mut bound_any = false;
                    for export_name in mod_record.exports.names() {
                        if let Some(address) =
                            newcp_runtime::native_export_address(import, export_name)
                        {
                            if debug {
                                eprintln!(
                                    "[loader]     {}.{} -> 0x{:x} (native)",
                                    import, export_name, address
                                );
                            }
                            symbol_mappings
                                .insert(format!("{}.{}", import, export_name), address);
                            bound_any = true;
                        } else if debug {
                            eprintln!(
                                "[loader]     {}.{} -> NO ADDRESS (native_export_address returned None)",
                                import, export_name
                            );
                        }
                    }
                    if bound_any {
                        continue;
                    }
                    // Fall through if the native module exposes no addresses
                    // (shouldn't happen, but stay safe).
                }
            }

            if let Some(export_addresses) = staged_export_addresses.get(import) {
                for (public_name, address) in export_addresses {
                    symbol_mappings.insert(public_name.clone(), *address);
                }
                // Also publish staged method addresses (cross-module
                // vtable patching uses qualified `<Module>.<llvm_name>`
                // keys; the IR layer emits matching slot names).
                if let Some(method_addresses) = staged_method_addresses.get(import) {
                    for (llvm_name, address) in method_addresses {
                        symbol_mappings
                            .insert(format!("{import}.{llvm_name}"), *address);
                    }
                }
                continue;
            }

            if let Some(image) = self.active_executable_images.get(import) {
                for (public_name, address) in &image.export_addresses {
                    symbol_mappings.insert(public_name.clone(), *address);
                }
                for (llvm_name, address) in &image.method_addresses {
                    symbol_mappings
                        .insert(format!("{import}.{llvm_name}"), *address);
                }
                continue;
            }

            if debug {
                eprintln!("[loader]   import {} NOT FOUND in any source", import);
            }
        }

        if debug {
            eprintln!(
                "[loader] collected {} symbol mappings for {}",
                symbol_mappings.len(),
                module.spec.name
            );
        }
        symbol_mappings
    }

    fn clear_failure(&mut self) {
        self.last_failure = None;
    }

    fn record_failure(
        &mut self,
        module_name: Option<String>,
        phase: LoaderFailurePhase,
        detail: String,
    ) {
        self.last_failure = Some(LoaderFailureRecord {
            module_name,
            phase,
            detail,
        });
    }

    fn record_failure_record(&mut self, failure: LoaderFailureRecord) {
        self.last_failure = Some(failure);
    }
}

pub fn dump_module_graph(path: &Path) -> String {
    match discover_source_module_graph(path) {
        Ok(graph) => {
            let root_spec = graph.root_spec();
            let imports = if root_spec.imports.is_empty() {
                "<none>".to_string()
            } else {
                root_spec.imports.join(", ")
            };
            let dependency_edges = if graph.dependency_edges.is_empty() {
                "<none>".to_string()
            } else {
                graph
                    .dependency_edges
                    .iter()
                    .map(|(module, import)| format!("{module} -> {import}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let runtime_imports = if graph.runtime_imports.is_empty() {
                "<none>".to_string()
            } else {
                graph
                    .runtime_imports
                    .iter()
                    .map(|(module, import)| format!("{module} -> {import}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let initialization_order = if graph.modules.is_empty() {
                "<none>".to_string()
            } else {
                graph.initialization_order().join(" -> ")
            };

            format!(
                concat!(
                    "newcp-loader module graph\n",
                    "input: {}\n",
                    "root-module: {}\n",
                    "imports: {}\n",
                    "dependency-edges: {}\n",
                    "runtime-imports: {}\n",
                    "initialization-order: {}"
                ),
                path.display(),
                root_spec.name,
                imports,
                dependency_edges,
                runtime_imports,
                initialization_order
            )
        }
        Err(error) => format!("newcp-loader module graph error\ninput: {}\nerror: {}", path.display(), error),
    }
}

pub fn discover_source_module_graph(module_ref: impl AsRef<Path>) -> Result<SourceModuleGraph, String> {
    discover_source_module_graph_detailed(module_ref).map_err(|failure| failure.detail)
}

fn discover_source_module_graph_detailed(
    module_ref: impl AsRef<Path>,
) -> Result<SourceModuleGraph, LoaderFailureRecord> {
    let root_path = resolve_module_source(module_ref.as_ref());
    let mut state = GraphDiscoveryState::default();
    visit_source_module(&root_path, &mut state)?;

    Ok(SourceModuleGraph {
        root_path,
        modules: state.modules,
        dependency_edges: state.dependency_edges,
        runtime_imports: state.runtime_imports,
    })
}

pub fn register_source_module_graph(
    report: &mut BootstrapReport,
    compiler: &ResidentCompiler,
    module_ref: &str,
) -> Result<SourceLoadResult, String> {
    let graph = discover_source_module_graph(module_ref)?;
    let mut materialized_modules = Vec::new();

    for module in &graph.modules {
        let import_refs = module.spec.imports.iter().map(String::as_str).collect::<Vec<_>>();
        let exports = module
            .spec
            .exports
            .iter()
            .map(|export| ExportEntry {
                name: export.name.clone(),
                kind: match export.kind {
                    SourceExportKind::Constant => ExportKind::Constant,
                    SourceExportKind::Type => ExportKind::Type,
                    SourceExportKind::Variable => ExportKind::Variable,
                    SourceExportKind::Procedure => ExportKind::Procedure,
                    SourceExportKind::Command => ExportKind::Command,
                },
            })
            .collect::<Vec<_>>();

        let artifact = compiler.compile_module(
            &module.spec.name,
            &import_refs,
            exports,
            &format!("{}.body", module.spec.name),
            &module.path.display().to_string(),
        );

        report.kernel.register_compiled_module(artifact);
        report
            .hosted_modules
            .retain(|entry| !entry.starts_with(&format!("{} [", module.spec.name)));
        report
            .compiled_modules
            .retain(|entry| !entry.starts_with(&format!("{} [", module.spec.name)));
        report.compiled_modules.push(format!(
            "{} [compiled by {} from {}]",
            module.spec.name,
            compiler.service_name(),
            module.path.display()
        ));
        materialized_modules.push(module.spec.name.clone());
    }

    let root_name = graph.root_spec().name.clone();
    let load_log = report
        .kernel
        .load_mod(&root_name)
        .map_err(|error| error.render())?;

    for entry in &load_log {
        if !report.init_log.iter().any(|existing| existing == entry) {
            report.init_log.push(entry.clone());
        }
    }

    Ok(SourceLoadResult {
        graph,
        load_log,
        materialized_modules,
    })
}

pub fn can_resolve_module_source(module_ref: &str) -> bool {
    resolve_module_source(Path::new(module_ref)).exists()
}

pub fn module_name_from_ref(module_ref: &str) -> String {
    Path::new(module_ref)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(module_ref)
        .to_string()
}

pub fn resolve_module_source(module_ref: &Path) -> PathBuf {
    let path = PathBuf::from(module_ref);
    if path.exists() {
        return path;
    }

    if path.extension().is_some() {
        Path::new("Mod").join(path)
    } else {
        Path::new("Mod").join(format!("{}.cp", module_ref.display()))
    }
}

#[derive(Default)]
struct GraphDiscoveryState {
    modules: Vec<SourceModuleRecord>,
    dependency_edges: Vec<(String, String)>,
    runtime_imports: Vec<(String, String)>,
    visited: HashSet<String>,
    active: Vec<String>,
}

fn visit_source_module(path: &Path, state: &mut GraphDiscoveryState) -> Result<(), LoaderFailureRecord> {
    let source_text = fs::read_to_string(path).map_err(|error| LoaderFailureRecord {
        module_name: Some(module_name_from_path(path)),
        phase: LoaderFailurePhase::ReadModuleSource,
        detail: format!("failed to read {}: {error}", path.display()),
    })?;
    let spec = parse_source_module(&source_text).map_err(|error| LoaderFailureRecord {
        module_name: Some(module_name_from_path(path)),
        phase: LoaderFailurePhase::ParseModule,
        detail: error,
    })?;

    if state.visited.contains(&spec.name) {
        return Ok(());
    }
    if state.active.iter().any(|name| name == &spec.name) {
        let cycle = state
            .active
            .iter()
            .cloned()
            .chain(std::iter::once(spec.name.clone()))
            .collect::<Vec<_>>()
            .join(" -> ");
        return Err(LoaderFailureRecord {
            module_name: Some(spec.name.clone()),
            phase: LoaderFailurePhase::DiscoverGraph,
            detail: format!("source module import cycle detected: {cycle}"),
        });
    }

    state.active.push(spec.name.clone());
    for import in &spec.imports {
        state.dependency_edges.push((spec.name.clone(), import.clone()));
        if let Some(import_path) = resolve_import_source(import, path) {
            visit_source_module(&import_path, state)?;
        } else {
            state.runtime_imports.push((spec.name.clone(), import.clone()));
        }
    }
    state.active.pop();
    state.visited.insert(spec.name.clone());

    // Definition modules provide type information only; they are implemented
    // by a Rust-native module registered under the same name.  Do not add
    // them to the compilable source-module list.
    if spec.is_definition {
        return Ok(());
    }

    state.modules.push(SourceModuleRecord {
        path: path.to_path_buf(),
        spec,
    });
    Ok(())
}

fn resolve_import_source(import: &str, source_path: &Path) -> Option<PathBuf> {
    let filename = format!("{import}.cp");

    // 1. Same directory as the importing module (fastest / most common case).
    if let Some(parent) = source_path.parent() {
        let sibling = parent.join(&filename);
        if sibling.exists() {
            return Some(sibling);
        }
    }

    // 2. Walk up from the source file's directory to find the "Mod" root, then
    //    search it recursively.  This lets modules in Mod/Tests/ import modules
    //    that live in Mod/ (or any other subfolder under the same root).
    if let Some(mut dir) = source_path.parent() {
        // Walk up until we find a directory named "Mod" (case-sensitive).
        loop {
            if dir.file_name().and_then(|n| n.to_str()) == Some("Mod") {
                // Found the module root — search its entire subtree.
                if let Some(hit) = find_cp_in_dir(dir, &filename) {
                    return Some(hit);
                }
                break;
            }
            match dir.parent() {
                Some(p) => dir = p,
                None => break,
            }
        }
    }

    None
}

/// Recursively search `dir` for a file named `filename`.  Returns the first
/// match found (order is unspecified beyond depth-first traversal).
fn find_cp_in_dir(dir: &Path, filename: &str) -> Option<PathBuf> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return None,
    };
    let mut subdirs: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            subdirs.push(path);
        } else if path.file_name().and_then(|n| n.to_str()) == Some(filename) {
            return Some(path);
        }
    }
    for sub in subdirs {
        if let Some(hit) = find_cp_in_dir(&sub, filename) {
            return Some(hit);
        }
    }
    None
}

fn module_name_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| path.display().to_string())
}

fn analyze_source_module(path: &Path) -> Result<(), LoaderFailureRecord> {
    let module = analyze_module(path).map_err(|error| LoaderFailureRecord {
        module_name: Some(module_name_from_path(path)),
        phase: LoaderFailurePhase::ParseModule,
        detail: error,
    })?;

    let diagnostics = render_semantic_diagnostics(&module);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(LoaderFailureRecord {
            module_name: Some(module.name.clone()),
            phase: LoaderFailurePhase::AnalyzeModule,
            detail: diagnostics.join(" | "),
        })
    }
}

fn render_semantic_diagnostics(module: &SemanticModule) -> Vec<String> {
    let mut diagnostics = module
        .diagnostics
        .iter()
        .map(|diagnostic| format_semantic_diagnostic("<module>", diagnostic))
        .collect::<Vec<_>>();

    for procedure in &module.procedures {
        for diagnostic in &procedure.diagnostics {
            diagnostics.push(format_semantic_diagnostic(&procedure.name, diagnostic));
        }
    }

    diagnostics
}

fn format_semantic_diagnostic(scope: &str, diagnostic: &SemanticDiagnostic) -> String {
    let scope_label = diagnostic.procedure.as_deref().unwrap_or(scope);
    format!(
        "error {}@{}:{}: {}",
        scope_label, diagnostic.line, diagnostic.column, diagnostic.message
    )
}

fn source_file_stamp(path: &Path) -> Result<SourceFileStamp, String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("failed to stat {}: {error}", path.display()))?;
    let modified = metadata
        .modified()
        .map_err(|error| format!("failed to read modified time for {}: {error}", path.display()))?;
    let modified_unix_ms = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("modified time for {} predates unix epoch: {error}", path.display()))?
        .as_millis();

    Ok(SourceFileStamp {
        size_bytes: metadata.len(),
        modified_unix_ms,
    })
}

fn collect_graph_file_stamps(graph: &SourceModuleGraph) -> Result<Vec<(PathBuf, SourceFileStamp)>, String> {
    graph
        .modules
        .iter()
        .map(|module| Ok((module.path.clone(), source_file_stamp(&module.path)?)))
        .collect()
}

fn materialized_module_state(module: &MaterializedModuleRecord) -> DirtyModuleState {
    match source_file_stamp(&module.path) {
        Ok(stamp) if stamp == module.stamp => DirtyModuleState::Clean,
        Ok(_) => DirtyModuleState::Modified,
        Err(_) => DirtyModuleState::Missing,
    }
}

fn compile_executable_image(path: &Path, generation: u64) -> Result<CompiledModule, String> {
    #[cfg(test)]
    {
        let source_text = fs::read_to_string(path)
            .map_err(|error| format!("failed to read {} for test build hook: {error}", path.display()))?;
        if source_text.contains("LOADER_TEST_FAIL_BUILD") {
            return Err(format!(
                "test-injected loader build failure for {}",
                path.display()
            ));
        }
    }

    let mut options = CodegenOptions::default();
    options.export_generation = Some(generation);
    newcp_llvm::compile_from_path(path, &options)
        .map_err(|error| format!("failed to compile executable image from {}: {error}", path.display()))
}

fn materialize_compiled_image(
    path: &Path,
    generation: u64,
    compiled: CompiledModule,
    import_symbol_mappings: &HashMap<String, usize>,
) -> Result<
    (
        OwnedJitModule,
        HashMap<String, usize>,
        HashMap<String, usize>,
    ),
    String,
> {
    let mut options = CodegenOptions::default();
    options.export_generation = Some(generation);
    // Public exports the loader needs to resolve to JIT addresses:
    //   - emitted procedures from `compiled.exported_functions`
    //     (correctly excludes abstract/forward declarations that have
    //     no native body)
    //   - exported VARiables, picked from the source AST (these aren't
    //     "functions" so they don't appear in `exported_functions`,
    //     but tests like `Counter.n` rely on resolving them).
    let mut public_exports: Vec<String> = compiled
        .exported_functions
        .iter()
        .map(|export| export.public_name.clone())
        .collect();
    if let Ok(module_spec) = newcp_parser::read_source_module(path) {
        for export in &module_spec.exports {
            if matches!(export.kind, SourceExportKind::Variable) {
                public_exports.push(format!("{}.{}", module_spec.name, export.name));
            }
        }
    }
    let image = OwnedJitModule::from_compiled_with_symbol_mappings(compiled, &options, import_symbol_mappings)
        .map_err(|error| format!("failed to JIT materialize executable image from {}: {error}", path.display()))?;
    let export_addresses = public_exports
        .into_iter()
        .map(|public_name| {
            let address = image.export_address(&public_name).map_err(|error| {
                format!(
                    "failed to resolve exported address '{public_name}' from {}: {error}",
                    path.display()
                )
            })?;
            Ok((public_name, address))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;

    // Method-body addresses keyed by their LLVM symbol — exposed to
    // downstream importers as `<ImportedModule>.<llvm_name>` for
    // cross-module vtable patching.
    let method_addresses = image.collect_method_addresses();

    Ok((image, export_addresses, method_addresses))
}

pub fn bootstrap_plan() -> String {
    [
        "newcp-loader bootstrap plan",
        "status: bootstrap sequencing not implemented yet",
        "steps:",
        "  1. start resident kernel",
        "  2. start resident init",
        "  3. expose and initialize resident compiler services",
        "  4. open JIT session",
        "  5. compile first base modules into memory",
        "  6. register modules and run module bodies",
    ]
    .join("\n")
}

pub fn bootstrap_report() -> String {
    let session = LoaderSession::new();
    format!(
        "{}\n{}",
        session.report().render(),
        session.materialization_summary()
    )
}

pub fn blackbox_like_bootstrap_demo() -> String {
    let mut kernel = KernelState::bootstrap("x86_64-pc-windows-msvc");
    let compiler = ResidentCompiler::bootstrap();
    kernel.register_resident_module("Kernel");
    kernel.register_resident_module("Init");
    kernel.register_hosted_module(HostedModuleArtifact::new(
        "HostMenus",
        vec!["Kernel".to_string()],
        ExportDirectory::new(vec![ExportEntry::command("OpenApp")]),
        "HostMenus.bootstrap",
        "Rust-hosted facade until CP HostMenus is available",
        vec![RustCommandHandlerSpec::new(
            "OpenApp",
            "open the outer application shell stub",
        )],
    ));
    kernel.register_compiled_module(compiler.compile_module(
        "System",
        &["Kernel"],
        vec![ExportEntry::procedure("Start")],
        "System.body",
        "MODULE System; BEGIN END System.",
    ));
    kernel.register_compiled_module(compiler.compile_module(
        "InitShell",
        &["Kernel", "System", "HostMenus"],
        vec![ExportEntry::command("EnterShell")],
        "InitShell.body",
        "MODULE InitShell; IMPORT System, HostMenus; BEGIN END InitShell.",
    ));

    match kernel.load_mod("InitShell") {
        Ok(log) => format!(
            "blackbox-like bootstrap demo\nresident-compiler: {}\nthis_mod(InitShell): {}\ninit-log: {}",
            compiler.service_name(),
            kernel.this_mod("InitShell").map(|module| module.name.as_str()).unwrap_or("missing"),
            log.join(" | ")
        ),
        Err(error) => format!("blackbox-like bootstrap demo\nerror: {}", error.render()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_plan_mentions_compiling_base_modules() {
        let plan = bootstrap_plan();

        assert!(plan.contains("start resident kernel"));
        assert!(plan.contains("start resident init"));
        assert!(plan.contains("expose and initialize resident compiler services"));
        assert!(plan.contains("compile first base modules into memory"));
    }

    #[test]
    fn bootstrap_report_uses_loader_session_state() {
        let report = bootstrap_report();

        assert!(report.contains("newcp-runtime bootstrap report"));
        assert!(report.contains("loader-session"));
        assert!(report.contains("cached-graphs: 0"));
        assert!(report.contains("active-generations: <none>"));
    }

    #[test]
    fn blackbox_like_bootstrap_demo_initializes_import_chain() {
        let demo = blackbox_like_bootstrap_demo();

        assert!(demo.contains("resident-compiler: resident-compiler"));
        assert!(demo.contains("this_mod(InitShell): InitShell"));
        assert!(demo.contains("init-log: init Kernel via bootstrap | init System via System.body | init HostMenus via HostMenus.bootstrap | init InitShell via InitShell.body"));
    }

    #[test]
    fn module_graph_lists_import_edges() {
        let temp = std::env::temp_dir().join("newcp-loader-test.cp");
        std::fs::write(&temp, "MODULE Demo;\nIMPORT Kernel, System;\nEND Demo.")
            .expect("write test module");

        let dump = dump_module_graph(&temp);
        let _ = std::fs::remove_file(&temp);

        assert!(dump.contains("root-module: Demo"));
        assert!(dump.contains("dependency-edges: Demo -> Kernel, Demo -> System"));
        assert!(dump.contains("runtime-imports: Demo -> Kernel, Demo -> System"));
        assert!(dump.contains("initialization-order: Demo"));
    }

    #[test]
    fn discovers_recursive_source_module_graph_in_dependency_order() {
        let root_dir = std::env::temp_dir().join("newcp-loader-graph");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create loader graph temp dir");
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; END Leaf.")
            .expect("write Leaf.cp");
        std::fs::write(root_dir.join("Middle.cp"), "MODULE Middle; IMPORT Leaf; END Middle.")
            .expect("write Middle.cp");
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Middle, Kernel; END Root.")
            .expect("write Root.cp");

        let graph = discover_source_module_graph(root_dir.join("Root.cp")).expect("discover graph");

        assert_eq!(graph.initialization_order(), vec!["Leaf", "Middle", "Root"]);
        assert!(graph.dependency_edges.contains(&("Root".to_string(), "Middle".to_string())));
        assert!(graph.runtime_imports.contains(&("Root".to_string(), "Kernel".to_string())));

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn register_source_module_graph_compiles_source_backed_imports() {
        let root_dir = std::env::temp_dir().join("newcp-loader-register");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create loader register temp dir");
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; PROCEDURE Ping*; BEGIN END Ping; END Leaf.")
            .expect("write Leaf.cp");
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; PROCEDURE Run*; BEGIN END Run; END Root.")
            .expect("write Root.cp");

        let mut report = BootstrapReport::new();
        let compiler = ResidentCompiler::bootstrap();
        let result = register_source_module_graph(
            &mut report,
            &compiler,
            root_dir.join("Root.cp").to_str().expect("utf-8 temp path"),
        )
        .expect("register graph");

        assert_eq!(result.graph.initialization_order(), vec!["Leaf", "Root"]);
        assert!(report.kernel.this_mod("Leaf").is_some());
        assert!(report.kernel.this_mod("Root").is_some());
        assert!(result.load_log.iter().any(|entry| entry.contains("init Root via Root.body")));

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn loader_session_reuses_unchanged_materializations() {
        let root_dir = std::env::temp_dir().join("newcp-loader-session-reuse");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create loader session temp dir");
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; END Leaf.")
            .expect("write Leaf.cp");
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; END Root.")
            .expect("write Root.cp");

        let mut session = LoaderSession::new();
        let first = session
            .ensure_module_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("first load");
        let compiled_after_first = session.report().compiled_modules.len();

        let second = session
            .ensure_module_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("second load");

        assert_eq!(first.graph.initialization_order(), vec!["Leaf", "Root"]);
        assert!(second.load_log.is_empty());
        assert!(second.materialized_modules.is_empty());
        assert_eq!(session.report().compiled_modules.len(), compiled_after_first);
        assert!(session.materialization_summary().contains("cached-graphs: 1"));
        assert!(session.materialization_summary().contains("retired-generations: 0"));

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn loader_session_refreshes_cached_graph_after_source_edit() {
        let root_dir = std::env::temp_dir().join("newcp-loader-session-refresh");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create loader session refresh temp dir");
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; END Leaf.")
            .expect("write Leaf.cp");
        std::fs::write(root_dir.join("Extra.cp"), "MODULE Extra; END Extra.")
            .expect("write Extra.cp");
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; END Root.")
            .expect("write Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_module_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("initial load");

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(
            root_dir.join("Root.cp"),
            "MODULE Root; IMPORT Leaf, Extra; END Root.",
        )
        .expect("rewrite Root.cp");

        let refreshed = session
            .ensure_module_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("refreshed load");

        assert_eq!(refreshed.graph.initialization_order(), vec!["Leaf", "Extra", "Root"]);
        assert_eq!(refreshed.materialized_modules, vec!["Extra", "Root"]);
        assert!(session.report().kernel.this_mod("Extra").is_some());

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn loader_session_marks_dirty_modules_before_refresh() {
        let root_dir = std::env::temp_dir().join("newcp-loader-session-dirty");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create loader session dirty temp dir");
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; END Leaf.")
            .expect("write Leaf.cp");
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; END Root.")
            .expect("write Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_module_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("initial load");

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; CONST X* = 1; END Leaf.")
            .expect("rewrite Leaf.cp");

        let dirty = session.dirty_modules();
        let status = session.status();

        assert!(dirty.iter().any(|module| {
            module.name == "Leaf" && module.state == DirtyModuleState::Modified
        }));
        assert!(dirty.iter().any(|module| {
            module.name == "Root" && module.state == DirtyModuleState::Clean
        }));
        assert_eq!(status.invalidation_state, LoaderInvalidationState::Dirty);
        assert_eq!(status.recovery_state, LoaderRecoveryState::Ready);
        assert_eq!(status.dirty_modules, dirty);

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn loader_session_recompiles_importers_after_dependency_edit() {
        let root_dir = std::env::temp_dir().join("newcp-loader-session-importers");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create loader session importer temp dir");
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; END Leaf.")
            .expect("write Leaf.cp");
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; END Root.")
            .expect("write Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_module_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("initial load");
        let original_root_generation = session
            .materialized_modules
            .get("Root")
            .map(|module| module.generation)
            .expect("Root generation after initial load");

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; TYPE T* = RECORD END; END Leaf.")
            .expect("rewrite Leaf.cp");

        let refreshed = session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("refresh graph");

        assert_eq!(refreshed.materialized_modules, vec!["Leaf", "Root"]);
        assert!(refreshed.load_log.is_empty());
        assert!(session.retired_materializations().iter().any(|retired| {
            retired.name == "Root"
                && retired.generation == original_root_generation
                && retired.reason == RetirementReason::DependencyChanged
        }));
        assert_eq!(
            session
                .report()
                .kernel
                .this_mod("Root")
                .map(|module| module.initialized),
            Some(false)
        );

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn retired_generations_wait_for_quiescent_gc() {
        let root_dir = std::env::temp_dir().join("newcp-loader-retired-gc");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create retired gc temp dir");
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; END Leaf.")
            .expect("write Leaf.cp");
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; END Root.")
            .expect("write Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_module_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("initial load");

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; CONST X* = 1; END Root.")
            .expect("rewrite Root.cp");
        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("refresh load");

        assert_eq!(session.retired_materializations().len(), 1);
        assert!(session.garbage_collect_retired().is_empty());

        session.note_quiescent_point().expect("quiescent point after refresh");
        let collected = session.garbage_collect_retired();

        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].name, "Root");
        assert!(session.retired_materializations().is_empty());

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn loader_session_keeps_runtime_only_executable_image_active() {
        let root_dir = std::env::temp_dir().join("newcp-loader-exec-active");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create executable image temp dir");
        std::fs::write(
            root_dir.join("Root.cp"),
            concat!(
                "MODULE Root;\n",
                "IMPORT Console;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Console.WriteLn()\n",
                "END Run;\n",
                "END Root."
            ),
        )
        .expect("write Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("load runtime-only executable module");

        assert!(session.has_active_executable_image("Root"));
        assert!(session.materialization_summary().contains("active-executable-generations: 1"));

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn loader_session_retires_and_collects_executable_images() {
        let root_dir = std::env::temp_dir().join("newcp-loader-exec-retire");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create executable retirement temp dir");
        std::fs::write(
            root_dir.join("Root.cp"),
            concat!(
                "MODULE Root;\n",
                "IMPORT Console;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Console.WriteLn()\n",
                "END Run;\n",
                "END Root."
            ),
        )
        .expect("write Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("initial runtime-only executable load");
        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(
            root_dir.join("Root.cp"),
            concat!(
                "MODULE Root;\n",
                "IMPORT Console;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Console.WriteInt(1);\n",
                "  Console.WriteLn()\n",
                "END Run;\n",
                "END Root."
            ),
        )
        .expect("rewrite Root.cp");

        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("reload runtime-only executable module");

        assert_eq!(session.retired_executable_image_count(), 1);
        assert!(session.garbage_collect_retired().is_empty());
        session.note_quiescent_point().expect("quiescent point after executable replacement");
        let collected = session.garbage_collect_retired();

        assert_eq!(collected.len(), 1);
        assert_eq!(session.retired_executable_image_count(), 0);
        assert!(session.has_active_executable_image("Root"));

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn loader_session_materializes_source_backed_importers() {
        let root_dir = std::env::temp_dir().join("newcp-loader-exec-source-backed");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create source-backed executable temp dir");
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; PROCEDURE Ping*; BEGIN END Ping; END Leaf.")
            .expect("write Leaf.cp");
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; PROCEDURE Run*; BEGIN Leaf.Ping END Run; END Root.")
            .expect("write Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("load graph with source-backed import execution");

        assert!(session.has_active_executable_image("Leaf"));
        assert!(session.has_active_executable_image("Root"));

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn loader_session_executes_source_backed_import_call_chain() {
        let root_dir = std::env::temp_dir().join("newcp-loader-exec-call-chain");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create call-chain executable temp dir");
        std::fs::write(
            root_dir.join("Leaf.cp"),
            concat!(
                "MODULE Leaf;\n",
                "IMPORT Console;\n",
                "PROCEDURE Ping*;\n",
                "BEGIN\n",
                "  Console.WriteInt(7);\n",
                "  Console.WriteLn()\n",
                "END Ping;\n",
                "END Leaf."
            ),
        )
        .expect("write Leaf.cp");
        std::fs::write(
            root_dir.join("Root.cp"),
            concat!(
                "MODULE Root;\n",
                "IMPORT Leaf;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Leaf.Ping()\n",
                "END Run;\n",
                "END Root."
            ),
        )
        .expect("write Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("load source-backed call chain executable images");

        newcp_runtime::console::reset();
        newcp_runtime::console::begin_capture();
        let run_address = session
            .active_export_address("Root", "Root.Run")
            .expect("Root.Run active export address");
        let run: unsafe extern "C" fn() = unsafe { std::mem::transmute(run_address) };
        unsafe { run() };

        assert_eq!(newcp_runtime::console::end_capture(), "7\n");
        newcp_runtime::console::reset();

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn failed_reload_preserves_last_known_good_generation() {
        let root_dir = std::env::temp_dir().join("newcp-loader-failed-reload");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create failed reload temp dir");
        std::fs::write(
            root_dir.join("Root.cp"),
            concat!(
                "MODULE Root;\n",
                "IMPORT Console;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Console.WriteLn()\n",
                "END Run;\n",
                "END Root."
            ),
        )
        .expect("write Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("initial runtime-only executable load");

        let original_generation = session
            .materialized_modules
            .get("Root")
            .map(|module| module.generation)
            .expect("Root generation after initial load");
        let original_active_address = session
            .active_export_address("Root", "Root.Run")
            .expect("Root.Run address after initial load");

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(
            root_dir.join("Root.cp"),
            concat!(
                "MODULE Root;\n",
                "IMPORT Console;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Console.WriteLn(\n",
                "END Run;\n",
                "END Root."
            ),
        )
        .expect("rewrite Root.cp with syntax error");

        let error = session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect_err("reload with invalid source should fail");

        assert!(!error.is_empty());
        assert_eq!(
            session.last_failure(),
            Some(&LoaderFailureRecord {
                module_name: Some("Root".to_string()),
                phase: LoaderFailurePhase::ParseModule,
                detail: error.clone(),
            })
        );
        assert_eq!(
            session
                .materialized_modules
                .get("Root")
                .map(|module| module.generation),
            Some(original_generation)
        );
        assert_eq!(session.retired_materializations().len(), 0);
        assert_eq!(session.retired_executable_image_count(), 0);
        assert_eq!(
            session.active_export_address("Root", "Root.Run"),
            Some(original_active_address)
        );

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn status_reports_active_modules_and_failures() {
        let root_dir = std::env::temp_dir().join("newcp-loader-status");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create status temp dir");
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; PROCEDURE Ping*; BEGIN END Ping; END Leaf.")
            .expect("write Leaf.cp");
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; PROCEDURE Run*; BEGIN Leaf.Ping() END Run; END Root.")
            .expect("write Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("load graph for status");

        let status = session.status();

        assert_eq!(status.cached_graph_count, 1);
        assert_eq!(status.active_modules.len(), 2);
        assert!(status.active_modules.iter().any(|module| module.name == "Leaf"));
        assert!(status.active_modules.iter().any(|module| module.name == "Root"));
        assert_eq!(status.invalidation_state, LoaderInvalidationState::Clean);
        assert_eq!(status.recovery_state, LoaderRecoveryState::Ready);
        assert!(status.dirty_modules.iter().all(|module| module.state == DirtyModuleState::Clean));
        assert!(status.last_failure.is_none());

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn ensure_command_loaded_returns_command_invocation() {
        let root_dir = std::env::temp_dir().join("newcp-loader-command");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create command temp dir");
        let root_path = root_dir.join("Root.cp");
        std::fs::write(&root_path, "MODULE Root; PROCEDURE Run*; BEGIN END Run; END Root.")
            .expect("write Root.cp");

        let mut session = LoaderSession::new();
        let command_path = format!("{}::Run", root_path.display());
        let result = session.ensure_command_loaded(&command_path).expect("load Root.Run command");

        assert_eq!(result.command.module_name, "Root");
        assert_eq!(result.command.command_name, "Run");
        assert!(result.load.graph.initialization_order().contains(&"Root".to_string()));

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn ensure_command_loaded_rejects_invalid_path() {
        let mut session = LoaderSession::new();

        let error = session
            .ensure_command_loaded("BrokenPath")
            .expect_err("invalid command path should fail");

        assert_eq!(error, "invalid command path: BrokenPath");
    }

    #[test]
    fn invoke_command_executes_source_backed_command() {
        let root_dir = std::env::temp_dir().join("newcp-loader-invoke-command");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create invoke command temp dir");
        let root_path = root_dir.join("Root.cp");
        std::fs::write(
            &root_path,
            concat!(
                "MODULE Root;\n",
                "IMPORT Console;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Console.WriteInt(9);\n",
                "  Console.WriteLn()\n",
                "END Run;\n",
                "END Root."
            ),
        )
        .expect("write Root.cp");

        let mut session = LoaderSession::new();
        newcp_runtime::console::reset();
        newcp_runtime::console::begin_capture();
        let command_path = format!("{}::Run", root_path.display());

        let result = session
            .invoke_command(&command_path)
            .expect("invoke source-backed Root.Run command");

        assert_eq!(result.command.module_name, "Root");
        assert_eq!(result.command.command_name, "Run");
        assert_eq!(result.execution_log, vec!["invoke Root.Run".to_string()]);
        assert_eq!(newcp_runtime::console::end_capture(), "9\n");
        newcp_runtime::console::reset();

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn invoke_command_rejects_missing_command_export() {
        let root_dir = std::env::temp_dir().join("newcp-loader-invoke-missing");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create invoke missing temp dir");
        let root_path = root_dir.join("Root.cp");
        std::fs::write(&root_path, "MODULE Root; END Root.")
            .expect("write Root.cp");

        let mut session = LoaderSession::new();
        let command_path = format!("{}::Run", root_path.display());
        let error = session
            .invoke_command(&command_path)
            .expect_err("missing command export should fail");

        assert_eq!(error, format!("command not found: {command_path}"));

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn failed_graph_reload_preserves_last_known_good_generations() {
        let root_dir = std::env::temp_dir().join("newcp-loader-failed-graph-reload");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create failed graph reload temp dir");
        std::fs::write(
            root_dir.join("Leaf.cp"),
            concat!(
                "MODULE Leaf;\n",
                "IMPORT Console;\n",
                "PROCEDURE Ping*;\n",
                "BEGIN\n",
                "  Console.WriteInt(1);\n",
                "  Console.WriteLn()\n",
                "END Ping;\n",
                "END Leaf."
            ),
        )
        .expect("write Leaf.cp");
        std::fs::write(
            root_dir.join("Root.cp"),
            concat!(
                "MODULE Root;\n",
                "IMPORT Leaf;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Leaf.Ping()\n",
                "END Run;\n",
                "END Root."
            ),
        )
        .expect("write Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("initial graph load");

        let original_leaf_generation = session
            .materialized_modules
            .get("Leaf")
            .map(|module| module.generation)
            .expect("Leaf generation after initial load");
        let original_root_generation = session
            .materialized_modules
            .get("Root")
            .map(|module| module.generation)
            .expect("Root generation after initial load");
        let original_leaf_address = session
            .active_export_address("Leaf", "Leaf.Ping")
            .expect("Leaf.Ping address after initial load");
        let original_root_address = session
            .active_export_address("Root", "Root.Run")
            .expect("Root.Run address after initial load");

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(
            root_dir.join("Leaf.cp"),
            concat!(
                "MODULE Leaf;\n",
                "IMPORT Console;\n",
                "PROCEDURE Ping*;\n",
                "BEGIN\n",
                "  Console.WriteInt(2);\n",
                "  Console.WriteLn()\n",
                "END Ping;\n",
                "END Leaf."
            ),
        )
        .expect("rewrite Leaf.cp");
        std::fs::write(
            root_dir.join("Root.cp"),
            concat!(
                "MODULE Root;\n",
                "IMPORT Leaf;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Leaf.Ping()\n",
                "END Run;\n",
                "(* LOADER_TEST_FAIL_BUILD *)\n",
                "END Root."
            ),
        )
        .expect("rewrite Root.cp with injected build failure");

        let error = session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect_err("graph reload with bad downstream module should fail");

        assert!(!error.is_empty());
        assert_eq!(
            session.last_failure(),
            Some(&LoaderFailureRecord {
                module_name: Some("Root".to_string()),
                phase: LoaderFailurePhase::CodegenModule,
                detail: error.clone(),
            })
        );
        assert_eq!(
            session
                .materialized_modules
                .get("Leaf")
                .map(|module| module.generation),
            Some(original_leaf_generation)
        );
        assert_eq!(
            session
                .materialized_modules
                .get("Root")
                .map(|module| module.generation),
            Some(original_root_generation)
        );
        assert_eq!(session.retired_materializations().len(), 0);
        assert_eq!(session.retired_executable_image_count(), 0);
        assert_eq!(
            session.active_export_address("Leaf", "Leaf.Ping"),
            Some(original_leaf_address)
        );
        assert_eq!(
            session.active_export_address("Root", "Root.Run"),
            Some(original_root_address)
        );

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn failed_graph_reload_can_retry_with_cached_updated_graph() {
        let root_dir = std::env::temp_dir().join("newcp-loader-graph-retry");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create graph retry temp dir");
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; END Leaf.")
            .expect("write Leaf.cp");
        std::fs::write(root_dir.join("Extra.cp"), "MODULE Extra; END Extra.")
            .expect("write Extra.cp");
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; END Root.")
            .expect("write Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("initial graph load");

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(
            root_dir.join("Root.cp"),
            concat!(
                "MODULE Root;\n",
                "IMPORT Leaf, Extra;\n",
                "(* LOADER_TEST_FAIL_BUILD *)\n",
                "END Root."
            ),
        )
        .expect("rewrite Root.cp with expanded graph and injected build failure");

        let error = session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect_err("reload with expanded graph and build failure should fail");

        assert!(!error.is_empty());
        assert_eq!(
            session.last_failure().map(|failure| failure.phase),
            Some(LoaderFailurePhase::CodegenModule)
        );
        assert!(session.report().kernel.this_mod("Extra").is_none());

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(
            root_dir.join("Root.cp"),
            "MODULE Root; IMPORT Leaf, Extra; END Root.",
        )
        .expect("rewrite Root.cp valid again");

        let retried = session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("retry after failed graph rebuild should succeed");

        assert_eq!(retried.graph.initialization_order(), vec!["Leaf", "Extra", "Root"]);
        assert_eq!(retried.materialized_modules, vec!["Extra", "Root"]);
        assert!(session.report().kernel.this_mod("Extra").is_some());
        assert!(session.last_failure().is_none());

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn successful_reload_clears_last_failure() {
        let root_dir = std::env::temp_dir().join("newcp-loader-clear-failure");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create clear failure temp dir");
        std::fs::write(
            root_dir.join("Root.cp"),
            concat!(
                "MODULE Root;\n",
                "IMPORT Console;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Console.WriteLn()\n",
                "END Run;\n",
                "END Root."
            ),
        )
        .expect("write Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("initial load");

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(
            root_dir.join("Root.cp"),
            concat!(
                "MODULE Root;\n",
                "IMPORT Console;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Console.WriteLn(\n",
                "END Run;\n",
                "END Root."
            ),
        )
        .expect("rewrite Root.cp with syntax error");

        let _ = session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect_err("reload with invalid source should fail");
        let failed_status = session.status();
        assert!(session.last_failure().is_some());
        assert_eq!(failed_status.invalidation_state, LoaderInvalidationState::Dirty);
        assert_eq!(failed_status.recovery_state, LoaderRecoveryState::RecoverableFailure);
        assert!(failed_status.dirty_modules.iter().any(|module| module.state == DirtyModuleState::Modified));

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(
            root_dir.join("Root.cp"),
            concat!(
                "MODULE Root;\n",
                "IMPORT Console;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Console.WriteInt(3);\n",
                "  Console.WriteLn()\n",
                "END Run;\n",
                "END Root."
            ),
        )
        .expect("rewrite Root.cp valid again");

        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("reload with valid source should succeed");

        let recovered_status = session.status();
        assert!(session.last_failure().is_none());
        assert_eq!(recovered_status.invalidation_state, LoaderInvalidationState::Clean);
        assert_eq!(recovered_status.recovery_state, LoaderRecoveryState::Ready);
        assert!(recovered_status.dirty_modules.iter().all(|module| module.state == DirtyModuleState::Clean));

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn semantic_reload_failure_is_classified_separately() {
        let root_dir = std::env::temp_dir().join("newcp-loader-sema-failure");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create sema failure temp dir");
        std::fs::write(
            root_dir.join("Root.cp"),
            concat!(
                "MODULE Root;\n",
                "IMPORT Console;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  Console.WriteLn()\n",
                "END Run;\n",
                "END Root."
            ),
        )
        .expect("write valid Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("initial valid load");

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(
            root_dir.join("Root.cp"),
            concat!(
                "MODULE Root;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  MissingProc()\n",
                "END Run;\n",
                "END Root."
            ),
        )
        .expect("rewrite Root.cp with semantic error");

        let error = session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect_err("reload with semantic error should fail");

        assert!(!error.is_empty());
        assert_eq!(
            session.last_failure().map(|failure| failure.phase),
            Some(LoaderFailurePhase::AnalyzeModule)
        );

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn active_execution_scope_blocks_quiescent_point() {
        let root_dir = std::env::temp_dir().join("newcp-loader-quiescent-scope");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create quiescent scope temp dir");
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; END Leaf.")
            .expect("write Leaf.cp");
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; END Root.")
            .expect("write Root.cp");

        let mut session = LoaderSession::new();
        let scope_id = session
            .begin_execution_scope(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("begin execution scope");

        assert!(!session.can_observe_quiescent_point());
        assert!(session.note_quiescent_point().is_err());
        assert!(session.end_execution_scope(scope_id));
        assert!(session.can_observe_quiescent_point());

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn pinned_retired_generation_survives_until_scope_ends() {
        let root_dir = std::env::temp_dir().join("newcp-loader-pinned-retired");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create pinned retired temp dir");
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; END Leaf.")
            .expect("write Leaf.cp");
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; END Root.")
            .expect("write Root.cp");

        let mut session = LoaderSession::new();
        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("initial graph load");
        let scope_id = session
            .begin_execution_scope(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("begin execution scope");

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; CONST X* = 1; END Root.")
            .expect("rewrite Root.cp");
        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("reload graph after edit");

        assert_eq!(session.retired_materializations().len(), 1);
        assert!(session.end_execution_scope(scope_id));
        session.note_quiescent_point().expect("quiescent point after scope end");
        let collected = session.garbage_collect_retired();

        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].name, "Root");

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn invoke_command_drives_quiescent_collection() {
        // After a recompile-and-invoke cycle, retired generations must be
        // reclaimed automatically — without an explicit note_quiescent_point /
        // garbage_collect_retired call from the client.
        //
        // The Run body is intentionally empty: this test runs in parallel
        // with Console-capturing tests, and any WriteLn output emitted here
        // would land in their capture buffers.
        let root_dir = std::env::temp_dir().join("newcp-loader-invoke-drives-gc");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create gc-drive temp dir");
        std::fs::write(
            root_dir.join("Root.cp"),
            "MODULE Root; PROCEDURE Run*; BEGIN END Run; END Root.",
        )
        .expect("write Root.cp");

        let mut session = LoaderSession::new();
        let command_path = format!("{}::Run", root_dir.join("Root.cp").display());

        // First invoke materializes the module.
        session.invoke_command(&command_path).expect("first invoke");
        assert_eq!(
            session.retired_materializations().len(),
            0,
            "no retirement on first compile"
        );

        // Edit + reinvoke. The previous generation gets retired, then the
        // auto-drive at the end of invoke_command collects it.
        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(
            root_dir.join("Root.cp"),
            "MODULE Root; CONST Tag* = 1; PROCEDURE Run*; BEGIN END Run; END Root.",
        )
        .expect("rewrite Root.cp");

        session.invoke_command(&command_path).expect("second invoke");

        assert_eq!(
            session.retired_materializations().len(),
            0,
            "auto-drive should reclaim retired metadata after invoke_command"
        );
        assert_eq!(
            session.retired_executable_image_count(),
            0,
            "auto-drive should reclaim retired JIT images after invoke_command"
        );
        newcp_runtime::console::reset();

        let _ = std::fs::remove_dir_all(&root_dir);
    }

    #[test]
    fn retired_image_drop_predicate_can_veto_collection() {
        // A drop predicate returning false must pin retired images and their
        // matching metadata across collection cycles, even when stack-quiescent.
        let root_dir = std::env::temp_dir().join("newcp-loader-drop-veto");
        let _ = std::fs::remove_dir_all(&root_dir);
        std::fs::create_dir_all(&root_dir).expect("create drop-veto temp dir");
        std::fs::write(root_dir.join("Leaf.cp"), "MODULE Leaf; END Leaf.")
            .expect("write Leaf.cp");
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; END Root.")
            .expect("write Root.cp");

        let mut session = LoaderSession::new();
        // Block every drop. Realistic probes would scan the heap; here we
        // just verify the veto path.
        session.set_retired_image_drop_predicate(RetiredImageDropPredicate::new(
            |_name, _gen| false,
        ));

        session
            .ensure_module_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("initial load");

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(root_dir.join("Root.cp"), "MODULE Root; IMPORT Leaf; CONST X* = 1; END Root.")
            .expect("rewrite Root.cp");
        session
            .ensure_import_graph_loaded(root_dir.join("Root.cp").to_str().expect("utf-8 temp path"))
            .expect("refresh load");

        assert_eq!(session.retired_materializations().len(), 1);
        session.note_quiescent_point().expect("quiescent");
        let collected = session.garbage_collect_retired();

        // Veto: nothing collected, both metadata and image still retained.
        assert!(collected.is_empty(), "predicate veto should suppress collection");
        assert_eq!(
            session.retired_materializations().len(),
            1,
            "metadata must follow image fate when probe vetoes"
        );

        // Lift the veto and re-run: the now-eligible records get reclaimed.
        session.clear_retired_image_drop_predicate();
        let collected = session.garbage_collect_retired();
        assert_eq!(collected.len(), 1);
        assert!(session.retired_materializations().is_empty());

        let _ = std::fs::remove_dir_all(&root_dir);
    }
}
