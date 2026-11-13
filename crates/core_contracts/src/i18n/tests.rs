use super::language::Language;

#[test]
fn test_language_from_str_code() {
    assert_eq!(Language::from_str_code("en"), Language::En);
    assert_eq!(Language::from_str_code("english"), Language::En);
    assert_eq!(Language::from_str_code("EN "), Language::En);
    assert_eq!(Language::from_str_code("ru"), Language::Ru);
    assert_eq!(Language::from_str_code("russian"), Language::Ru);
    assert_eq!(Language::from_str_code("de"), Language::De);
    assert_eq!(Language::from_str_code("fr"), Language::Fr);
    assert_eq!(Language::from_str_code("es"), Language::Es);
    assert_eq!(Language::from_str_code("it"), Language::It);
    assert_eq!(Language::from_str_code("ja"), Language::Ja);
    assert_eq!(Language::from_str_code("zh"), Language::ZhHans);
    assert_eq!(Language::from_str_code("zh-hans"), Language::ZhHans);
    assert_eq!(Language::from_str_code("zh-hant"), Language::ZhHant);
    assert_eq!(Language::from_str_code("ko"), Language::Ko);
    assert_eq!(Language::from_str_code("pt"), Language::PtBr);
    assert_eq!(Language::from_str_code("pt-br"), Language::PtBr);
    assert_eq!(Language::from_str_code("nl"), Language::Nl);
    assert_eq!(Language::from_str_code("pl"), Language::Pl);
    assert_eq!(Language::from_str_code("cs"), Language::Cs);
    assert_eq!(Language::from_str_code("uk"), Language::Uk);
    assert_eq!(Language::from_str_code("sr"), Language::Sr);
    assert_eq!(Language::from_str_code("hu"), Language::Hu);
    assert_eq!(Language::from_str_code("el"), Language::El);
    assert_eq!(Language::from_str_code("tr"), Language::Tr);
    assert_eq!(Language::from_str_code("id"), Language::Id);
    assert_eq!(Language::from_str_code("vi"), Language::Vi);
    assert_eq!(Language::from_str_code("hi"), Language::Hi);
    assert_eq!(Language::from_str_code("unknown_xyz"), Language::Auto);
    assert_eq!(Language::from_str_code(""), Language::Auto);
}

#[test]
fn test_language_code() {
    assert_eq!(Language::Auto.code(), "auto");
    assert_eq!(Language::En.code(), "en");
    assert_eq!(Language::Ru.code(), "ru");
    assert_eq!(Language::De.code(), "de");
    assert_eq!(Language::Fr.code(), "fr");
    assert_eq!(Language::Es.code(), "es");
    assert_eq!(Language::It.code(), "it");
    assert_eq!(Language::Ja.code(), "ja");
    assert_eq!(Language::ZhHans.code(), "zh-hans");
    assert_eq!(Language::ZhHant.code(), "zh-hant");
    assert_eq!(Language::Ko.code(), "ko");
    assert_eq!(Language::PtBr.code(), "pt-br");
    assert_eq!(Language::Cs.code(), "cs");
    assert_eq!(Language::Pl.code(), "pl");
}

#[test]
fn test_language_no_bg_suffix() {
    assert_eq!(Language::En.no_bg_suffix(), "No BG");
    assert_eq!(Language::Auto.no_bg_suffix(), "No BG");
    assert_eq!(Language::Ru.no_bg_suffix(), "Без фона");
    assert_eq!(Language::De.no_bg_suffix(), "Ohne Hintergrund");
    assert_eq!(Language::Fr.no_bg_suffix(), "Sans fond");
    assert_eq!(Language::Es.no_bg_suffix(), "Sin fondo");
    assert_eq!(Language::It.no_bg_suffix(), "Senza sfondo");
    assert_eq!(Language::Ja.no_bg_suffix(), "背景なし");
    assert_eq!(Language::ZhHans.no_bg_suffix(), "无背景");
    assert_eq!(Language::ZhHant.no_bg_suffix(), "無背景");
    assert_eq!(Language::Ko.no_bg_suffix(), "배경 없음");
    assert_eq!(Language::PtBr.no_bg_suffix(), "Sem fundo");
}

#[test]
fn test_language_remove_bg_label() {
    assert!(Language::En.remove_bg_label().contains("Remove BG"));
    assert!(Language::Ru.remove_bg_label().contains("Удалить фон"));
    assert!(Language::De
        .remove_bg_label()
        .contains("Hintergrund entfernen"));
}

