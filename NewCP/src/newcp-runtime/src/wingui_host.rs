/// wingui_host.rs
///
/// Safe Rust wrappers around wingui's spec_bind APIs.
///
/// Primary entry point: `SpecBindRuntime` - owns `WinguiSpecBindRuntime*`.
/// JIT-visible shims ("HostWindows.*", "WinSpec.*") are registered via
/// `native_module_artifact()` / `winspec_module_artifact()`.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::{CStr, CString, c_void};
use std::sync::{Condvar, Mutex, OnceLock};

use crate::wingui_ffi::SuperTerminalRunResult;
use crate::wingui_spec_ffi::{
    WinguiSpecBindEventView, WinguiSpecBindRunDesc, WinguiSpecBindRuntime,
};
use crate::{
    ExportDirectory, ExportEntry, HostedModuleArtifact, NativeExportBinding, NativeModuleArtifact,
};

// ---------------------------------------------------------------------------
// Event queue
// ---------------------------------------------------------------------------

struct GuiEvent {
    name:    String,
    payload: String,
}

struct EventQueue {
    queue: Mutex<VecDeque<GuiEvent>>,
    ready: Condvar,
}

static EVENT_QUEUE: EventQueue = EventQueue {
    queue: Mutex::new(VecDeque::new()),
    ready: Condvar::new(),
};

unsafe extern "system" fn on_event(
    _user_data: *mut c_void,
    _runtime: *mut WinguiSpecBindRuntime,
    view: *const WinguiSpecBindEventView,
) {
    if view.is_null() {
        eprintln!("[wingui_event] on_event called with null view");
        return;
    }
    let v = unsafe { &*view };
    let name = if v.event_name_utf8.is_null() { String::new() }
               else { unsafe { CStr::from_ptr(v.event_name_utf8) }.to_string_lossy().into_owned() };
    let payload = if v.payload_json_utf8.is_null() { String::new() }
                  else { unsafe { CStr::from_ptr(v.payload_json_utf8) }.to_string_lossy().into_owned() };
    let source = if v.source_utf8.is_null() { String::new() }
                 else { unsafe { CStr::from_ptr(v.source_utf8) }.to_string_lossy().into_owned() };
    eprintln!("[wingui_event] name={:?} source={:?} payload={:?}", name, source, payload);
    let mut q = EVENT_QUEUE.queue.lock().expect("event queue poisoned");
    q.push_back(GuiEvent { name, payload });
    EVENT_QUEUE.ready.notify_one();
}

unsafe extern "system" fn on_frame(
    _user_data: *mut c_void,
    _runtime: *mut WinguiSpecBindRuntime,
    _frame_view: *const crate::wingui_spec_ffi::WinguiSpecBindFrameView,
) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static LOGGED: AtomicBool = AtomicBool::new(false);
    if !LOGGED.swap(true, Ordering::Relaxed) {
        eprintln!("[wingui_frame] first frame callback fired");
    }
}

// ---------------------------------------------------------------------------
// Global runtime pointer
// ---------------------------------------------------------------------------

static RUNTIME: OnceLock<usize> = OnceLock::new();

fn runtime_ptr() -> *mut WinguiSpecBindRuntime {
    RUNTIME.get().copied().unwrap_or(0) as *mut WinguiSpecBindRuntime
}

// ---------------------------------------------------------------------------
// SpecBindRuntime RAII wrapper
// ---------------------------------------------------------------------------

pub struct SpecBindRuntime {
    ptr: *mut WinguiSpecBindRuntime,
}

unsafe impl Send for SpecBindRuntime {}

