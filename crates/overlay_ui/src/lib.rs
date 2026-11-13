pub mod dynamic_tool;
pub mod i18n;
pub mod icons;
pub mod manager;
pub mod offline_modal;
pub mod overlay;
pub mod store;
pub mod theme;
pub mod toolstrip;
pub mod updater;
pub mod win32_window;

pub use dynamic_tool::{DynamicToolEngine, DynamicWindowState};
pub use eframe::egui;
pub use icons::{
    get_egui_icon_data, get_icon_rgba, get_system_small_icon_size, get_system_window_icon_size,
    get_tray_icon, APP_ICON_SVG,
};
pub use manager::{ActiveTab, PluginManagerApp};
pub use offline_modal::OfflineModalApp;
pub use overlay::{
    create_tray_menu, launch_companion_overlay, launch_plugin_manager, tray_t, CompanionOverlayApp,
    OverlayBar,
};
pub use store::{
    StoreCategoryFilter, StoreEvent, StoreLoadStatus, StorePluginEntry, StoreRegistry, StoreState,
};
pub use theme::{ThemeMode, ThemePalette};
pub use toolstrip::ToolstripApp;
pub use updater::{UpdateCheckResult, UpdateState};
pub use win32_window::{calculate_toolstrip_spawn_pos, find_photocraft_window_rect};
