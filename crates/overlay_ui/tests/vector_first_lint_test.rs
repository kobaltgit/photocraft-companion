use std::fs;
use std::path::Path;

/// Character categories forbidden in UI text according to Rule 6 (Vector-First UI):
/// 1. Emojis and Pictographs (e.g., ⏳, ⚠️, ✨, 🪄, 🔥, 🚀, 💡, 🤖, 🔄, 📥)
/// 2. Miscellaneous Symbols (e.g., ⚙, ⚡)
/// 3. Dingbats & Checkmarks/Crosses (e.g., ✓, ✔, ✕, ✖)
/// 4. Unicode geometric shapes & bullets (e.g., ● U+25CF, • U+2022)
/// 5. Unicode arrows (e.g., ⬇, ⬆, ⬅, ➡)
fn is_prohibited_ui_char(ch: char) -> Option<&'static str> {
    let code = ch as u32;

    // Explicitly banned symbols from Rule 6
    match ch {
        '⏳' => {
            return Some("Hourglass / Busy emoji (U+23F3) - use '...' or vector spinner instead")
        }
        '\u{26A0}' => return Some("Warning emoji (U+26A0) - use vector icon instead"),
        '⬇' | '⬆' | '⬅' | '➡' => {
            return Some(
                "Arrow symbol/emoji - use vector primitives (painter.line_segment) instead",
            )
        }
        '✨' | '🪄' | '🔥' | '🚀' | '💡' => {
            return Some("Emoji - use vector icons or text badges instead")
        }
        '●' => {
            return Some("Black Circle bullet (U+25CF) - use ui.painter().circle_filled() instead")
        }
        '•' => {
            return Some(
                "Bullet symbol (U+2022) - causes tofu in egui; use '-', '|', or vector dot instead",
            )
        }
        '✓' | '✔' => {
            return Some(
                "Checkmark symbol (U+2713/U+2714) - use text status or vector checkmark instead",
            )
        }
        '✕' | '✖' => {
            return Some("Cross symbol (U+2715/U+2716) - use vector cross or standard 'x' instead")
        }
        '⚙' => return Some("Gear symbol (U+2699) - use vector SvgIconCache instead"),
        '🔍' => return Some("Search symbol (U+1F50D) - use vector SvgIconCache instead"),
        _ => {}
    }

    // Unicode Blocks that cause tofu in egui:
    // Emoji & Pictographs: 0x1F300..=0x1FAFF
    if (0x1F300..=0x1FAFF).contains(&code) {
        return Some("Emoji / Pictograph (U+1F300..U+1FAFF) - not supported by egui default font");
    }

    // Dingbats: 0x2700..=0x27BF
    if (0x2700..=0x27BF).contains(&code) {
        return Some("Dingbat symbol (U+2700..U+27BF) - not supported by egui default font");
    }

    // Miscellaneous Symbols: 0x2600..=0x26FF
    if (0x2600..=0x26FF).contains(&code) {
        return Some("Miscellaneous Symbol (U+2600..U+26FF) - not supported by egui default font");
    }

    // Geometric Shapes: 0x25A0..=0x25FF
    if (0x25A0..=0x25FF).contains(&code) {
        return Some("Geometric shape (U+25A0..U+25FF) - use ui.painter() primitives instead");
    }

    // Arrows: 0x2190..=0x21FF and 0x2B00..=0x2BFF
    if (0x2190..=0x21FF).contains(&code) || (0x2B00..=0x2BFF).contains(&code) {
        return Some("Arrow symbol (U+2190..U+21FF or U+2B00..U+2BFF) - use ui.painter() instead");
    }

    // Punctuation bullets (U+2020..=U+2027)
    if (0x2020..=0x2027).contains(&code) {
        return Some("Punctuation bullet/dagger (U+2020..U+2027) - causes tofu in egui");
    }

    None
}