impl SpecBindRuntime {
    pub fn new() -> Option<Self> {
        let mut ptr: *mut WinguiSpecBindRuntime = std::ptr::null_mut();
        let ret = unsafe { crate::wingui_spec_ffi::wingui_spec_bind_runtime_create(&mut ptr) };
        eprintln!("[wingui_host] wingui_spec_bind_runtime_create: ret={} ptr={:?}", ret, ptr);
        if ret != 0 && !ptr.is_null() {
            // Load a minimal placeholder spec so that `run` can open the window
            // before the CP worker calls PublishUi with the real layout.
            let placeholder = c"{\"type\":\"window\",\"title\":\"NewCP\",\"children\":[{\"type\":\"textarea\",\"id\":\"log\",\"text\":\"Starting...\"}]}";
            unsafe {
                crate::wingui_spec_ffi::wingui_spec_bind_runtime_load_spec_json(
                    ptr, placeholder.as_ptr() as *const std::os::raw::c_char,
                );
                crate::wingui_spec_ffi::wingui_spec_bind_runtime_set_default_handler(
                    ptr, Some(on_event), std::ptr::null_mut(),
                );
                crate::wingui_spec_ffi::wingui_spec_bind_runtime_set_frame_handler(
                    ptr, Some(on_frame), std::ptr::null_mut(),
                );
            }
            eprintln!("[wingui_host] runtime created, handlers registered, placeholder spec loaded");
            Some(Self { ptr })
        } else {
            eprintln!("[wingui_host] wingui_spec_bind_runtime_create failed (ret={})", ret);
            None
        }
    }

    pub fn load_spec_json(&mut self, json: &str) -> bool {
        let c = CString::new(json).unwrap_or_default();
        let ret = unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_runtime_load_spec_json(self.ptr, c.as_ptr())
        };
        if ret == 0 { eprintln!("[wingui_host] load_spec_json failed"); }
        ret != 0
    }

    pub fn request_stop(&self, exit_code: i32) {
        eprintln!("[wingui_host] request_stop called with exit_code={}", exit_code);
        unsafe { crate::wingui_spec_ffi::wingui_spec_bind_runtime_request_stop(self.ptr, exit_code); }
    }

    pub fn as_ptr(&self) -> *mut WinguiSpecBindRuntime { self.ptr }

    pub fn run(&self, config: &HostConfig) -> i32 {
        let _ = RUNTIME.set(self.ptr as usize);
        let title = CString::new(config.title.as_str()).unwrap_or_default();
        let font = config.font_family.as_deref()
            .map(|s| CString::new(s).unwrap_or_default());
        let font_ptr = font.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
        // Resolve shader path: use config override, else auto-detect <exe_dir>/shaders.
        let shader_path_str = config.shader_path.clone().or_else(|| {
            std::env::current_exe().ok()
                .and_then(|p| p.parent().map(|d| d.join("shaders")))
                .map(|p| p.to_string_lossy().into_owned())
        });
        let shader_cstr = shader_path_str.as_deref()
            .map(|s| CString::new(s).unwrap_or_default());
        let shader_ptr = shader_cstr.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
        let shader_path_display = shader_path_str.as_deref().unwrap_or("<none>");
        eprintln!("[wingui_host] run desc: title={:?} cols={} rows={} font_px={} shader_path={:?}",
            config.title, config.columns, config.rows, config.font_pixel_height, shader_path_display);
        let desc = WinguiSpecBindRunDesc {
            title_utf8: title.as_ptr(),
            columns: config.columns,
            rows: config.rows,
            flags: 0,
            command_queue_capacity: config.command_queue_capacity,
            event_queue_capacity: config.event_queue_capacity,
            font_family_utf8: font_ptr,
            font_pixel_height: config.font_pixel_height,
            dpi_scale: config.dpi_scale,
            text_shader_path_utf8: shader_ptr,
            target_frame_ms: 16,
            auto_request_present: 1,
        };
        let mut result = SuperTerminalRunResult { exit_code: 0, host_error_code: 0, message_utf8: [0; 256] };
        eprintln!("[wingui_host] calling spec_bind_runtime_run...");
        let ret = unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_runtime_run(self.ptr, &desc, &mut result)
        };
        let msg = unsafe { CStr::from_ptr(result.message_utf8.as_ptr()) };
        eprintln!("[wingui_host] spec_bind_runtime_run returned: ret={} exit_code={} host_error={} msg={}",
            ret, result.exit_code, result.host_error_code, msg.to_string_lossy());
        result.exit_code
    }
}

