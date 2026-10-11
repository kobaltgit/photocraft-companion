#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn LoadLibraryA(lpLibFileName: *const u8) -> *mut std::ffi::c_void;
    fn GetProcAddress(
        hModule: *mut std::ffi::c_void,
        lpProcName: *const u8,
    ) -> *mut std::ffi::c_void;
}

#[cfg(windows)]
pub fn apply_win32_menu_theme(mode: i32) {
    unsafe {
        let uxtheme = LoadLibraryA(c"uxtheme.dll".as_ptr() as *const u8);
        if !uxtheme.is_null() {
            // Ordinal 135 is SetPreferredAppMode
            let proc = GetProcAddress(uxtheme, 135_usize as *const u8);
            if !proc.is_null() {
                let func: unsafe extern "system" fn(i32) -> i32 = std::mem::transmute(proc);
                func(mode);
            }
        }
    }
}

#[cfg(not(windows))]
pub fn apply_win32_menu_theme(_mode: i32) {}
