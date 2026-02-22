use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager, Runtime,
};

pub fn create_tray<R: Runtime>(app: &tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let open_mini = MenuItemBuilder::with_id("open_mini", "打开迷你窗口").build(app)?;
    let open_main = MenuItemBuilder::with_id("open_main", "打开主窗口").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&open_mini)
        .item(&open_main)
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
            "open_mini" => {
                if let Some(main_win) = app_handle.get_webview_window("main") {
                    let _ = main_win.hide();
                }
                if let Some(mini_win) = app_handle.get_webview_window("mini") {
                    let _ = mini_win.show();
                    let _ = mini_win.set_focus();
                }
            }
            "open_main" => {
                if let Some(mini_win) = app_handle.get_webview_window("mini") {
                    let _ = mini_win.hide();
                }
                if let Some(main_win) = app_handle.get_webview_window("main") {
                    let _ = main_win.show();
                    let _ = main_win.set_focus();
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
                if let Some(mini_win) = app.get_webview_window("mini") {
                    let _ = mini_win.hide();
                }
                if let Some(main_win) = app.get_webview_window("main") {
                    let _ = main_win.show();
                    let _ = main_win.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
