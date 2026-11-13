use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn package_lightweight_pcpl(plugin_dir: &Path, output_path: &Path) -> Result<()> {
    use std::io::{Read, Write};
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    let file = fs::File::create(output_path)
        .with_context(|| format!("Failed to create {}", output_path.display()))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    fn add_dir(
        zip: &mut ZipWriter<fs::File>,
        dir: &Path,
        prefix: &Path,
        options: SimpleFileOptions,
    ) -> Result<()> {
        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory {}", dir.display()))?;

        for entry in entries.flatten() {
            let path = entry.path();
            let relative = path
                .strip_prefix(prefix)
                .with_context(|| format!("Failed to strip prefix from {}", path.display()))?;
            let name_str = relative.to_string_lossy().replace('\\', "/");

            if path.is_dir() {
                // Skip packaging models directory if it only contains .onnx
                zip.add_directory(&name_str, options)?;
                add_dir(zip, &path, prefix, options)?;
            } else {
                // Strictly exclude heavy model weight files (*.onnx, *.bin, *.safetensors)
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_ascii_lowercase();
                    if ext_lower == "onnx"
                        || ext_lower == "bin"
                        || ext_lower == "safetensors"
                        || ext_lower == "part"
                    {
                        continue;
                    }
                }
                zip.start_file(&name_str, options)?;
                let mut f = fs::File::open(&path)
                    .with_context(|| format!("Failed to open {}", path.display()))?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
            }
        }
        Ok(())
    }

    add_dir(&mut zip, plugin_dir, plugin_dir, options)?;
    zip.finish()?;
    Ok(())
}
