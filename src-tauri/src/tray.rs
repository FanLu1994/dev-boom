use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager, Runtime,
};

#[derive(Clone, Copy)]
enum WindowMode {
    Main,
    Mini,
}

impl WindowMode {
    fn from_str(s: &str) -> Self {
        match s {
            "mini" => WindowMode::Mini,
            _ => WindowMode::Main,
        }
    }
}

fn show_window_mode<R: Runtime>(app: &tauri::AppHandle<R>, mode: WindowMode) {
    match mode {
        WindowMode::Main => {
            if let Some(mini_win) = app.get_webview_window("mini") {
                let _ = mini_win.hide();
            }
            if let Some(main_win) = app.get_webview_window("main") {
                let _ = main_win.show();
                let _ = main_win.set_focus();
            }
        }
        WindowMode::Mini => {
            if let Some(main_win) = app.get_webview_window("main") {
                let _ = main_win.hide();
            }
            if let Some(mini_win) = app.get_webview_window("mini") {
                let _ = mini_win.show();
                let _ = mini_win.set_focus();
            }
        }
    }
}

pub fn create_tray<R: Runtime>(app: &tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let show_main = MenuItemBuilder::with_id("show_main", "显示主窗口").build(app)?;
    let show_mini = MenuItemBuilder::with_id("show_mini", "显示迷你窗口").build(app)?;
    let hide_all = MenuItemBuilder::with_id("hide_all", "隐藏所有窗口").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出程序").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show_main)
        .item(&show_mini)
        .item(&hide_all)
        .separator()
        .item(&quit)
        .build()?;

    let _tray = TrayIconBuilder::new()
        .icon(Image::from_path("icons/icon.ico").unwrap_or_else(|_| {
            app.default_window_icon()
                .cloned()
                .unwrap_or_else(|| Image::new(&[], 0, 0))
        }))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app_handle, event| match event.id().as_ref() {
            "show_main" => {
                show_window_mode(app_handle, WindowMode::Main);
            }
            "show_mini" => {
                show_window_mode(app_handle, WindowMode::Mini);
            }
            "hide_all" => {
                if let Some(main_win) = app_handle.get_webview_window("main") {
                    let _ = main_win.hide();
                }
                if let Some(mini_win) = app_handle.get_webview_window("mini") {
                    let _ = mini_win.hide();
                }
            }
            "quit" => {
                app_handle.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray_icon, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app = tray_icon.app_handle();
                // 获取最后激活的窗口模式
                let last_window = app.state::<crate::AppState>();
                let last_window_guard = last_window.last_active_window.lock().unwrap();
                let mode = WindowMode::from_str(
                    last_window_guard
                        .as_ref()
                        .as_deref()
                        .unwrap_or(&"main".to_string())
                );
                drop(last_window_guard);
                show_window_mode(&app, mode);
            } else if let tauri::tray::TrayIconEvent::DoubleClick {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app = tray_icon.app_handle();
                // 双击切换到另一个窗口
                let last_window = app.state::<crate::AppState>();
                let last_window_guard = last_window.last_active_window.lock().unwrap();
                let current = WindowMode::from_str(
                    last_window_guard
                        .as_ref()
                        .as_deref()
                        .unwrap_or(&"main".to_string())
                );
                drop(last_window_guard);

                let new_mode = match current {
                    WindowMode::Main => WindowMode::Mini,
                    WindowMode::Mini => WindowMode::Main,
                };
                show_window_mode(&app, new_mode);
            }
        })
        .build(app)?;

    Ok(())
}
