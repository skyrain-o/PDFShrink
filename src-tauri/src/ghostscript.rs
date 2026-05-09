/// Returns the Tauri sidecar base name (without extension) for the GS binary.
/// Tauri appends platform-specific suffix at runtime when resolving externalBin.
pub fn sidecar_name() -> &'static str {
    "gs"
}

/// Returns the platform-specific binary filename, used by the fetch script
/// to know what to drop into `binaries/`.
pub fn target_triple_filename() -> String {
    let triple = current_target_triple();
    if cfg!(target_os = "windows") {
        format!("gs-{triple}.exe")
    } else {
        format!("gs-{triple}")
    }
}

fn current_target_triple() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "x86_64-pc-windows-msvc"
    } else {
        "unsupported"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidecar_name_is_gs() {
        assert_eq!(sidecar_name(), "gs");
    }

    #[test]
    fn filename_matches_current_platform() {
        let name = target_triple_filename();
        if cfg!(target_os = "windows") {
            assert!(name.ends_with(".exe"), "win sidecar must end with .exe, got {name}");
        } else {
            assert!(!name.ends_with(".exe"), "non-win sidecar must not end with .exe, got {name}");
        }
        assert!(name.starts_with("gs-"));
    }

    #[test]
    fn triple_is_known() {
        let t = current_target_triple();
        assert_ne!(t, "unsupported", "build target {t} is not in supported list");
    }
}
