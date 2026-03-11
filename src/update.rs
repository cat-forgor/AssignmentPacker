use crate::error::{Error, Result};
use crate::ui;
use owo_colors::OwoColorize;
use std::path::Path;

const CURRENT: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = "cat-forgor/AssignmentPacker";

pub fn run() -> Result<()> {
    eprintln!("Checking for updates...");

    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let tag = fetch_latest_tag(&url)?;
    let latest = tag.strip_prefix('v').unwrap_or(&tag);

    if !version_newer(latest, CURRENT) {
        eprintln!("  Already up to date ({})", CURRENT.bold());
        return Ok(());
    }

    eprintln!(
        "\n  {} {} -> {}",
        "Update available:".green().bold(),
        CURRENT.dimmed(),
        latest.bold()
    );

    let current_exe = std::env::current_exe()
        .map_err(|e| Error::Validation(format!("can't find current binary path: {e}")))?;

    let method = detect_install_method(&current_exe);

    if let Err(e) = do_self_update(&current_exe, &tag, latest) {
        if method == InstallMethod::SelfUpdate {
            return Err(e);
        }
        eprintln!("  Self-update failed ({}), try your package manager instead:\n", e);
        print_update_command(&method, &tag);
    }

    Ok(())
}

fn do_self_update(current_exe: &Path, tag: &str, latest: &str) -> Result<()> {
    let asset = platform_asset().ok_or_else(|| {
        Error::Validation(format!(
            "no pre-built binary for this platform, update manually:\n  \
             cargo install assignment_packer\n  \
             https://github.com/{REPO}/releases/tag/{tag}"
        ))
    })?;

    ui::step(&format!("Downloading {asset}..."));
    let download_url = format!("https://github.com/{REPO}/releases/download/{tag}/{asset}");
    let bytes = download_binary(&download_url)?;

    ui::step("Replacing binary...");
    replace_binary(current_exe, &bytes)?;

    ui::done(&format!("Updated to {latest}"));
    Ok(())
}

fn print_update_command(method: &InstallMethod, tag: &str) {
    eprintln!("  https://github.com/{REPO}/releases/tag/{tag}\n");
    eprintln!("  Update with:");
    match method {
        InstallMethod::Cargo => eprintln!("    cargo install assignment_packer"),
        InstallMethod::Scoop => eprintln!("    scoop update ap"),
        InstallMethod::Chocolatey => eprintln!("    choco upgrade ap"),
        InstallMethod::WinGet => eprintln!("    winget upgrade cat-forgor.ap"),
        InstallMethod::Msi => eprintln!("    winget upgrade cat-forgor.ap"),
        InstallMethod::Homebrew => eprintln!("    brew upgrade ap"),
        InstallMethod::Nix => eprintln!("    nix profile upgrade github:{REPO}"),
        InstallMethod::SystemPackage => {
            eprintln!("    yay -Syu ap-bin          # AUR");
            eprintln!("    sudo dpkg -i ap_*.deb    # Debian/Ubuntu");
        }
        InstallMethod::SelfUpdate => unreachable!(),
    }
}

#[derive(Debug, PartialEq)]
enum InstallMethod {
    Cargo,
    Scoop,
    Chocolatey,
    WinGet,
    Msi,
    Homebrew,
    Nix,
    SystemPackage,
    SelfUpdate,
}

fn detect_install_method(exe_path: &Path) -> InstallMethod {
    let path_str = exe_path.to_string_lossy();

    let p = path_str.replace('\\', "/").to_lowercase();

    if p.contains("/.cargo/bin/") {
        return InstallMethod::Cargo;
    }
    if p.contains("/scoop/") {
        return InstallMethod::Scoop;
    }
    if p.contains("/chocolatey/") || p.contains("/programdata/chocolatey") {
        return InstallMethod::Chocolatey;
    }
    if p.contains("/nix/store/") {
        return InstallMethod::Nix;
    }
    if p.contains("/cellar/") || p.contains("/homebrew/") || p.contains("/linuxbrew/") {
        return InstallMethod::Homebrew;
    }
    if p.contains("/program files/") || p.contains("/program files (x86)/") {
        if p.contains("/windowsapps/") {
            return InstallMethod::WinGet;
        }
        return InstallMethod::Msi;
    }
    if p.starts_with("/usr/bin/") || p.starts_with("/usr/local/bin/") {
        return InstallMethod::SystemPackage;
    }

    InstallMethod::SelfUpdate
}

fn platform_asset() -> Option<&'static str> {
    if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        Some("ap-windows-x64.exe")
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        Some("ap-linux-x64")
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        Some("ap-macos-arm64")
    } else {
        None
    }
}

fn download_binary(url: &str) -> Result<Vec<u8>> {
    let buf = ureq::get(url)
        .header("User-Agent", "ap-updater")
        .call()
        .map_err(|e| Error::Validation(format!("download failed: {e}")))?
        .body_mut()
        .read_to_vec()
        .map_err(|e| Error::Validation(format!("failed to read download: {e}")))?;

    if buf.len() < 1024 {
        return Err(Error::Validation(
            "downloaded file is suspiciously small, aborting".into(),
        ));
    }

    Ok(buf)
}

