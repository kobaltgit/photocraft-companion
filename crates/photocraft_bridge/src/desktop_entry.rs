use std::path::{Path, PathBuf};

/// Structure representing a parsed FreeDesktop Desktop Entry specification file (`.desktop`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DesktopEntry {
    /// Value of `Name` (application display name)
    pub name: Option<String>,
    /// Value of `Exec` (command line string with arguments and field codes)
    pub exec: Option<String>,
    /// Value of `Type` (e.g., `Application`, `Link`, `Directory`)
    pub entry_type: Option<String>,
    /// Value of `TryExec` (binary existence check hint)
    pub try_exec: Option<String>,
    /// Value of `Path` (working directory to run the application in)
    pub path: Option<PathBuf>,
}

/// Parse FreeDesktop .desktop file contents according to the Desktop Entry Specification.
/// Extracts fields strictly from the `[Desktop Entry]` section.
pub fn parse_desktop_entry(content: &str) -> Option<DesktopEntry> {
    let mut in_desktop_entry_section = false;
    let mut entry = DesktopEntry::default();
    let mut found_desktop_entry_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comment lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Section header
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let section_name = trimmed[1..trimmed.len() - 1].trim();
            if section_name == "Desktop Entry" {
                in_desktop_entry_section = true;
                found_desktop_entry_section = true;
            } else {
                in_desktop_entry_section = false;
            }
            continue;
        }

        if !in_desktop_entry_section {
            continue;
        }

        // Key-Value pair
        if let Some((key_raw, val_raw)) = trimmed.split_once('=') {
            let key = key_raw.trim();
            let val = val_raw.trim();

            match key {
                "Name" => {
                    if entry.name.is_none() {
                        entry.name = Some(val.to_string());
                    }
                }
                "Exec" => {
                    if entry.exec.is_none() {
                        entry.exec = Some(val.to_string());
                    }
                }
                "Type" => {
                    if entry.entry_type.is_none() {
                        entry.entry_type = Some(val.to_string());
                    }
                }
                "TryExec" => {
                    if entry.try_exec.is_none() {
                        entry.try_exec = Some(val.to_string());
                    }
                }
                "Path" => {
                    if entry.path.is_none() && !val.is_empty() {
                        entry.path = Some(PathBuf::from(val));
                    }
                }
                _ => {
                    // Fallback to localized Name if base Name not encountered yet
                    if key.starts_with("Name[") && entry.name.is_none() {
                        entry.name = Some(val.to_string());
                    }
                }
            }
        }
    }

    if found_desktop_entry_section
        && (entry.name.is_some() || entry.exec.is_some() || entry.entry_type.is_some())
    {
        Some(entry)
    } else {
        None
    }
}

