use std::env;
use std::path::PathBuf;

fn main() {
    // Only link wingui when the "gui" feature is enabled.
    // This keeps CLI-only builds (dump-asm, check-mod, etc.) free of the
    // D3D11/Win32 dependency.
    if env::var("CARGO_FEATURE_GUI").is_ok() {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        // CARGO_MANIFEST_DIR = …/NewCP/src/newcp-runtime
        // The wingui vcxproj outputs to …/multiwingui/manual_build/debug/
        // (older builds land in x64/Debug/ — we prefer manual_build/debug/).
        //
        // The path was historically `..\..\..\multiwingui` which assumes the
        // main checkout layout (multiwingui as sibling of NewCP). git worktrees
        // under `.claude/worktrees/<name>/NewCP/` break that assumption, so
        // walk upward until we find a `multiwingui` directory (or the env
        // override below). Falls back to the original three-up sibling guess
        // so existing checkouts keep building unchanged.
        let wingui_root = env::var("NEWCP_WINGUI_DIR")
            .ok()
            .map(PathBuf::from)
            .or_else(|| find_multiwingui_upward(&PathBuf::from(&manifest_dir)))
            .unwrap_or_else(|| {
                PathBuf::from(&manifest_dir)
                    .join("..")
                    .join("..")
                    .join("..")
                    .join("multiwingui")
            });

        let preferred = wingui_root.join("manual_build").join("debug");
        let fallback  = wingui_root.join("x64").join("Debug");
        let wingui_dir = if preferred.join("wingui.lib").exists() { preferred } else { fallback };

        println!("cargo:rustc-link-search=native={}", wingui_dir.display());
        // wingui.lib is an import library for wingui.dll — use dylib, not static
        println!("cargo:rustc-link-lib=dylib=wingui");

        // Required Windows system libraries for the wingui D3D11/Win32 backend.
        // These all ship with Windows so no extra files are needed.
        println!("cargo:rustc-link-lib=dylib=user32");
        println!("cargo:rustc-link-lib=dylib=gdi32");
        println!("cargo:rustc-link-lib=dylib=d3d11");
        println!("cargo:rustc-link-lib=dylib=d3dcompiler");
        println!("cargo:rustc-link-lib=dylib=dxgi");
        println!("cargo:rustc-link-lib=dylib=xaudio2");
        println!("cargo:rustc-link-lib=dylib=ole32");

        // Copy wingui.dll next to the built executable so it can be found at
        // runtime without needing to add multiwingui/x64/Debug to PATH.
        let dll_src = wingui_dir.join("wingui.dll");
        if dll_src.exists() {
            let out_dir = env::var("OUT_DIR").unwrap();
            // OUT_DIR = …/target/debug/build/newcp-runtime-<hash>/out
            // exe lives 3 levels up at …/target/debug/
            let exe_dir = PathBuf::from(&out_dir)
                .parent().unwrap() // newcp-runtime-<hash>
                .parent().unwrap() // build
                .parent().unwrap() // debug | release
                .to_path_buf();
            let dll_dst = exe_dir.join("wingui.dll");
            if let Err(e) = std::fs::copy(&dll_src, &dll_dst) {
                println!("cargo:warning=Could not copy wingui.dll to {}: {}", dll_dst.display(), e);
            }

            // Copy the shaders directory next to the exe so wingui can compile
            // its HLSL shaders at runtime via d3dcompiler.
            let shaders_src = wingui_root.join("shaders");
            let shaders_dst = exe_dir.join("shaders");
            if shaders_src.is_dir() {
                if let Err(e) = copy_dir_all(&shaders_src, &shaders_dst) {
                    println!("cargo:warning=Could not copy shaders to {}: {}", shaders_dst.display(), e);
                }
            } else {
                println!("cargo:warning=shaders directory not found at {}", shaders_src.display());
            }
            println!("cargo:rerun-if-changed={}", shaders_src.display());
        } else {
            println!("cargo:warning=wingui.dll not found at {} — build multiwingui first", dll_src.display());
        }

        println!("cargo:rerun-if-changed={}", dll_src.display());
    }
}

/// Walk up from `start` until a directory named `multiwingui` is found, or
/// six levels — enough to climb out of `.claude/worktrees/<name>/NewCP/src/
/// newcp-runtime` and still hit the project root that holds multiwingui as
/// a sibling.
fn find_multiwingui_upward(start: &std::path::Path) -> Option<PathBuf> {
    let mut cursor: Option<&std::path::Path> = Some(start);
    for _ in 0..8 {
        let dir = cursor?;
        let candidate = dir.join("multiwingui");
        if candidate.is_dir() {
            return Some(candidate);
        }
        cursor = dir.parent();
    }
    None
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}