fn replace_binary(current: &Path, new_bytes: &[u8]) -> Result<()> {
    use std::fs;

    if cfg!(windows) {
        let old = current.with_extension("old.exe");
        let _ = fs::remove_file(&old);
        fs::rename(current, &old)
            .map_err(|e| Error::Validation(format!("can't rename current binary: {e}")))?;
        if let Err(e) = fs::write(current, new_bytes) {
            let _ = fs::rename(&old, current);
            return Err(Error::Validation(format!("can't write new binary: {e}")));
        }
        let _ = fs::remove_file(&old);
    } else {
        let tmp = current.with_extension("tmp");
        fs::write(&tmp, new_bytes)
            .map_err(|e| Error::Validation(format!("can't write temp file: {e}")))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755))
                .map_err(|e| Error::Validation(format!("can't set permissions: {e}")))?;
        }

        fs::rename(&tmp, current)
            .map_err(|e| Error::Validation(format!("can't replace binary: {e}")))?;
    }

    Ok(())
}

fn fetch_latest_tag(url: &str) -> Result<String> {
    let body: String = ureq::get(url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "ap-update-checker")
        .call()
        .map_err(|e| Error::Validation(format!("failed to check for updates: {e}")))?
        .body_mut()
        .read_to_string()
        .map_err(|e| Error::Validation(format!("failed to read response: {e}")))?;

    let needle = "\"tag_name\":";
    let pos = body
        .find(needle)
        .ok_or_else(|| Error::Validation("unexpected response from GitHub API".into()))?;
    let rest = &body[pos + needle.len()..];
    let rest = rest.trim_start();
    if !rest.starts_with('"') {
        return Err(Error::Validation(
            "unexpected response from GitHub API".into(),
        ));
    }
    let rest = &rest[1..];
    let end = rest
        .find('"')
        .ok_or_else(|| Error::Validation("unexpected response from GitHub API".into()))?;
    Ok(rest[..end].to_string())
}

fn version_newer(latest: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        let mut parts: Vec<u64> = s
            .split('.')
            .filter_map(|part| part.parse::<u64>().ok())
            .collect();
        if parts.len() < 3 {
            parts.resize(3, 0);
        }
        parts
    };
    let l = parse(latest);
    let c = parse(current);
    l > c
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn newer_version() {
        assert!(version_newer("0.2.0", "0.1.0"));
        assert!(version_newer("1.0.0", "0.9.9"));
        assert!(version_newer("0.1.1", "0.1.0"));
    }

    #[test]
    fn same_version() {
        assert!(!version_newer("0.1.0", "0.1.0"));
    }

    #[test]
    fn older_version() {
        assert!(!version_newer("0.1.0", "0.2.0"));
    }

    #[test]
    fn short_versions_are_padded() {
        assert!(!version_newer("1.0", "1.0.0"));
        assert!(version_newer("1.0.1", "1.0"));
    }

    #[test]
    fn platform_asset_returns_something() {
        let asset = platform_asset();
        if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
            assert_eq!(asset, Some("ap-windows-x64.exe"));
        } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
            assert_eq!(asset, Some("ap-linux-x64"));
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            assert_eq!(asset, Some("ap-macos-arm64"));
        }
    }

    #[test]
    fn detect_cargo() {
        let p = PathBuf::from("/home/user/.cargo/bin/ap");
        assert_eq!(detect_install_method(&p), InstallMethod::Cargo);
    }

    #[test]
    fn detect_scoop() {
        let p = PathBuf::from(r"C:\Users\me\scoop\apps\ap\current\ap.exe");
        assert_eq!(detect_install_method(&p), InstallMethod::Scoop);
    }

    #[test]
    fn detect_chocolatey() {
        let p = PathBuf::from(r"C:\ProgramData\chocolatey\bin\ap.exe");
        assert_eq!(detect_install_method(&p), InstallMethod::Chocolatey);
    }

    #[test]
    fn detect_nix() {
        let p = PathBuf::from("/nix/store/abc123-ap-0.1.0/bin/ap");
        assert_eq!(detect_install_method(&p), InstallMethod::Nix);
    }

    #[test]
    fn detect_homebrew() {
        let p = PathBuf::from("/opt/homebrew/Cellar/ap/0.1.0/bin/ap");
        assert_eq!(detect_install_method(&p), InstallMethod::Homebrew);
    }

    #[test]
    fn detect_msi() {
        let p = PathBuf::from(r"C:\Program Files\ap\ap.exe");
        assert_eq!(detect_install_method(&p), InstallMethod::Msi);
    }

    #[test]
    fn detect_system_package() {
        let p = PathBuf::from("/usr/bin/ap");
        assert_eq!(detect_install_method(&p), InstallMethod::SystemPackage);
    }

    #[test]
    fn detect_standalone() {
        let p = PathBuf::from("/home/user/.local/bin/ap");
        assert_eq!(detect_install_method(&p), InstallMethod::SelfUpdate);

        let p = PathBuf::from(r"C:\Users\me\Desktop\ap.exe");
        assert_eq!(detect_install_method(&p), InstallMethod::SelfUpdate);
    }
}
