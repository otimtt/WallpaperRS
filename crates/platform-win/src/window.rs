// Win32 desktop wallpaper window — creates a borderless full-screen window
// and injects it into WorkerW so it renders behind desktop icons.

#[cfg(target_os = "windows")]
pub use win::{create_wallpaper_window, destroy_window, pump_messages};

#[cfg(not(target_os = "windows"))]
pub fn create_wallpaper_window() -> anyhow::Result<(usize, u32, u32)> {
    anyhow::bail!("Wallpaper window only available on Windows")
}
#[cfg(not(target_os = "windows"))]
pub fn destroy_window(_hwnd: usize) {}
#[cfg(not(target_os = "windows"))]
pub fn pump_messages(_hwnd: usize) {}

#[cfg(target_os = "windows")]
mod win {
    use anyhow::{anyhow, Result};
    use windows::{
        core::PCWSTR,
        w,
        Win32::{
            Foundation::{HWND, LPARAM, LRESULT, WPARAM},
            System::LibraryLoader::GetModuleHandleW,
            UI::WindowsAndMessaging::*,
        },
    };

    pub fn create_wallpaper_window() -> Result<(usize, u32, u32)> {
        unsafe {
            let hmodule  = GetModuleHandleW(PCWSTR::null())?;
            let hinstance = windows::Win32::Foundation::HINSTANCE(hmodule.0);

            let width  = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);

            // Register class (ignore error — class might already exist from a prior run)
            let wc = WNDCLASSEXW {
                cbSize:        std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc:   Some(wnd_proc),
                hInstance:     hinstance,
                lpszClassName: w!("WallpaperRS_Desktop"),
                ..Default::default()
            };
            RegisterClassExW(&wc);

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("WallpaperRS_Desktop"),
                w!("WallpaperRS"),
                WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS,
                0, 0, width, height,
                HWND::default(), HMENU::default(), hinstance, None,
            )?;

            // Inject into WorkerW so the window lives behind desktop icons
            super::super::worker_window::inject_hwnd(hwnd)?;

            log::info!("Desktop wallpaper window created: {}×{}", width, height);
            Ok((hwnd.0 as usize, width as u32, height as u32))
        }
    }

    pub fn destroy_window(hwnd: usize) {
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(
                HWND(hwnd as _)
            );
        }
    }

    pub fn pump_messages(_hwnd: usize) {
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
    ) -> LRESULT {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}