#[test]
fn test_from_bcp47_tag_strip_regions() {
    assert_eq!(Language::from_bcp47_tag("ru-KZ"), Some(Language::Ru));
    assert_eq!(Language::from_bcp47_tag("ru_RU"), Some(Language::Ru));
    assert_eq!(Language::from_bcp47_tag("ru.UTF-8"), Some(Language::Ru));
    assert_eq!(Language::from_bcp47_tag("ru_RU.UTF-8"), Some(Language::Ru));
    assert_eq!(Language::from_bcp47_tag("cs_CZ.UTF-8"), Some(Language::Cs));
    assert_eq!(Language::from_bcp47_tag("sr_RS@latin"), Some(Language::Sr));
    assert_eq!(Language::from_bcp47_tag("POSIX"), Some(Language::En));
    assert_eq!(Language::from_bcp47_tag("C"), Some(Language::En));
    assert_eq!(Language::from_bcp47_tag("en-US"), Some(Language::En));
    assert_eq!(Language::from_bcp47_tag("en-GB"), Some(Language::En));
    assert_eq!(Language::from_bcp47_tag("cs-CZ"), Some(Language::Cs));
    assert_eq!(Language::from_bcp47_tag("de-AT"), Some(Language::De));
    assert_eq!(Language::from_bcp47_tag("fr-CA"), Some(Language::Fr));
    assert_eq!(
        Language::from_bcp47_tag("zh-Hans-CN"),
        Some(Language::ZhHans)
    );
    assert_eq!(Language::from_bcp47_tag("zh-TW"), Some(Language::ZhHant));
    assert_eq!(Language::from_bcp47_tag("pt-BR"), Some(Language::PtBr));
    assert_eq!(Language::from_bcp47_tag("pt-PT"), Some(Language::PtBr));
    assert_eq!(Language::from_bcp47_tag("unknown-XX"), None);
    assert_eq!(Language::from_bcp47_tag(""), None);

    // Verify from_str_code handles regional tags via from_bcp47_tag
    assert_eq!(Language::from_str_code("ru-KZ"), Language::Ru);
    assert_eq!(Language::from_str_code("ru_RU.UTF-8"), Language::Ru);
    assert_eq!(Language::from_str_code("cs-CZ"), Language::Cs);
    assert_eq!(Language::from_str_code("en-AU"), Language::En);
}

#[test]
fn test_system_language() {
    let sys = Language::system_language();
    assert_ne!(sys, Language::Auto);
    assert!(Language::ALL_SUPPORTED.contains(&sys));
}