impl Drop for SpecBindRuntime {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { crate::wingui_spec_ffi::wingui_spec_bind_runtime_destroy(self.ptr) };
        }
    }
}

// ---------------------------------------------------------------------------
// HostConfig
// ---------------------------------------------------------------------------

pub struct HostConfig {
    pub title: String,
    pub columns: u32,
    pub rows: u32,
    pub command_queue_capacity: u32,
    pub event_queue_capacity: u32,
    pub font_family: Option<String>,
    pub font_pixel_height: i32,
    pub dpi_scale: f32,
    /// Path to the directory containing wingui HLSL shader files.
    /// If `None`, auto-detected as `<exe_dir>/shaders`.
    pub shader_path: Option<String>,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            title: "NewCP".to_string(),
            columns: 120, rows: 40,
            command_queue_capacity: 256, event_queue_capacity: 256,
            font_family: None, font_pixel_height: 18, dpi_scale: 1.0,
            shader_path: None,
        }
    }
}

// ---------------------------------------------------------------------------
// HostWindows shims
// ---------------------------------------------------------------------------

#[unsafe(export_name = "HostWindows.RequestPresent")]
pub extern "C" fn host_request_present() {}

#[unsafe(export_name = "HostWindows.RequestClose")]
pub extern "C" fn host_request_close() {
    eprintln!("[wingui_host] HostWindows.RequestClose called");
    let r = runtime_ptr();
    if r.is_null() { eprintln!("[wingui_host] RequestClose: runtime ptr is null"); return; }
    unsafe { crate::wingui_spec_ffi::wingui_spec_bind_runtime_request_stop(r, 0) };
}

#[unsafe(export_name = "HostWindows.PublishUi")]
pub extern "C" fn host_publish_ui(json_ptr: *const u8) {
    let r = runtime_ptr();
    if r.is_null() {
        eprintln!("[wingui_host] PublishUi: runtime ptr is null — window not open yet");
        return;
    }
    if json_ptr.is_null() {
        eprintln!("[wingui_host] PublishUi: json_ptr is null");
        return;
    }
    let json_preview = unsafe {
        let s = CStr::from_ptr(json_ptr as *const std::os::raw::c_char).to_string_lossy();
        if s.len() > 200 { format!("{}...", &s[..200]) } else { s.into_owned() }
    };
    eprintln!("[wingui_host] PublishUi: json={}", json_preview);
    let ret = unsafe {
        crate::wingui_spec_ffi::wingui_spec_bind_runtime_load_spec_json(
            r, json_ptr as *const std::os::raw::c_char,
        )
    };
    eprintln!("[wingui_host] PublishUi: load_spec_json returned {}", ret);
}

#[unsafe(export_name = "HostWindows.PatchUi")]
pub extern "C" fn host_patch_ui(_json_ptr: *const u8) {}

