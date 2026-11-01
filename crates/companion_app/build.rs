fn main() {
    #[cfg(windows)]
    {
        if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
            let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let icon_path = manifest_dir.join("../../assets/icon.ico");
            if icon_path.exists() {
                let mut res = winres::WindowsResource::new();
                res.set_icon(icon_path.to_str().expect("valid utf-8 icon path"));
                res.set("FileDescription", "PhotoCraft Companion");
                res.set("ProductName", "PhotoCraft Companion");
                res.set("LegalCopyright", "PhotoCraft Community");
                if let Err(e) = res.compile() {
                    eprintln!("cargo:warning=Failed to compile Windows PE resource: {e}");
                }
            } else {
                eprintln!(
                    "cargo:warning=Icon file not found at: {}",
                    icon_path.display()
                );
            }
        }
    }
}
