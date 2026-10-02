//! Windows system tray integration and menu actions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Open,
    ToggleHotkey,
    Exit,
}

#[cfg(all(windows, feature = "native"))]
mod platform {
    use super::TrayAction;
    use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
    use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

    pub struct SystemTray {
        _menu: Menu,
        _tray: TrayIcon,
        events: std::sync::mpsc::Receiver<TrayAction>,
        toggle_item: MenuItem,
    }

    impl SystemTray {
        pub fn new(hotkey_enabled: bool, ctx: eframe::egui::Context) -> crate::error::Result<Self> {
            let menu = Menu::new();
            let open_item = MenuItem::with_id("open", "Open TinySTT", true, None);
            let toggle_item =
                MenuItem::with_id("toggle-hotkey", toggle_label(hotkey_enabled), true, None);
            let separator = PredefinedMenuItem::separator();
            let exit_item = MenuItem::with_id("exit", "Exit", true, None);
            menu.append_items(&[&open_item, &toggle_item, &separator, &exit_item])
                .map_err(|e| crate::error::TinySttError::Settings(format!("tray menu: {e}")))?;

            let open_id = open_item.id().clone();
            let toggle_id = toggle_item.id().clone();
            let exit_id = exit_item.id().clone();
            let (event_tx, event_rx) = std::sync::mpsc::channel();
            MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
                let action = if event.id() == &open_id {
                    Some(TrayAction::Open)
                } else if event.id() == &toggle_id {
                    Some(TrayAction::ToggleHotkey)
                } else if event.id() == &exit_id {
                    Some(TrayAction::Exit)
                } else {
                    None
                };
                if let Some(action) = action {
                    let _ = event_tx.send(action);
                    ctx.request_repaint();
                }
            }));

            let icon = build_icon()
                .map_err(|e| crate::error::TinySttError::Settings(format!("tray icon: {e}")))?;
            let tray = TrayIconBuilder::new()
                .with_menu(Box::new(menu.clone()))
                .with_tooltip("TinySTT - Hold to speak")
                .with_icon(icon)
                .build()
                .map_err(|e| crate::error::TinySttError::Settings(format!("tray icon: {e}")))?;

            Ok(Self {
                _menu: menu,
                _tray: tray,
                events: event_rx,
                toggle_item,
            })
        }

        pub fn poll(&self) -> Option<TrayAction> {
            self.events.try_recv().ok()
        }

        pub fn set_hotkey_enabled(&self, enabled: bool) {
            self.toggle_item.set_text(toggle_label(enabled));
        }
    }

    fn toggle_label(enabled: bool) -> &'static str {
        if enabled {
            "Disable Hotkey"
        } else {
            "Enable Hotkey"
        }
    }

    fn build_icon() -> std::result::Result<Icon, tray_icon::BadIcon> {
        const SIZE: u32 = 32;
        let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];
        for y in 0..SIZE {
            for x in 0..SIZE {
                let index = ((y * SIZE + x) * 4) as usize;
                let dx = x as f32 - 15.5;
                let dy = y as f32 - 15.5;
                let in_circle = dx * dx + dy * dy <= 14.5 * 14.5;
                if in_circle {
                    rgba[index..index + 4].copy_from_slice(&[14, 116, 144, 255]);
                }
            }
        }

        for (x, height) in [(10_u32, 10_u32), (14, 18), (18, 24), (22, 14)] {
            let half = height / 2;
            for y in (16 - half)..(16 + half) {
                let index = ((y * SIZE + x) * 4) as usize;
                rgba[index..index + 4].copy_from_slice(&[245, 250, 252, 255]);
            }
        }

        Icon::from_rgba(rgba, SIZE, SIZE)
    }
}
#[cfg(not(all(windows, feature = "native")))]
mod platform {
    use super::TrayAction;

    pub struct SystemTray;

    impl SystemTray {
        pub fn new<T>(_hotkey_enabled: bool, _ctx: T) -> crate::error::Result<Self> {
            Err(crate::error::TinySttError::HotkeyRegistration(
                "system tray is only supported on Windows".to_string(),
            ))
        }

        pub fn poll(&self) -> Option<TrayAction> {
            None
        }

        pub fn set_hotkey_enabled(&self, _enabled: bool) {}
    }
}

pub use platform::SystemTray;