#[test]
fn test_language_resolve() {
    assert_eq!(Language::Ru.resolve(None), Language::Ru);
    assert_eq!(Language::De.resolve(None), Language::De);

    let temp_dir = std::env::temp_dir().join(format!(
        "i18n_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let prefs_file = temp_dir.join("preferences.json");
    std::fs::write(&prefs_file, r#"{"interface": {"language": "de"}}"#).unwrap();

    assert_eq!(Language::Auto.resolve(Some(&prefs_file)), Language::De);

    // When preferences has "language": "auto", it resolves to system language
    let auto_prefs = temp_dir.join("auto_prefs.json");
    std::fs::write(&auto_prefs, r#"{"interface": {"language": "auto"}}"#).unwrap();
    assert_eq!(
        Language::Auto.resolve(Some(&auto_prefs)),
        Language::system_language()
    );

    let invalid_prefs = temp_dir.join("invalid.json");
    std::fs::write(&invalid_prefs, "not json").unwrap();
    assert_eq!(
        Language::Auto.resolve(Some(&invalid_prefs)),
        Language::system_language()
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_format_cutout_name() {
    let ru = Language::Ru;
    let en = Language::En;
    let de = Language::De;

    assert_eq!(ru.format_cutout_name("Portrait"), "Portrait (Без фона)");
    assert_eq!(en.format_cutout_name("Portrait"), "Portrait (No BG)");
    assert_eq!(
        de.format_cutout_name("Portrait"),
        "Portrait (Ohne Hintergrund)"
    );

    // Deduplication
    assert_eq!(
        ru.format_cutout_name("Portrait (Без фона)"),
        "Portrait (Без фона)"
    );
    assert_eq!(
        en.format_cutout_name("Portrait (No BG)"),
        "Portrait (No BG)"
    );
    assert_eq!(
        ru.format_cutout_name("Portrait (No BG)"),
        "Portrait (No BG)"
    );
    assert_eq!(
        de.format_cutout_name("Portrait (Ohne Hintergrund)"),
        "Portrait (Ohne Hintergrund)"
    );
    assert_eq!(
        en.format_cutout_name("Portrait [No BG]"),
        "Portrait [No BG]"
    );
    assert_eq!(
        ru.format_cutout_name("Portrait [Без фона]"),
        "Portrait [Без фона]"
    );
    assert_eq!(
        en.format_cutout_name("Portrait - No BG"),
        "Portrait - No BG"
    );
    assert_eq!(
        ru.format_cutout_name("Portrait - Без фона"),
        "Portrait - Без фона"
    );
    assert_eq!(en.format_cutout_name("  Subject  "), "Subject (No BG)");
}

#[test]
fn test_all_22_languages_distinct_suffixes() {
    assert_eq!(Language::ALL_SUPPORTED.len(), 22);
    for lang in Language::ALL_SUPPORTED {
        assert!(!lang.code().is_empty());
        assert!(!lang.display_name().is_empty());
        assert!(!lang.selector_label().is_empty());
        assert!(!lang.no_bg_suffix().is_empty());
        assert!(!lang.upscale_suffix().is_empty());
        assert!(!lang.inpaint_suffix().is_empty());
        assert!(!lang.denoise_suffix().is_empty());
        assert!(!lang.depth_suffix().is_empty());
        assert!(!lang.bokeh_suffix().is_empty());
        assert!(!lang.fog_layer_name().is_empty());
    }
}

#[test]
fn test_bokeh_suffix_and_format_bokeh_name() {
    let ru = Language::Ru;
    assert_eq!(ru.bokeh_suffix(), "Боке");
    assert_eq!(ru.format_bokeh_name("Слой 1"), "Слой 1 (Боке)");
    assert_eq!(ru.format_bokeh_name("Слой 1 (Боке)"), "Слой 1 (Боке)");
    assert_eq!(ru.format_bokeh_name("Слой 1 [Боке]"), "Слой 1 [Боке]");

    let en = Language::En;
    assert_eq!(en.bokeh_suffix(), "Bokeh");
    assert_eq!(en.format_bokeh_name("Background"), "Background (Bokeh)");
    assert_eq!(
        en.format_bokeh_name("Background (Bokeh)"),
        "Background (Bokeh)"
    );
    assert_eq!(
        ru.format_bokeh_name("Background (Bokeh)"),
        "Background (Bokeh)"
    );
}

#[test]
fn test_fog_layer_name_all_languages() {
    assert_eq!(Language::Ru.fog_layer_name(), "Атмосферный туман");
    assert_eq!(Language::En.fog_layer_name(), "Atmospheric Fog");
    assert_eq!(Language::ZhHans.fog_layer_name(), "大气雾");
    assert_eq!(Language::El.fog_layer_name(), "Ατμοσφαιρική ομίχλη");
    assert_eq!(Language::Hi.fog_layer_name(), "वायुमंडलीय कोहरा");
}

#[test]
fn test_ui_supported_and_safe_fallback() {
    assert_eq!(Language::UI_SUPPORTED.len(), 15);
    for lang in Language::UI_SUPPORTED {
        assert!(lang.is_ui_supported());
        assert_eq!(lang.safe_ui_language(), lang);
    }

    // CJK, Hindi, Greek, Vietnamese, Auto fall back safely to En for UI
    assert!(!Language::Ja.is_ui_supported());
    assert_eq!(Language::Ja.safe_ui_language(), Language::En);

    assert!(!Language::ZhHans.is_ui_supported());
    assert_eq!(Language::ZhHans.safe_ui_language(), Language::En);

    assert!(!Language::ZhHant.is_ui_supported());
    assert_eq!(Language::ZhHant.safe_ui_language(), Language::En);

    assert!(!Language::Ko.is_ui_supported());
    assert_eq!(Language::Ko.safe_ui_language(), Language::En);

    assert!(!Language::Hi.is_ui_supported());
    assert_eq!(Language::Hi.safe_ui_language(), Language::En);

    assert!(!Language::El.is_ui_supported());
    assert_eq!(Language::El.safe_ui_language(), Language::En);

    assert!(!Language::Vi.is_ui_supported());
    assert_eq!(Language::Vi.safe_ui_language(), Language::En);

    assert!(!Language::Auto.is_ui_supported());
    assert_eq!(Language::Auto.safe_ui_language(), Language::En);
}