/// Strip line comments (`// ...`) outside of string literals
fn strip_line_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;
    let mut chars = line.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && in_string {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if !in_string && ch == '/' {
            if let Some(&(_, '/')) = chars.peek() {
                return &line[..i];
            }
        }
    }
    line
}

#[test]
fn test_no_unsupported_emoji_or_unicode_symbols_in_ui_crates() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let base_dir = Path::new(manifest_dir);

    // Scan overlay_ui and companion_app src directories
    let src_dirs = [
        base_dir.join("src"),
        base_dir.parent().unwrap().join("companion_app").join("src"),
    ];

    let mut violations = Vec::new();
    let mut files_scanned = 0;

    for dir in &src_dirs {
        if !dir.exists() {
            continue;
        }

        let entries = fs::read_dir(dir).unwrap_or_else(|e| {
            panic!("Failed to read dir {}: {e}", dir.display());
        });

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }

            files_scanned += 1;
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));

            for (line_idx, line) in content.lines().enumerate() {
                let code_part = strip_line_comment(line);
                let trimmed = code_part.trim();
                if trimmed.is_empty() {
                    continue;
                }

                for (col, ch) in code_part.char_indices() {
                    if let Some(reason) = is_prohibited_ui_char(ch) {
                        violations.push(format!(
                            "{}:{}:{} - Prohibited '{}' (U+{:04X}): {}\n    Code: {}",
                            file_name,
                            line_idx + 1,
                            col + 1,
                            ch,
                            ch as u32,
                            reason,
                            trimmed
                        ));
                    }
                }
            }
        }
    }

    assert!(
        files_scanned >= 5,
        "Expected to scan at least 5 UI source files, found {}",
        files_scanned
    );

    if !violations.is_empty() {
        panic!(
            "\n\n================ RULE 6 (Vector-First UI) VIOLATIONS DETECTED ================\n\
            Found {} forbidden character(s) in UI code that cause tofu squares (□) in egui:\n\n\
            {}\n\
            ===============================================================================\n",
            violations.len(),
            violations.join("\n\n")
        );
    }
}

#[test]
fn test_vector_first_linter_recognizes_all_rule6_examples() {
    let forbidden_samples = [
        ('⏳', "Hourglass"),
        ('\u{26A0}', "Warning"),
        ('⬇', "Down arrow"),
        ('⬆', "Up arrow"),
        ('⬅', "Left arrow"),
        ('➡', "Right arrow"),
        ('✨', "Sparkles"),
        ('🪄', "Magic wand"),
        ('🔥', "Fire"),
        ('🚀', "Rocket"),
        ('💡', "Bulb"),
        ('●', "Circle bullet"),
        ('•', "Bullet"),
        ('✓', "Checkmark"),
        ('✔', "Heavy checkmark"),
        ('✕', "Multiplication cross"),
        ('✖', "Heavy multiplication cross"),
        ('⚙', "Gear"),
        ('🔍', "Magnifying glass"),
        ('🤖', "Robot"),
        ('⚡', "Voltage"),
        ('🔄', "Arrows counterclockwise"),
        ('📥', "Inbox tray"),
    ];

    for (ch, name) in forbidden_samples {
        assert!(
            is_prohibited_ui_char(ch).is_some(),
            "Linter failed to recognize forbidden character: {} ({})",
            ch,
            name
        );
    }

    let allowed_samples = [
        'A', 'z', '0', '9', ' ', '\t', '\n', '-', '_', '/', '\\', ':', '.', ',', '!', '?', '(',
        ')', '[', ']', '{', '}', '+', '=', '*', '%', '&', '|', '<', '>', '~', '@', '#', '$', '^',
        '"', '\'', '`', 'А', 'я', 'Ё', 'ё', '«', '»',
    ];

    for ch in allowed_samples {
        assert!(
            is_prohibited_ui_char(ch).is_none(),
            "Linter falsely rejected allowed character: '{}' (U+{:04X})",
            ch,
            ch as u32
        );
    }
}
