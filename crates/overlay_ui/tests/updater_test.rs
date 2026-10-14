use overlay_ui::updater::{
    categorize_asset, generate_update_script, is_asset_matching_os, is_version_newer,
    prepare_portable_update, ReleaseAsset, UpdateCheckResult, UpdateState,
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

#[test]
fn test_portable_update_script_content() {
    let dummy_pid = 99999;
    let dummy_src = PathBuf::from("C:/test_temp/src");
    let dummy_dest = PathBuf::from("C:/test_dest");
    let dummy_exe = PathBuf::from("C:/test_dest/companion.exe");

    let (script_path, content) =
        generate_update_script(dummy_pid, &dummy_src, &dummy_dest, &dummy_exe);

    // Negative assertions: deprecated batch commands must not be used
    assert!(
        !content.contains("xcopy"),
        "Update script must not use xcopy"
    );
    assert!(
        !content.contains("timeout /t"),
        "Update script must not use timeout /t"
    );

    // Positive assertions: wait logic and cleanup logic
    if cfg!(target_os = "windows") {
        assert!(
            content.contains("Wait-Process"),
            "Windows update script must use Wait-Process"
        );
        assert!(
            content.contains("Remove-Item"),
            "Windows update script must clean up with Remove-Item"
        );
        assert_eq!(
            script_path.extension().and_then(|s| s.to_str()),
            Some("ps1"),
            "Windows update script must have .ps1 extension"
        );
    } else {
        assert!(
            content.contains("kill -0"),
            "Unix update script must check process liveness with kill -0"
        );
        assert!(
            content.contains("rm -rf"),
            "Unix update script must clean up with rm -rf"
        );
        assert_eq!(
            script_path.extension().and_then(|s| s.to_str()),
            Some("sh"),
            "Unix update script must have .sh extension"
        );
    }

    // End-to-end test with prepare_portable_update and a minimal zip file
    let temp_zip_path =
        std::env::temp_dir().join(format!("test_update_archive_{}.zip", std::process::id()));
    {
        let file = std::fs::File::create(&temp_zip_path).expect("Create test zip file");
        let mut zip_writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip_writer
            .start_file("dummy.txt", options)
            .expect("start_file");
        use std::io::Write;
        zip_writer
            .write_all(b"test file content")
            .expect("write content");
        zip_writer.finish().expect("finish zip");
    }

    let prep_res = prepare_portable_update(&temp_zip_path);
    assert!(prep_res.is_ok(), "prepare_portable_update should succeed");
    let generated_path = prep_res.unwrap();
    assert!(generated_path.exists(), "Generated script file must exist");

    let file_content =
        std::fs::read_to_string(&generated_path).expect("Read generated script file");
    assert!(
        !file_content.contains("xcopy"),
        "Generated file must not contain xcopy"
    );
    assert!(
        !file_content.contains("timeout /t"),
        "Generated file must not contain timeout /t"
    );
    if cfg!(target_os = "windows") {
        assert!(file_content.contains("Wait-Process"));
        assert!(file_content.contains("Remove-Item"));
    } else {
        assert!(file_content.contains("kill -0"));
        assert!(file_content.contains("rm -rf"));
    }

    let _ = std::fs::remove_file(&temp_zip_path);
    let _ = std::fs::remove_file(&generated_path);
}