/// Tokenize an `Exec` command line honoring single/double quotes and backslash escapes.
pub fn parse_exec_tokens(exec_line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut escaped = false;

    let chars: Vec<char> = exec_line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if escaped {
            current.push(c);
            escaped = false;
            i += 1;
            continue;
        }

        if c == '\\' && !in_single_quotes {
            if in_double_quotes {
                if i + 1 < chars.len() {
                    let next = chars[i + 1];
                    if next == '"' || next == '\\' || next == '$' || next == '`' {
                        current.push(next);
                        i += 2;
                        continue;
                    }
                }
                current.push('\\');
            } else {
                escaped = true;
            }
            i += 1;
            continue;
        }

        if c == '\'' && !in_double_quotes {
            in_single_quotes = !in_single_quotes;
            i += 1;
            continue;
        }

        if c == '"' && !in_single_quotes {
            in_double_quotes = !in_double_quotes;
            i += 1;
            continue;
        }

        if (c == ' ' || c == '\t') && !in_single_quotes && !in_double_quotes {
            if !current.is_empty() {
                tokens.push(current);
                current = String::new();
            }
            i += 1;
            continue;
        }

        current.push(c);
        i += 1;
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// Returns true if a token represents an XDG field code (%f, %F, %u, etc.) or Flatpak argument marker (@@u, @@).
fn is_field_code_or_marker(token: &str) -> bool {
    if token == "@@" || token == "@@u" || token == "@@U" || token == "@@F" || token == "@@f" {
        return true;
    }
    if token.starts_with('%') && token.len() == 2 {
        let code = token.chars().nth(1).unwrap();
        matches!(
            code,
            'f' | 'F' | 'u' | 'U' | 'd' | 'D' | 'n' | 'N' | 'i' | 'c' | 'k' | 'v' | '%'
        )
    } else {
        false
    }
}

/// Extract the primary executable candidate from an `Exec` line, stripping quotes, `env` wrappers,
/// and trailing XDG field codes or Flatpak markers.
pub fn extract_exec_binary(exec_line: &str) -> Option<PathBuf> {
    let tokens = parse_exec_tokens(exec_line);
    let mut iter = tokens.into_iter();

    let mut candidate = iter.next()?;

    // Skip `env` or `/usr/bin/env` wrappers and any FOO=BAR variable definitions
    if candidate == "env" || candidate == "/usr/bin/env" || candidate == "/bin/env" {
        let mut found_next = false;
        for next_tok in iter {
            if next_tok.contains('=')
                && !next_tok.starts_with('/')
                && !next_tok.starts_with('.')
                && !next_tok.starts_with('~')
            {
                continue;
            } else {
                candidate = next_tok;
                found_next = true;
                break;
            }
        }
        if !found_next {
            return None;
        }
    }

    if is_field_code_or_marker(&candidate) || candidate.is_empty() {
        return None;
    }

    Some(PathBuf::from(candidate))
}

/// Parse an `Exec` line and resolve the target executable:
/// - If absolute or relative with path separators, expands tilde and checks disk existence.
/// - If relative binary name, searches in system PATH.
pub fn parse_exec_command(exec_line: &str) -> Option<PathBuf> {
    let raw = extract_exec_binary(exec_line)?;

    // Check if it has path separators or is explicitly path-like
    let is_path_like = raw.is_absolute()
        || raw.components().count() > 1
        || raw.starts_with(".")
        || raw.starts_with("~");

    if is_path_like {
        let expanded = if raw.starts_with("~") {
            if let Ok(home) = std::env::var("HOME") {
                let mut p = PathBuf::from(home);
                if let Ok(stripped) = raw.strip_prefix("~") {
                    p.push(stripped);
                }
                p
            } else {
                raw
            }
        } else {
            raw
        };

        if expanded.is_file() {
            Some(expanded)
        } else {
            None
        }
    } else {
        // Simple binary name: search PATH
        if let Some(path_var) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let candidate = dir.join(&raw);
                if candidate.is_file() {
                    return Some(candidate);
                }
                #[cfg(windows)]
                {
                    if candidate.extension().is_none() {
                        let candidate_exe = candidate.with_extension("exe");
                        if candidate_exe.is_file() {
                            return Some(candidate_exe);
                        }
                    }
                }
            }
        }

        if raw.is_file() {
            return Some(raw);
        }

        None
    }
}

