#[cfg(target_os = "windows")]
pub use win::is_fullscreen_running;

#[cfg(not(target_os = "windows"))]
pub fn is_fullscreen_running() -> bool {
    false
}

#[cfg(target_os = "windows")]
mod win {
    use windows::Win32::{
        Foundation::{BOOL, HWND, LPARAM, RECT},
        Graphics::Gdi::GetMonitorInfoW,
        UI::WindowsAndMessaging::{EnumWindows, GetForegroundWindow, GetWindowRect, IsWindowVisible, MONITORINFO},
    };

    struct CheckState {
        desktop_rect: RECT,
        found: bool,
    }

    unsafe extern "system" fn check_fullscreen(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let state = &mut *(lparam.0 as *mut CheckState);
        if IsWindowVisible(hwnd).as_bool() {
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_ok() {
                let dr = &state.desktop_rect;
                if rect.left <= dr.left
                    && rect.top <= dr.top
                    && rect.right >= dr.right
                    && rect.bottom >= dr.bottom
                {
                    state.found = true;
                    return BOOL(0);
                }
            }
        }
        BOOL(1)
    }

    pub fn is_fullscreen_running() -> bool {
        unsafe {
            use windows::Win32::Graphics::Gdi::{MonitorFromWindow, MONITOR_DEFAULTTOPRIMARY};

            let hwnd = GetForegroundWindow();
            let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY);

            let mut info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };

            if GetMonitorInfoW(hmonitor, &mut info).as_bool() {
                let mut state = CheckState {
                    desktop_rect: info.rcMonitor,
                    found: false,
                };
                let _ = EnumWindows(
                    Some(check_fullscreen),
                    LPARAM(&mut state as *mut _ as isize),
                );
                return state.found;
            }

            false
        }
    }
}
