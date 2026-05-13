#[cfg(target_os = "windows")]
pub use win::{inject_hwnd, DesktopWindow};

#[cfg(not(target_os = "windows"))]
pub fn inject_hwnd(_hwnd: usize) -> anyhow::Result<()> { Ok(()) }

#[cfg(not(target_os = "windows"))]
pub struct DesktopWindow;
#[cfg(not(target_os = "windows"))]
impl DesktopWindow {
    pub fn inject(_hwnd: usize) -> anyhow::Result<Self> { Ok(Self) }
}

#[cfg(target_os = "windows")]
mod win {
    use anyhow::{anyhow, Result};
    use windows::{
        core::PCWSTR,
        w,
        Win32::Foundation::{BOOL, HWND, LPARAM},
        Win32::UI::WindowsAndMessaging::*,
    };

    pub fn inject_hwnd(hwnd: HWND) -> Result<()> {
        unsafe {
            let progman = FindWindowW(w!("Progman"), PCWSTR::null());
            if progman.0 == 0 {
                return Err(anyhow!("Progman not found"));
            }
            SendMessageTimeoutW(progman, 0x052C, Default::default(), Default::default(), SMTO_NORMAL, 1000, None);

            let mut worker = HWND::default();
            let _ = EnumWindows(Some(find_worker_w), LPARAM(&mut worker as *mut HWND as isize));

            if worker.0 == 0 {
                return Err(anyhow!("WorkerW not found"));
            }

            SetParent(hwnd, worker)?;
            log::info!("Window injected into WorkerW");
            Ok(())
        }
    }

    unsafe extern "system" fn find_worker_w(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let p = lparam.0 as *mut HWND;
        let def_view = FindWindowExW(hwnd, HWND::default(), w!("SHELLDLL_DefView"), PCWSTR::null());
        if def_view.0 != 0 {
            let worker = FindWindowExW(HWND::default(), hwnd, w!("WorkerW"), PCWSTR::null());
            if worker.0 != 0 {
                *p = worker;
            }
        }
        BOOL(1)
    }

    pub struct DesktopWindow;
    impl DesktopWindow {
        pub fn inject(hwnd: usize) -> anyhow::Result<Self> {
            inject_hwnd(windows::Win32::Foundation::HWND(hwnd as _))?;
            Ok(Self)
        }
    }
}
