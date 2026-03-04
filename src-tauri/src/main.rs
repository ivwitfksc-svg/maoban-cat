// 喵伴 - 桌面宠物猫主程序
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, Window, WindowBuilder, WindowUrl,
};
use std::sync::Mutex;

// 应用状态
struct AppState {
    cat_name: Mutex<String>,
    auto_start: Mutex<bool>,
}

fn main() {
    // 创建托盘菜单
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show_info", "🐱 查看状态"))
        .add_item(CustomMenuItem::new("rename", "✏️ 改名"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("auto_start", "⚡ 开机启动"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit", "❌ 退出"));

    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .manage(AppState {
            cat_name: Mutex::new("小黑".to_string()),
            auto_start: Mutex::new(false),
        })
        .system_tray(tray)
        .on_system_tray_event(|app, event| match event {
            // 左键点击托盘图标
            SystemTrayEvent::LeftClick { .. } => {
                show_setting_window(app);
            }
            // 菜单点击
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "show_info" => show_setting_window(app),
                "rename" => show_rename_dialog(app),
                "auto_start" => toggle_auto_start(app),
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .setup(|app| {
            // 创建桌面宠物窗口（无边框、透明背景）
            create_cat_window(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_cat_info,
            set_cat_name,
            toggle_auto_start
        ])
        .run(tauri::generate_context!())
        .expect("运行失败");
}

// 创建桌面宠物窗口
fn create_cat_window(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let window = WindowBuilder::new(
        app,
        "cat",
        WindowUrl::App("cat.html".into()),
    )
    .title("喵伴")
    .inner_size(1200.0, 800.0)
    .position(100.0, 50.0)
    .fullscreen(false)
    .decorations(false) // 无边框
    .transparent(true)  // 透明背景
    .always_on_top(true) // 始终在最上层
    .skip_taskbar(true)  // 不在任务栏显示
    .resizable(false)
    .build()?;

    // 设置窗口在桌面工作区（不覆盖其他窗口时可选）
    #[cfg(target_os = "windows")]
    {
        use tauri::utils::config::WindowEffect;
        // Windows 特定设置
    }

    Ok(())
}

// 显示设置窗口
fn show_setting_window(app: &AppHandle) {
    if let Some(window) = app.get_window("setting") {
        window.show().unwrap();
        window.set_focus().unwrap();
    } else {
        // 创建设置窗口
        WindowBuilder::new(
            app,
            "setting",
            WindowUrl::App("setting.html".into()),
        )
        .title("喵伴设置")
        .inner_size(350.0, 500.0)
        .center()
        .decorations(true)
        .transparent(false)
        .build()
        .unwrap();
    }
}

// 显示改名对话框
fn show_rename_dialog(app: &AppHandle) {
    // 通过 JavaScript 对话框实现
    if let Some(window) = app.get_window("cat") {
        window.eval("window.prompt('给你的猫起个名字：')").unwrap();
    }
}

// 切换开机启动
fn toggle_auto_start(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut auto_start = state.auto_start.lock().unwrap();
    *auto_start = !*auto_start;

    #[cfg(target_os = "windows")]
    {
        // Windows 注册表实现开机启动
        use winreg::enums::*;
        use winreg::RegKey;
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        let (key, _) = hkcu.create_subkey(path).unwrap();
        
        if *auto_start {
            let exe_path = std::env::current_exe().unwrap();
            key.set_value("喵伴", &exe_path.to_str().unwrap()).unwrap();
        } else {
            key.delete_value("喵伴").ok();
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS 使用 launch agent
        // 需要创建 plist 文件
    }

    // 更新菜单文本
    let tray_handle = app.tray_handle();
    let label = if *auto_start { "✅ 开机启动" } else { "⚡ 开机启动" };
    tray_handle.get_item("auto_start").set_title(label).unwrap();
}

// Tauri 命令 - 获取猫咪信息
#[tauri::command]
fn get_cat_info(state: tauri::State<AppState>) -> serde_json::Value {
    serde_json::json!({
        "name": *state.cat_name.lock().unwrap(),
        "auto_start": *state.auto_start.lock().unwrap(),
    })
}

// Tauri 命令 - 设置猫咪名字
#[tauri::command]
fn set_cat_name(state: tauri::State<AppState>, name: String) {
    *state.cat_name.lock().unwrap() = name;
}

#[tauri::command]
fn toggle_auto_start_cmd(state: tauri::State<AppState>) -> bool {
    let mut auto_start = state.auto_start.lock().unwrap();
    *auto_start = !*auto_start;
    *auto_start
}