/// Parse a `.desktop` file from disk and resolve its executable path.
pub fn parse_desktop_file(path: &Path) -> Option<PathBuf> {
    let content = std::fs::read_to_string(path).ok()?;
    let entry = parse_desktop_entry(&content)?;

    // If Type is specified, only Type=Application can be executed
    if let Some(ref t) = entry.entry_type {
        if !t.eq_ignore_ascii_case("Application") {
            return None;
        }
    }

    let exec = entry.exec.as_deref()?;
    parse_exec_command(exec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_desktop_entry_standard() {
        let content = r#"
[Desktop Entry]
Name=PhotoCraft
GenericName=Image Editor
Comment=Digital Art & Photo Editor
Exec=/opt/photocraft/PhotoCraft-1.2.3.AppImage %F
Icon=photocraft
Type=Application
Categories=Graphics;2DGraphics;RasterGraphics;
"#;
        let entry = parse_desktop_entry(content).expect("Should parse standard desktop entry");
        assert_eq!(entry.name.as_deref(), Some("PhotoCraft"));
        assert_eq!(
            entry.exec.as_deref(),
            Some("/opt/photocraft/PhotoCraft-1.2.3.AppImage %F")
        );
        assert_eq!(entry.entry_type.as_deref(), Some("Application"));
    }

    #[test]
    fn test_parse_desktop_entry_with_comments_and_spaces() {
        let content = r#"
# Header comment
[Desktop Entry]
  # Line comment inside
Name = PhotoCraft Studio
Exec = "photocraft" --flag
Type = Application
"#;
        let entry = parse_desktop_entry(content).expect("Should parse with spaces around =");
        assert_eq!(entry.name.as_deref(), Some("PhotoCraft Studio"));
        assert_eq!(entry.exec.as_deref(), Some("\"photocraft\" --flag"));
        assert_eq!(entry.entry_type.as_deref(), Some("Application"));
    }

    #[test]
    fn test_parse_desktop_entry_multiple_sections() {
        let content = r#"
[Other Group]
Name=Wrong
Exec=wrong

[Desktop Entry]
Name=Correct
Exec=/usr/bin/correct
Type=Application

[Desktop Action Gallery]
Name=Open Gallery
Exec=/usr/bin/correct --gallery
"#;
        let entry = parse_desktop_entry(content).expect("Should parse desktop entry section");
        assert_eq!(entry.name.as_deref(), Some("Correct"));
        assert_eq!(entry.exec.as_deref(), Some("/usr/bin/correct"));
    }

    #[test]
    fn test_parse_exec_tokens_quotes_and_spaces() {
        let line =
            r#""/home/user/My Applications/PhotoCraft-1.2.3.AppImage" --flag "two words" %F"#;
        let tokens = parse_exec_tokens(line);
        assert_eq!(
            tokens,
            vec![
                "/home/user/My Applications/PhotoCraft-1.2.3.AppImage",
                "--flag",
                "two words",
                "%F"
            ]
        );
    }

    #[test]
    fn test_parse_exec_tokens_single_quotes() {
        let line = r#"'/opt/my apps/photocraft' 'single param'"#;
        let tokens = parse_exec_tokens(line);
        assert_eq!(tokens, vec!["/opt/my apps/photocraft", "single param"]);
    }

    #[test]
    fn test_parse_exec_tokens_escaped_characters() {
        let line = r#"photocraft\ app --option\ with\ space"#;
        let tokens = parse_exec_tokens(line);
        assert_eq!(tokens, vec!["photocraft app", "--option with space"]);
    }

    #[test]
    fn test_extract_exec_binary_strip_field_codes() {
        let line = r#""/usr/bin/photocraft" %f %F %u %U @@u %U @@"#;
        let bin = extract_exec_binary(line);
        assert_eq!(bin, Some(PathBuf::from("/usr/bin/photocraft")));
    }

    #[test]
    fn test_extract_exec_binary_env_wrapper() {
        let line = r#"env GDK_BACKEND=x11 FOO=BAR "/opt/photocraft/PhotoCraft.AppImage" %F"#;
        let bin = extract_exec_binary(line);
        assert_eq!(
            bin,
            Some(PathBuf::from("/opt/photocraft/PhotoCraft.AppImage"))
        );

        let line2 = r#"/usr/bin/env VAR=123 photocraft %u"#;
        let bin2 = extract_exec_binary(line2);
        assert_eq!(bin2, Some(PathBuf::from("photocraft")));
    }

    #[test]
    fn test_extract_exec_binary_flatpak() {
        let line = r#"/usr/bin/flatpak run --branch=stable io.photocraft.PhotoCraft @@u %U @@"#;
        let bin = extract_exec_binary(line);
        assert_eq!(bin, Some(PathBuf::from("/usr/bin/flatpak")));
    }

    #[test]
    fn test_parse_exec_command_with_temp_binary() {
        let temp_dir = std::env::temp_dir();
        let test_bin = temp_dir.join(format!("test_photocraft_exec_{}.tmp", std::process::id()));
        std::fs::write(&test_bin, b"binary content").expect("Write temp binary");

        let exec_line = format!("\"{}\" %F", test_bin.display());
        let resolved = parse_exec_command(&exec_line);
        assert_eq!(resolved, Some(test_bin.clone()));

        let _ = std::fs::remove_file(&test_bin);
    }

    #[test]
    fn test_parse_exec_command_non_existent() {
        let exec_line = r#""/path/to/nonexistent/photocraft_fake_12345" %F"#;
        let resolved = parse_exec_command(exec_line);
        assert_eq!(resolved, None);
    }

    #[test]
    fn test_parse_exec_command_search_in_path() {
        // `cargo` is guaranteed to be available in PATH during cargo test
        let resolved = parse_exec_command("cargo %F");
        assert!(
            resolved.is_some(),
            "Should locate 'cargo' executable via PATH search"
        );
        let path = resolved.unwrap();
        assert!(path.is_file(), "Resolved path must be an existing file");
    }

    #[test]
    fn test_parse_desktop_file_flow() {
        let temp_dir = std::env::temp_dir();
        let test_bin = temp_dir.join(format!("test_real_app_{}.tmp", std::process::id()));
        std::fs::write(&test_bin, b"exec content").unwrap();

        let desktop_file = temp_dir.join(format!("test_entry_{}.desktop", std::process::id()));
        let desktop_content = format!(
            "[Desktop Entry]\nType=Application\nName=PhotoCraft\nExec=\"{}\" %U\n",
            test_bin.display()
        );
        std::fs::write(&desktop_file, desktop_content).unwrap();

        let resolved = parse_desktop_file(&desktop_file);
        assert_eq!(resolved, Some(test_bin.clone()));

        // Test with non-Application type
        let invalid_desktop = temp_dir.join(format!("invalid_{}.desktop", std::process::id()));
        std::fs::write(
            &invalid_desktop,
            format!(
                "[Desktop Entry]\nType=Directory\nName=PhotoCraft\nExec=\"{}\"\n",
                test_bin.display()
            ),
        )
        .unwrap();
        assert_eq!(parse_desktop_file(&invalid_desktop), None);

        let _ = std::fs::remove_file(&test_bin);
        let _ = std::fs::remove_file(&desktop_file);
        let _ = std::fs::remove_file(&invalid_desktop);
    }
}