/// WaitNamedEvent: blocks until an event is available or timeout elapses.
/// Returns 1 on event, 0 on timeout.
/// Signature matches CP: PROCEDURE (VAR name: ARRAY 256 OF SHORTCHAR;
///                                   VAR payload: ARRAY 4096 OF SHORTCHAR;
///                                   timeout: INTEGER): INTEGER
/// Fixed-size VAR arrays are passed as bare pointers (no length word).
#[unsafe(export_name = "HostWindows.WaitNamedEvent")]
pub extern "C" fn host_wait_named_event(
    name_ptr:    *mut u8,
    payload_ptr: *mut u8,
    timeout_ms:  i32,
) -> i32 {
    const NAME_CAP: usize    = 256;
    const PAYLOAD_CAP: usize = 4096;
    if name_ptr.is_null() { return 0; }
    let event = {
        let mut q = EVENT_QUEUE.queue.lock().expect("event queue poisoned");
        if timeout_ms < 0 {
            loop {
                if let Some(ev) = q.pop_front() { break ev; }
                q = EVENT_QUEUE.ready.wait(q).expect("condvar wait failed");
            }
        } else {
            let dur = std::time::Duration::from_millis(timeout_ms as u64);
            let (mut q2, _) = EVENT_QUEUE.ready.wait_timeout(q, dur).expect("condvar timeout failed");
            match q2.pop_front() {
                Some(ev) => ev,
                None => return 0,
            }
        }
    };
    let name_bytes = event.name.as_bytes();
    let name_copy = (NAME_CAP - 1).min(name_bytes.len());
    unsafe {
        std::ptr::copy_nonoverlapping(name_bytes.as_ptr(), name_ptr, name_copy);
        *name_ptr.add(name_copy) = 0;
    }
    if !payload_ptr.is_null() {
        let pay_bytes = event.payload.as_bytes();
        let pay_copy = (PAYLOAD_CAP - 1).min(pay_bytes.len());
        unsafe {
            std::ptr::copy_nonoverlapping(pay_bytes.as_ptr(), payload_ptr, pay_copy);
            *payload_ptr.add(pay_copy) = 0;
        }
    }
    1
}

// ---------------------------------------------------------------------------
// WinSpec builder shims
// ---------------------------------------------------------------------------

struct FrameEntry {
    type_:    &'static str,
    gap:      Option<i32>,
    children: Vec<String>,
}

struct WinSpecBuilder {
    title: String,
    stack: Vec<FrameEntry>,
}

impl WinSpecBuilder {
    fn new() -> Self { Self { title: String::new(), stack: vec![] } }

    fn reset(&mut self, title: &str) {
        self.title = title.to_owned();
        self.stack = vec![FrameEntry { type_: "stack", gap: None, children: vec![] }];
    }

    fn push_leaf(&mut self, json: String) {
        if let Some(frame) = self.stack.last_mut() { frame.children.push(json); }
    }

    fn open(&mut self, type_: &'static str, gap: Option<i32>) {
        self.stack.push(FrameEntry { type_, gap, children: vec![] });
    }

    fn close(&mut self) {
        if self.stack.len() <= 1 { return; }
        let frame = self.stack.pop().unwrap();
        let gap_part = frame.gap.map(|g| format!(",\"gap\":{}", g)).unwrap_or_default();
        let node = format!(
            "{{\"type\":\"{}\"{},\"children\":[{}]}}",
            frame.type_, gap_part, frame.children.join(",")
        );
        self.push_leaf(node);
    }

    fn build(&self) -> String {
        let body_children = self.stack.last()
            .map(|f| f.children.join(",")).unwrap_or_default();
        let title_esc = self.title.replace('"', "\\\"");
        format!(
            "{{\"type\":\"window\",\"title\":\"{}\",\"body\":{{\"type\":\"stack\",\"children\":[{}]}}}}",
            title_esc, body_children
        )
    }
}

thread_local! {
    static SPEC: RefCell<WinSpecBuilder> = RefCell::new(WinSpecBuilder::new());
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
     .replace('\n', "\\n").replace('\r', "\\r")
}

#[unsafe(export_name = "WinSpec.Begin")]
pub extern "C" fn winspec_begin(title_ptr: *const u8) {
    if title_ptr.is_null() { return; }
    let title = unsafe { CStr::from_ptr(title_ptr as *const _) }.to_string_lossy();
    eprintln!("[WinSpec.Begin] title={:?}", title);
    SPEC.with(|s| s.borrow_mut().reset(&title));
}

#[unsafe(export_name = "WinSpec.OpenStack")]
pub extern "C" fn winspec_open_stack(gap: i32) {
    SPEC.with(|s| s.borrow_mut().open("stack", if gap < 0 { None } else { Some(gap) }));
}

