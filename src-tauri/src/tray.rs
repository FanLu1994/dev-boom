use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager, Runtime,
};

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
                if let Some(mini_win) = app_handle.get_webview_window("mini") {
                    let _ = mini_win.hide();
                }
                if let Some(main_win) = app_handle.get_webview_window("main") {
                    let _ = main_win.show();
                    let _ = main_win.set_focus();
                }
            }
            "show_mini" => {
                if let Some(main_win) = app_handle.get_webview_window("main") {
                    let _ = main_win.hide();
                }
                if let Some(mini_win) = app_handle.get_webview_window("mini") {
                    let _ = mini_win.show();
                    let _ = mini_win.set_focus();
                }
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
                // 左键点击托盘图标：显示主窗口
                if let Some(mini_win) = app.get_webview_window("mini") {
                    let _ = mini_win.hide();
                }
                if let Some(main_win) = app.get_webview_window("main") {
                    let _ = main_win.show();
                    let _ = main_win.set_focus();
                }
            } else if let tauri::tray::TrayIconEvent::DoubleClick {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app = tray_icon.app_handle();
                // 双击托盘图标：显示迷你窗口
                if let Some(main_win) = app.get_webview_window("main") {
                    let _ = main_win.hide();
                }
                if let Some(mini_win) = app.get_webview_window("mini") {
                    let _ = mini_win.show();
                    let _ = mini_win.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
