use overlay_ui::updater::{
    categorize_asset, is_asset_matching_os, is_version_newer, ReleaseAsset, UpdateCheckResult,
    UpdateState,
};
use std::path::PathBuf;

#[test]
fn test_semver_comparisons() {
    // Normal newer versions
    assert!(is_version_newer("0.2.0", "0.1.0"));
    assert!(is_version_newer("1.0.0", "0.9.9"));
    assert!(is_version_newer("0.1.1", "0.1.0"));

    // Equal versions
    assert!(!is_version_newer("0.1.0", "0.1.0"));
    assert!(!is_version_newer("1.0.0", "1.0.0"));

    // Older versions
    assert!(!is_version_newer("0.1.0", "0.2.0"));
    assert!(!is_version_newer("0.9.9", "1.0.0"));

    // Prefixes 'v' and 'V'
    assert!(is_version_newer("v0.2.0", "0.1.0"));
    assert!(is_version_newer("0.2.0", "v0.1.0"));
    assert!(is_version_newer("V1.0.0", "v0.9.9"));
    assert!(!is_version_newer("v0.1.0", "v0.1.0"));

    // Suffixes and prereleases
    assert!(is_version_newer("1.0.0-rc.1", "0.9.0"));
    assert!(is_version_newer("0.2.0-beta", "0.1.9"));

    // Different number of parts
    assert!(is_version_newer("1.0.1", "1.0"));
    assert!(!is_version_newer("1.0", "1.0.1"));
    assert!(!is_version_newer("1.0", "1.0.0"));
}

#[test]
fn test_os_asset_matching() {
    // Windows assets
    assert!(is_asset_matching_os(
        "photocraft-companion-setup.exe",
        "windows",
        "x86_64"
    ));
    assert!(is_asset_matching_os(
        "photocraft-companion-installer.msi",
        "windows",
        "x86_64"
    ));
    assert!(is_asset_matching_os(
        "photocraft-companion-windows-x64.zip",
        "windows",
        "x86_64"
    ));
    assert!(!is_asset_matching_os(
        "photocraft-companion-linux.tar.gz",
        "windows",
        "x86_64"
    ));
    assert!(!is_asset_matching_os(
        "photocraft-companion.dmg",
        "windows",
        "x86_64"
    ));

    // Architecture filtering
    assert!(!is_asset_matching_os(
        "photocraft-companion-arm64.exe",
        "windows",
        "x86_64"
    ));
    assert!(is_asset_matching_os(
        "photocraft-companion-arm64.exe",
        "windows",
        "aarch64"
    ));

    // Linux assets
    assert!(is_asset_matching_os(
        "photocraft-companion.AppImage",
        "linux",
        "x86_64"
    ));
    assert!(is_asset_matching_os(
        "photocraft-companion_amd64.deb",
        "linux",
        "x86_64"
    ));
    assert!(is_asset_matching_os(
        "photocraft-companion-linux.tar.gz",
        "linux",
        "x86_64"
    ));
    assert!(!is_asset_matching_os(
        "photocraft-companion-setup.exe",
        "linux",
        "x86_64"
    ));

    // macOS assets
    assert!(is_asset_matching_os(
        "photocraft-companion.dmg",
        "macos",
        "x86_64"
    ));
    assert!(is_asset_matching_os(
        "photocraft-companion.pkg",
        "macos",
        "x86_64"
    ));
    assert!(is_asset_matching_os(
        "photocraft-companion-mac.zip",
        "macos",
        "x86_64"
    ));
    assert!(!is_asset_matching_os(
        "photocraft-companion-setup.exe",
        "macos",
        "x86_64"
    ));
}

#[test]
fn test_categorize_asset() {
    let (is_inst, is_port) = categorize_asset("photocraft-companion-setup.exe");
    assert!(is_inst);
    assert!(!is_port);

    let (is_inst, is_port) = categorize_asset("photocraft-companion-installer.msi");
    assert!(is_inst);
    assert!(!is_port);

    let (is_inst, is_port) = categorize_asset("photocraft-companion-portable.zip");
    assert!(!is_inst);
    assert!(is_port);

    let (is_inst, is_port) = categorize_asset("photocraft-companion.dmg");
    assert!(is_inst);
    assert!(!is_port);
}

#[test]
fn test_update_state_types() {
    let idle = UpdateState::Idle;
    assert_eq!(idle, UpdateState::Idle);

    let check_res = UpdateCheckResult {
        tag_name: "v1.0.0".to_string(),
        version: "1.0.0".to_string(),
        release_notes: "First release".to_string(),
        html_url: "https://github.com/kobaltgit/photocraft-companion/releases/v1.0.0".to_string(),
        published_at: "2026-10-13".to_string(),
        assets: vec![ReleaseAsset {
            name: "setup.exe".to_string(),
            download_url: "https://example.com/setup.exe".to_string(),
            size_bytes: 1024,
            is_installer: true,
            is_portable: false,
        }],
    };

    let avail = UpdateState::UpdateAvailable {
        result: check_res.clone(),
    };
    assert_eq!(avail, UpdateState::UpdateAvailable { result: check_res });

    let dl = UpdateState::Downloaded {
        target_path: PathBuf::from("C:/test/setup.exe"),
        is_portable: false,
    };
    assert_eq!(
        dl,
        UpdateState::Downloaded {
            target_path: PathBuf::from("C:/test/setup.exe"),
            is_portable: false,
        }
    );
}