#[unsafe(export_name = "WinSpec.OpenRow")]
pub extern "C" fn winspec_open_row(gap: i32) {
    SPEC.with(|s| s.borrow_mut().open("row", if gap < 0 { None } else { Some(gap) }));
}

#[unsafe(export_name = "WinSpec.CloseContainer")]
pub extern "C" fn winspec_close_container() {
    SPEC.with(|s| s.borrow_mut().close());
}

#[unsafe(export_name = "WinSpec.AddTextarea")]
pub extern "C" fn winspec_add_textarea(
    id_ptr: *const u8, label_ptr: *const u8, value_ptr: *const u8, readonly: i32,
) {
    let id    = unsafe { CStr::from_ptr(id_ptr    as *const _) }.to_string_lossy();
    let label = unsafe { CStr::from_ptr(label_ptr as *const _) }.to_string_lossy();
    let value = escape_json(&unsafe { CStr::from_ptr(value_ptr as *const _) }.to_string_lossy());
    let ro = if readonly != 0 { ",\"readonly\":true" } else { "" };
    SPEC.with(|s| s.borrow_mut().push_leaf(format!(
        "{{\"type\":\"textarea\",\"id\":\"{}\",\"label\":\"{}\",\"value\":\"{}\"{}}}",
        id, label, value, ro
    )));
}

#[unsafe(export_name = "WinSpec.AddButton")]
pub extern "C" fn winspec_add_button(id_ptr: *const u8, label_ptr: *const u8, event_ptr: *const u8) {
    let id    = unsafe { CStr::from_ptr(id_ptr    as *const _) }.to_string_lossy();
    let label = unsafe { CStr::from_ptr(label_ptr as *const _) }.to_string_lossy();
    let ev    = unsafe { CStr::from_ptr(event_ptr as *const _) }.to_string_lossy();
    SPEC.with(|s| s.borrow_mut().push_leaf(format!(
        "{{\"type\":\"button\",\"id\":\"{}\",\"text\":\"{}\",\"event\":\"{}\"}}",
        id, label, ev
    )));
}

#[unsafe(export_name = "WinSpec.AddText")]
pub extern "C" fn winspec_add_text(text_ptr: *const u8) {
    let text = escape_json(&unsafe { CStr::from_ptr(text_ptr as *const _) }.to_string_lossy());
    SPEC.with(|s| s.borrow_mut().push_leaf(format!("{{\"type\":\"text\",\"text\":\"{}\"}}", text)));
}

#[unsafe(export_name = "WinSpec.GetSpec")]
pub extern "C" fn winspec_get_spec(buf_ptr: *mut u8) -> i32 {
    // VAR spec: ARRAY 10240 OF SHORTCHAR — fixed array, passed as bare pointer.
    const BUF_CAP: usize = 10240;
    if buf_ptr.is_null() { return 0; }
    let json = SPEC.with(|s| s.borrow().build());
    let bytes = json.as_bytes();
    eprintln!("[WinSpec.GetSpec] json_len={} cap={} json={}", bytes.len(), BUF_CAP, &json);
    if bytes.len() + 1 > BUF_CAP { return 0; }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf_ptr, bytes.len());
        *buf_ptr.add(bytes.len()) = 0;
    }
    1
}

/// Update a textarea's value in-place inside an existing spec JSON string.
/// Avoids a full spec rebuild when only output text changes.
#[unsafe(export_name = "WinSpec.UpdateTextarea")]
pub extern "C" fn winspec_update_textarea(
    spec_ptr: *mut u8, spec_len: u32, id_ptr: *const u8, value_ptr: *const u8,
) -> i32 {
    if spec_ptr.is_null() || id_ptr.is_null() || value_ptr.is_null() { return 0; }
    let spec  = unsafe { CStr::from_ptr(spec_ptr  as *const _) }.to_string_lossy().into_owned();
    let id    = unsafe { CStr::from_ptr(id_ptr    as *const _) }.to_string_lossy().into_owned();
    let value = escape_json(&unsafe { CStr::from_ptr(value_ptr as *const _) }.to_string_lossy());
    let result = patch_textarea_value(&spec, &id, &value);
    let bytes = result.as_bytes();
    if bytes.len() + 1 > spec_len as usize { return 0; }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), spec_ptr, bytes.len());
        *spec_ptr.add(bytes.len()) = 0;
    }
    1
}

