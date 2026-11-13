/// Helper to construct a localized tray menu matching the user's active/detected language.
pub fn create_tray_menu(lang: core_contracts::Language) -> tray_icon::menu::Menu {
    let tray_menu = tray_icon::menu::Menu::new();
    let toolbar_text = tray_t("PhotoCraft Toolbar", "Панель PhotoCraft", lang);
    let manager_text = tray_t("Plugins Manager", "Менеджер плагинов", lang);
    let reset_text = tray_t("Reset Layout", "Сбросить расположение", lang);
    let exit_text = tray_t("Exit Companion", "Выход из Компаньона", lang);

    let open_toolbar_item =
        tray_icon::menu::MenuItem::with_id("tray_toolbar", toolbar_text, true, None);
    let open_manager_item =
        tray_icon::menu::MenuItem::with_id("tray_manager", manager_text, true, None);
    let reset_layout_item =
        tray_icon::menu::MenuItem::with_id("tray_reset_layout", reset_text, true, None);
    let separator = tray_icon::menu::PredefinedMenuItem::separator();
    let exit_item = tray_icon::menu::MenuItem::with_id("tray_exit", exit_text, true, None);

    let _ = tray_menu.append(&open_toolbar_item);
    let _ = tray_menu.append(&open_manager_item);
    let _ = tray_menu.append(&reset_layout_item);
    let _ = tray_menu.append(&separator);
    let _ = tray_menu.append(&exit_item);

    tray_menu
}

/// Localizes a phrase between English, Russian, and any other supported dictionary language.
pub fn tray_t<'a>(en: &'a str, ru: &'a str, lang: core_contracts::Language) -> &'a str {
    match lang {
        core_contracts::Language::Ru => ru,
        core_contracts::Language::En => en,
        other => crate::i18n::lookup(en, other).unwrap_or(en),
    }
}
