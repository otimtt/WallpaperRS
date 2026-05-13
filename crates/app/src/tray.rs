pub enum TrayAction {
    Show,
    Pause,
    Quit,
}

// ── Windows implementation ──────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod win {
    use super::TrayAction;
    use tray_icon::{
        menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
        Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
    };

    pub struct AppTray {
        #[allow(dead_code)]
        tray: TrayIcon,
        pub pause_item_id: tray_icon::menu::MenuId,
        pub quit_item_id:  tray_icon::menu::MenuId,
        pub show_item_id:  tray_icon::menu::MenuId,
    }

    impl AppTray {
        pub fn new(rgba: Vec<u8>, width: u32, height: u32) -> anyhow::Result<Self> {
            let icon = Icon::from_rgba(rgba, width, height)
                .map_err(|e| anyhow::anyhow!("Tray icon: {e}"))?;

            let menu = Menu::new();
            let show_item  = MenuItem::new("Mostrar", true, None);
            let pause_item = MenuItem::new("Pausar / Retomar", true, None);
            let sep        = PredefinedMenuItem::separator();
            let quit_item  = MenuItem::new("Sair", true, None);

            let show_id  = show_item.id().clone();
            let pause_id = pause_item.id().clone();
            let quit_id  = quit_item.id().clone();

            menu.append_items(&[&show_item, &pause_item, &sep, &quit_item])
                .map_err(|e| anyhow::anyhow!("Menu: {e}"))?;

            let tray = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("Wallpaper RS")
                .with_icon(icon)
                .build()
                .map_err(|e| anyhow::anyhow!("TrayIcon: {e}"))?;

            Ok(Self { tray, pause_item_id: pause_id, quit_item_id: quit_id, show_item_id: show_id })
        }

        pub fn poll(&self) -> Option<TrayAction> {
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == self.quit_item_id  { return Some(TrayAction::Quit); }
                if event.id == self.show_item_id  { return Some(TrayAction::Show); }
                if event.id == self.pause_item_id { return Some(TrayAction::Pause); }
            }
            if let Ok(_) = TrayIconEvent::receiver().try_recv() {
                return Some(TrayAction::Show);
            }
            None
        }
    }
}

#[cfg(target_os = "windows")]
pub use win::AppTray;

// ── Stub for non-Windows (dev/check only) ───────────────────────────────────

#[cfg(not(target_os = "windows"))]
pub struct AppTray;

#[cfg(not(target_os = "windows"))]
impl AppTray {
    pub fn new(_rgba: Vec<u8>, _w: u32, _h: u32) -> anyhow::Result<Self> {
        Ok(Self)
    }
    pub fn poll(&self) -> Option<TrayAction> {
        None
    }
}