fn patch_textarea_value(spec: &str, id: &str, new_val_esc: &str) -> String {
    let id_marker = format!("\"id\":\"{}\"", id);
    let obj_start = match spec.find(&id_marker).and_then(|p| spec[..p].rfind('{')) {
        Some(pos) => pos,
        None => return spec.to_owned(),
    };
    let after = &spec[obj_start..];
    if let Some(vstart) = after.find("\"value\":\"") {
        let content_start = obj_start + vstart + 9;
        if let Some(end) = find_end_quote(&spec[content_start..]) {
            let mut out = spec.to_owned();
            out.replace_range(content_start..content_start + end, new_val_esc);
            return out;
        }
    }
    spec.to_owned()
}

fn find_end_quote(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' { i += 2; continue; }
        if b[i] == b'"'  { return Some(i); }
        i += 1;
    }
    None
}

// ---------------------------------------------------------------------------
// Module artifacts
// ---------------------------------------------------------------------------

pub fn native_module_artifact() -> NativeModuleArtifact {
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "HostWindows", vec![],
            ExportDirectory::new(vec![
                ExportEntry::procedure("RequestPresent"),
                ExportEntry::procedure("RequestClose"),
                ExportEntry::procedure("PublishUi"),
                ExportEntry::procedure("PatchUi"),
                ExportEntry::procedure("WaitNamedEvent"),
            ]),
            "HostWindows.bootstrap",
            "Rust-hosted wingui spec_bind bridge",
            vec![],
        ),
        vec![
            NativeExportBinding::procedure("RequestPresent", host_request_present as *const () as usize),
            NativeExportBinding::procedure("RequestClose",   host_request_close   as *const () as usize),
            NativeExportBinding::procedure("PublishUi",      host_publish_ui      as *const () as usize),
            NativeExportBinding::procedure("PatchUi",        host_patch_ui        as *const () as usize),
            NativeExportBinding::procedure("WaitNamedEvent", host_wait_named_event as *const () as usize),
        ],
    )
}

pub fn winspec_module_artifact() -> NativeModuleArtifact {
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "WinSpec", vec![],
            ExportDirectory::new(vec![
                ExportEntry::procedure("Begin"),
                ExportEntry::procedure("OpenStack"),
                ExportEntry::procedure("OpenRow"),
                ExportEntry::procedure("CloseContainer"),
                ExportEntry::procedure("AddTextarea"),
                ExportEntry::procedure("AddButton"),
                ExportEntry::procedure("AddText"),
                ExportEntry::procedure("GetSpec"),
                ExportEntry::procedure("UpdateTextarea"),
            ]),
            "WinSpec.bootstrap",
            "Rust-hosted JSON spec builder for wingui layouts",
            vec![],
        ),
        vec![
            NativeExportBinding::procedure("Begin",          winspec_begin           as *const () as usize),
            NativeExportBinding::procedure("OpenStack",      winspec_open_stack      as *const () as usize),
            NativeExportBinding::procedure("OpenRow",        winspec_open_row        as *const () as usize),
            NativeExportBinding::procedure("CloseContainer", winspec_close_container as *const () as usize),
            NativeExportBinding::procedure("AddTextarea",    winspec_add_textarea    as *const () as usize),
            NativeExportBinding::procedure("AddButton",      winspec_add_button      as *const () as usize),
            NativeExportBinding::procedure("AddText",        winspec_add_text        as *const () as usize),
            NativeExportBinding::procedure("GetSpec",        winspec_get_spec        as *const () as usize),
            NativeExportBinding::procedure("UpdateTextarea", winspec_update_textarea as *const () as usize),
        ],
    )
}
