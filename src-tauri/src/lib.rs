mod tray;

use std::{
    collections::{HashMap, HashSet},
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
    time::Duration,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use uuid::Uuid;

#[cfg(target_os = "windows")]
use windows::{
    core::PCWSTR,
    Win32::Graphics::Gdi::{
        DeleteObject, GetDC, ReleaseDC, CreateCompatibleDC, SelectObject, DeleteDC,
        CreateDIBSection, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, GetObjectW,
    },
    Win32::UI::Shell::{SHGetFileInfoW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_USEFILEATTRIBUTES},
    Win32::UI::WindowsAndMessaging::{DestroyIcon, HICON},
    Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
enum ProjectType {
    Rust,
    Nodejs,
    Python,
    Java,
    Go,
    Dotnet,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectMetadata {
    ide_preferences: Vec<String>,
    git_url: Option<String>,
    description: Option<String>,
    #[serde(default)]
    language_stats: Option<LanguageStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LanguageStats {
    total_lines: u64,
    languages: Vec<LanguageEntry>,
    scanned_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LanguageEntry {
    language: String,
    lines: u64,
    files: u32,
    percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Project {
    id: String,
    name: String,
    path: String,
    project_type: ProjectType,
    favorite: bool,
    tags: Vec<String>,
    last_opened: Option<String>,
    #[serde(default)]
    last_modified: Option<String>,
    created_at: String,
    #[serde(default)]
    display_order: i64,
    metadata: ProjectMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
enum IdeCategory {
    Gui,
    Cli,
    Terminal,
    Browser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IdeConfig {
    id: String,
    name: String,
    executable: String,
    args_template: String,
    icon: Option<String>,
    category: IdeCategory,
    priority: i32,
    auto_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct AppStore {
    projects: Vec<Project>,
    ides: Vec<IdeConfig>,
}

struct AppState {
    file_path: PathBuf,
    store: Mutex<AppStore>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewProjectInput {
    name: String,
    path: String,
    project_type: Option<ProjectType>,
    favorite: Option<bool>,
    tags: Option<Vec<String>>,
    description: Option<String>,
    ide_preferences: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewIdeInput {
    name: String,
    executable: String,
    args_template: Option<String>,
    icon: Option<String>,
    category: IdeCategory,
    priority: Option<i32>,
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn file_mtime_iso(path: &str) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let datetime: chrono::DateTime<Utc> = modified.into();
    Some(datetime.to_rfc3339())
}

#[cfg(target_os = "windows")]
fn normalize_windows_path_for_ui(path: &str) -> String {
    if let Some(rest) = path.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{}", rest);
    }
    if let Some(rest) = path.strip_prefix(r"\\?\") {
        return rest.to_string();
    }
    path.to_string()
}

#[cfg(not(target_os = "windows"))]
fn normalize_windows_path_for_ui(path: &str) -> String {
    path.to_string()
}

#[derive(Debug, Clone)]
struct IdeDefinition {
    id: &'static str,
    name: &'static str,
    executable_name: &'static str,
    paths: Vec<&'static str>,
    args_template: &'static str,
    category: IdeCategory,
    priority: i32,
}

fn get_known_ides() -> Vec<IdeDefinition> {
    vec![
        IdeDefinition {
            id: "vscode",
            name: "VSCode",
            executable_name: "Code.exe",
            paths: vec![
                "%LOCALAPPDATA%\\Programs\\Microsoft VS Code\\Code.exe",
                "%USERPROFILE%\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe",
                "C:\\Program Files\\Microsoft VS Code\\Code.exe",
                "C:\\Program Files (x86)\\Microsoft VS Code\\Code.exe",
            ],
            args_template: "{projectPath}",
            category: IdeCategory::Gui,
            priority: 100,
        },
        IdeDefinition {
            id: "cursor",
            name: "Cursor",
            executable_name: "cursor.exe",
            paths: vec![
                "%USERPROFILE%\\AppData\\Local\\cursor\\cursor.exe",
                "%LOCALAPPDATA%\\Programs\\cursor\\cursor.exe",
                "C:\\Program Files\\cursor\\cursor.exe",
            ],
            args_template: "{projectPath}",
            category: IdeCategory::Gui,
            priority: 110,
        },
        IdeDefinition {
            id: "webstorm",
            name: "WebStorm",
            executable_name: "webstorm64.exe",
            paths: vec![
                "%LOCALAPPDATA%\\Programs\\WebStorm\\bin\\webstorm64.exe",
                "C:\\Program Files\\JetBrains\\WebStorm\\bin\\webstorm64.exe",
                "C:\\Program Files (x86)\\JetBrains\\WebStorm\\bin\\webstorm64.exe",
            ],
            args_template: "{projectPath}",
            category: IdeCategory::Gui,
            priority: 120,
        },
        IdeDefinition {
            id: "intellij",
            name: "IntelliJ IDEA",
            executable_name: "idea64.exe",
            paths: vec![
                "%LOCALAPPDATA%\\Programs\\IntelliJ IDEA\\bin\\idea64.exe",
                "C:\\Program Files\\JetBrains\\IntelliJ IDEA\\bin\\idea64.exe",
                "C:\\Program Files (x86)\\JetBrains\\IntelliJ IDEA\\bin\\idea64.exe",
            ],
            args_template: "{projectPath}",
            category: IdeCategory::Gui,
            priority: 121,
        },
        IdeDefinition {
            id: "pycharm",
            name: "PyCharm",
            executable_name: "pycharm64.exe",
            paths: vec![
                "%LOCALAPPDATA%\\Programs\\PyCharm\\bin\\pycharm64.exe",
                "C:\\Program Files\\JetBrains\\PyCharm\\bin\\pycharm64.exe",
                "C:\\Program Files (x86)\\JetBrains\\PyCharm\\bin\\pycharm64.exe",
            ],
            args_template: "{projectPath}",
            category: IdeCategory::Gui,
            priority: 122,
        },
        IdeDefinition {
            id: "clion",
            name: "CLion",
            executable_name: "clion64.exe",
            paths: vec![
                "%LOCALAPPDATA%\\Programs\\CLion\\bin\\clion64.exe",
                "C:\\Program Files\\JetBrains\\CLion\\bin\\clion64.exe",
                "C:\\Program Files (x86)\\JetBrains\\CLion\\bin\\clion64.exe",
            ],
            args_template: "{projectPath}",
            category: IdeCategory::Gui,
            priority: 123,
        },
        IdeDefinition {
            id: "goland",
            name: "GoLand",
            executable_name: "goland64.exe",
            paths: vec![
                "%LOCALAPPDATA%\\Programs\\GoLand\\bin\\goland64.exe",
                "C:\\Program Files\\JetBrains\\GoLand\\bin\\goland64.exe",
                "C:\\Program Files (x86)\\JetBrains\\GoLand\\bin\\goland64.exe",
            ],
            args_template: "{projectPath}",
            category: IdeCategory::Gui,
            priority: 124,
        },
        IdeDefinition {
            id: "rider",
            name: "Rider",
            executable_name: "rider64.exe",
            paths: vec![
                "%LOCALAPPDATA%\\Programs\\JetBrains\\Rider\\bin\\rider64.exe",
                "C:\\Program Files\\JetBrains\\Rider\\bin\\rider64.exe",
                "C:\\Program Files (x86)\\JetBrains\\Rider\\bin\\rider64.exe",
            ],
            args_template: "{projectPath}",
            category: IdeCategory::Gui,
            priority: 125,
        },
        IdeDefinition {
            id: "fleet",
            name: "Fleet",
            executable_name: "fleet.exe",
            paths: vec![
                "%LOCALAPPDATA%\\Programs\\Fleet\\bin\\fleet.exe",
                "C:\\Program Files\\JetBrains\\Fleet\\bin\\fleet.exe",
            ],
            args_template: "{projectPath}",
            category: IdeCategory::Gui,
            priority: 126,
        },
        IdeDefinition {
            id: "android-studio",
            name: "Android Studio",
            executable_name: "studio64.exe",
            paths: vec![
                "%LOCALAPPDATA%\\Android\\android-studio\\bin\\studio64.exe",
                "C:\\Program Files\\Android\\Android Studio\\bin\\studio64.exe",
                "C:\\Program Files (x86)\\Android\\Android Studio\\bin\\studio64.exe",
            ],
            args_template: "{projectPath}",
            category: IdeCategory::Gui,
            priority: 127,
        },
        IdeDefinition {
            id: "neovim",
            name: "Neovim",
            executable_name: "nvim",
            paths: vec![
                "%LOCALAPPDATA%\\nvim\\bin\\nvim.exe",
                "C:\\Program Files\\Neovim\\bin\\nvim.exe",
                "C:\\tools\\neovim\\bin\\nvim.exe",
            ],
            args_template: "{projectPath}",
            category: IdeCategory::Cli,
            priority: 200,
        },
        IdeDefinition {
            id: "vim",
            name: "Vim",
            executable_name: "vim",
            paths: vec![
                "C:\\Program Files\\Vim\\vim90\\vim.exe",
                "C:\\Program Files (x86)\\Vim\\vim90\\vim.exe",
            ],
            args_template: "{projectPath}",
            category: IdeCategory::Cli,
            priority: 201,
        },
        IdeDefinition {
            id: "claude",
            name: "Claude CLI",
            executable_name: "claude",
            paths: vec![],
            args_template: "",
            category: IdeCategory::Cli,
            priority: 210,
        },
        IdeDefinition {
            id: "codex",
            name: "Codex CLI",
            executable_name: "codex",
            paths: vec![],
            args_template: "",
            category: IdeCategory::Cli,
            priority: 211,
        },
        IdeDefinition {
            id: "opencode",
            name: "OpenCode CLI",
            executable_name: "opencode",
            paths: vec![],
            args_template: "",
            category: IdeCategory::Cli,
            priority: 212,
        },
    ]
}

fn expand_env_path(path: &str) -> Option<String> {
    let mut result = path.to_string();

    // 手动扩展环境变量
    if result.contains("%LOCALAPPDATA%") {
        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            result = result.replace("%LOCALAPPDATA%", &local_app_data);
        }
    }
    if result.contains("%USERPROFILE%") {
        if let Ok(user_profile) = env::var("USERPROFILE") {
            result = result.replace("%USERPROFILE%", &user_profile);
        }
    }
    if result.contains("%APPDATA%") {
        if let Ok(app_data) = env::var("APPDATA") {
            result = result.replace("%APPDATA%", &app_data);
        }
    }

    Some(result)
}

fn find_executable_in_known_paths(paths: &[&str]) -> Option<PathBuf> {
    paths
        .iter()
        .filter_map(|p| expand_env_path(p))
        .map(PathBuf::from)
        .find(|p| p.exists())
}

#[cfg(target_os = "windows")]
fn find_executable_in_path(command_name: &str) -> Option<PathBuf> {
    let mut candidates = vec![command_name.to_string()];
    if command_name.ends_with(".exe") {
        candidates.push(command_name.trim_end_matches(".exe").to_string());
    } else if !command_name.contains('.') {
        candidates.push(format!("{command_name}.exe"));
        candidates.push(format!("{command_name}.cmd"));
        candidates.push(format!("{command_name}.bat"));
    }

    for candidate in candidates {
        let output = Command::new("where.exe").arg(&candidate).output().ok()?;
        if !output.status.success() {
            continue;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
        if let Some(best) = lines.iter().find(|line| {
            let lower = line.to_ascii_lowercase();
            lower.ends_with(".cmd") || lower.ends_with(".exe") || lower.ends_with(".bat")
        }) {
            return Some(PathBuf::from(best));
        }
        if let Some(first) = lines.first() {
            return Some(PathBuf::from(first));
        }
    }

    None
}

#[cfg(not(target_os = "windows"))]
fn find_executable_in_path(command_name: &str) -> Option<PathBuf> {
    let output = Command::new("which").arg(command_name).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
}

fn resolve_ide_executable(ide_def: &IdeDefinition) -> Option<PathBuf> {
    find_executable_in_known_paths(&ide_def.paths)
        .or_else(|| find_executable_in_path(ide_def.executable_name))
}

#[cfg(target_os = "windows")]
fn resolve_icon_source_path(executable_path: &Path, executable_name: &str) -> PathBuf {
    let ext = executable_path
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    // 对脚本 shim（.cmd/.bat/.ps1）优先尝试找到同名 .exe 作为图标来源
    if matches!(ext.as_str(), "cmd" | "bat" | "ps1") {
        if let (Some(parent), Some(stem)) = (executable_path.parent(), executable_path.file_stem()) {
            let sibling = parent.join(format!("{}.exe", stem.to_string_lossy()));
            if sibling.exists() {
                return sibling;
            }
        }
        let normalized = executable_name.trim_end_matches(".exe");
        if let Some(path) = find_executable_in_path(&format!("{normalized}.exe")) {
            return path;
        }
    }

    executable_path.to_path_buf()
}

#[cfg(not(target_os = "windows"))]
fn resolve_icon_source_path(executable_path: &Path, _executable_name: &str) -> PathBuf {
    executable_path.to_path_buf()
}

#[cfg(target_os = "windows")]
fn extract_icon_from_exe(exe_path: &Path) -> Option<String> {
    let path_str = exe_path.to_string_lossy().to_string();
    let path_wide: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe fn load_hicon(path_wide: &[u16], use_file_attributes: bool) -> Option<HICON> {
        use windows::Win32::UI::Shell::SHFILEINFOW;
        let mut shfi = SHFILEINFOW::default();
        let mut flags = SHGFI_ICON | SHGFI_LARGEICON;
        if use_file_attributes {
            flags |= SHGFI_USEFILEATTRIBUTES;
        }
        let result = SHGetFileInfoW(
            PCWSTR(path_wide.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            flags,
        );
        if result == 0 || shfi.hIcon == HICON::default() {
            return None;
        }
        Some(shfi.hIcon)
    }

    unsafe {
        // 1) 优先取真实文件图标；2) 再回退文件类型关联图标
        let hicon = if exe_path.exists() {
            load_hicon(&path_wide, false).or_else(|| load_hicon(&path_wide, true))
        } else {
            load_hicon(&path_wide, true)
        }?;

        let icon = extract_hicon_to_png(hicon)?;
        let _ = DestroyIcon(hicon);
        Some(format!("data:image/png;extraction=v3;base64,{}", icon))
    }
}

#[cfg(target_os = "windows")]
unsafe fn extract_hicon_to_png(hicon: HICON) -> Option<String> {
    use image::ImageEncoder;
    use image::codecs::png::PngEncoder;
    use std::ptr::null_mut;

    let hdc = GetDC(None);
    if hdc.is_invalid() {
        return None;
    }

    let mem_dc = CreateCompatibleDC(hdc);
    if mem_dc.is_invalid() {
        ReleaseDC(None, hdc);
        return None;
    }

    let mut icon_info = windows::Win32::UI::WindowsAndMessaging::ICONINFO::default();
    if windows::Win32::UI::WindowsAndMessaging::GetIconInfo(hicon, &mut icon_info).is_ok() {
        let (width, height) = icon_dimensions_from_info(&icon_info);

        let mut ppv_bits: *mut std::ffi::c_void = null_mut();

        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                // 目标 DIB 本身不是 icon-mask，不能把高度翻倍；负值表示 top-down。
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [Default::default()],
        };

        let hbitmap = match CreateDIBSection(
            mem_dc,
            &bmi,
            DIB_RGB_COLORS,
            &mut ppv_bits,
            None,
            0,
        ) {
            Ok(v) => v,
            Err(_) => {
                if !icon_info.hbmColor.is_invalid() {
                    let _ = DeleteObject(icon_info.hbmColor);
                }
                if !icon_info.hbmMask.is_invalid() {
                    let _ = DeleteObject(icon_info.hbmMask);
                }
                let _ = DeleteDC(mem_dc);
                ReleaseDC(None, hdc);
                return None;
            }
        };

        if ppv_bits.is_null() || width <= 0 || height <= 0 {
            let _ = DeleteObject(hbitmap);
            if !icon_info.hbmColor.is_invalid() {
                let _ = DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_invalid() {
                let _ = DeleteObject(icon_info.hbmMask);
            }
            let _ = DeleteDC(mem_dc);
            ReleaseDC(None, hdc);
            return None;
        }

        let old_bitmap = SelectObject(mem_dc, hbitmap);
        let _ = windows::Win32::UI::WindowsAndMessaging::DrawIconEx(
            mem_dc,
            0,
            0,
            hicon,
            width,
            height,
            0,
            None,
            windows::Win32::UI::WindowsAndMessaging::DI_NORMAL,
        );

        // 先复制位图内存，再释放 GDI 对象，避免 UAF 崩溃
        let pixels_len = (width * height) as usize;
        let pixels_slice = std::slice::from_raw_parts(ppv_bits as *const u32, pixels_len);
        let mut rgba_pixels: Vec<u8> = Vec::with_capacity(pixels_len * 4);
        let mut has_alpha = false;
        for &pixel in pixels_slice {
            let b = (pixel & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let r = ((pixel >> 16) & 0xFF) as u8;
            let a = ((pixel >> 24) & 0xFF) as u8;
            if a > 0 {
                has_alpha = true;
            }
            rgba_pixels.push(r);
            rgba_pixels.push(g);
            rgba_pixels.push(b);
            rgba_pixels.push(a);
        }

        // 无 alpha 时，按 AND mask 恢复透明度（1=透明，0=不透明）。
        if !has_alpha {
            if let Some(mask_alpha) = build_alpha_from_icon_mask(hicon, width, height) {
                for (idx, chunk) in rgba_pixels.chunks_mut(4).enumerate() {
                    chunk[3] = mask_alpha[idx];
                }
            } else {
                for chunk in rgba_pixels.chunks_mut(4) {
                    // 最后兜底：仅在像素不是纯黑时设为不透明，避免整图黑块。
                    chunk[3] = if chunk[0] == 0 && chunk[1] == 0 && chunk[2] == 0 { 0 } else { 255 };
                }
            }
        } else {
            // DrawIconEx/AlphaBlend 路径常见预乘 alpha，导出 PNG 需要转回直通 alpha，避免发黑。
            for chunk in rgba_pixels.chunks_mut(4) {
                let a = chunk[3] as u32;
                if a == 0 || a == 255 {
                    continue;
                }
                chunk[0] = ((chunk[0] as u32 * 255 + a / 2) / a).min(255) as u8;
                chunk[1] = ((chunk[1] as u32 * 255 + a / 2) / a).min(255) as u8;
                chunk[2] = ((chunk[2] as u32 * 255 + a / 2) / a).min(255) as u8;
            }
        }

        let _ = SelectObject(mem_dc, old_bitmap);
        let _ = DeleteObject(hbitmap);
        if !icon_info.hbmColor.is_invalid() {
            let _ = DeleteObject(icon_info.hbmColor);
        }
        if !icon_info.hbmMask.is_invalid() {
            let _ = DeleteObject(icon_info.hbmMask);
        }

        let mut png_bytes = Vec::new();
        let encoder = PngEncoder::new(&mut png_bytes);
        if encoder
            .write_image(
                &rgba_pixels,
                width as u32,
                height as u32,
                image::ExtendedColorType::Rgba8,
            )
            .is_ok()
        {
            let _ = DeleteDC(mem_dc);
            ReleaseDC(None, hdc);
            use base64::Engine;
            return Some(base64::engine::general_purpose::STANDARD.encode(&png_bytes));
        }
    }

    let _ = DeleteDC(mem_dc);
    ReleaseDC(None, hdc);
    None
}

#[cfg(target_os = "windows")]
fn icon_dimensions_from_info(
    icon_info: &windows::Win32::UI::WindowsAndMessaging::ICONINFO,
) -> (i32, i32) {
    unsafe {
        let mut bmp = BITMAP::default();

        if !icon_info.hbmColor.is_invalid() {
            let got = GetObjectW(
                icon_info.hbmColor,
                std::mem::size_of::<BITMAP>() as i32,
                Some((&mut bmp as *mut BITMAP).cast()),
            );
            if got > 0 && bmp.bmWidth > 0 && bmp.bmHeight > 0 {
                return (bmp.bmWidth.clamp(1, 512), bmp.bmHeight.clamp(1, 512));
            }
        }

        if !icon_info.hbmMask.is_invalid() {
            let got = GetObjectW(
                icon_info.hbmMask,
                std::mem::size_of::<BITMAP>() as i32,
                Some((&mut bmp as *mut BITMAP).cast()),
            );
            if got > 0 && bmp.bmWidth > 0 && bmp.bmHeight > 0 {
                let mut h = bmp.bmHeight;
                if icon_info.hbmColor.is_invalid() {
                    // 单色 icon: hbmMask 高度包含 AND/XOR 两段。
                    h /= 2;
                }
                return (bmp.bmWidth.clamp(1, 512), h.clamp(1, 512));
            }
        }
    }

    (32, 32)
}

#[cfg(target_os = "windows")]
fn build_alpha_from_icon_mask(hicon: HICON, width: i32, height: i32) -> Option<Vec<u8>> {
    unsafe {
        use std::ptr::null_mut;
        let hdc = GetDC(None);
        if hdc.is_invalid() {
            return None;
        }

        let dc = CreateCompatibleDC(hdc);
        if dc.is_invalid() {
            ReleaseDC(None, hdc);
            return None;
        }

        let mut bits_ptr: *mut std::ffi::c_void = null_mut();
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [Default::default()],
        };

        let dib = match CreateDIBSection(dc, &bmi, DIB_RGB_COLORS, &mut bits_ptr, None, 0) {
            Ok(v) => v,
            Err(_) => {
                let _ = DeleteDC(dc);
                ReleaseDC(None, hdc);
                return None;
            }
        };

        if bits_ptr.is_null() {
            let _ = DeleteObject(dib);
            let _ = DeleteDC(dc);
            ReleaseDC(None, hdc);
            return None;
        }

        // 先填白，DI_MASK 绘制后透明区域保持白色，实像素区域为黑色。
        let pixels_len = (width * height) as usize;
        let mask_pixels = std::slice::from_raw_parts_mut(bits_ptr as *mut u32, pixels_len);
        for px in mask_pixels.iter_mut() {
            *px = 0xFFFFFFFF;
        }

        let old = SelectObject(dc, dib);
        let _ = windows::Win32::UI::WindowsAndMessaging::DrawIconEx(
            dc,
            0,
            0,
            hicon,
            width,
            height,
            0,
            None,
            windows::Win32::UI::WindowsAndMessaging::DI_MASK,
        );

        let mut alpha = Vec::with_capacity(pixels_len);
        for &pixel in mask_pixels.iter() {
            let b = (pixel & 0xFF) as u8;
            alpha.push(if b > 127 { 0 } else { 255 });
        }

        let _ = SelectObject(dc, old);
        let _ = DeleteObject(dib);
        let _ = DeleteDC(dc);
        ReleaseDC(None, hdc);
        Some(alpha)
    }
}

#[cfg(target_os = "windows")]
fn is_cached_v3_icon(icon: &str) -> bool {
    icon.starts_with("data:image/png;extraction=v3;base64,")
}

#[cfg(not(target_os = "windows"))]
fn is_cached_v3_icon(_icon: &str) -> bool {
    true
}

#[cfg(not(target_os = "windows"))]
fn extract_icon_from_exe(_exe_path: &Path) -> Option<String> {
    None
}

fn default_ides() -> Vec<IdeConfig> {
    vec![
        IdeConfig {
            id: "vscode".to_string(),
            name: "VSCode".to_string(),
            executable: "code".to_string(),
            args_template: "{projectPath}".to_string(),
            icon: None,
            category: IdeCategory::Gui,
            priority: 100,
            auto_detected: false,
        },
        IdeConfig {
            id: "cursor".to_string(),
            name: "Cursor".to_string(),
            executable: "cursor".to_string(),
            args_template: "{projectPath}".to_string(),
            icon: None,
            category: IdeCategory::Gui,
            priority: 110,
            auto_detected: false,
        },
    ]
}

fn ide_icon_cache_dir(store_file_path: &Path) -> PathBuf {
    store_file_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("ide-icons")
}

fn decode_ide_icon_cache_mime(path: &Path) -> &'static str {
    match path.extension().and_then(|v| v.to_str()).map(|s| s.to_ascii_lowercase()) {
        Some(ext) if ext == "svg" => "image/svg+xml",
        Some(ext) if ext == "ico" => "image/x-icon",
        Some(ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg",
        Some(ext) if ext == "webp" => "image/webp",
        _ => "image/png",
    }
}

fn image_mime_by_extension(path: &Path) -> Option<&'static str> {
    match path
        .extension()
        .and_then(|v| v.to_str())
        .map(|s| s.to_ascii_lowercase())
    {
        Some(ext) if ext == "png" => Some("image/png"),
        Some(ext) if ext == "svg" => Some("image/svg+xml"),
        Some(ext) if ext == "ico" => Some("image/x-icon"),
        Some(ext) if ext == "jpg" || ext == "jpeg" => Some("image/jpeg"),
        Some(ext) if ext == "webp" => Some("image/webp"),
        _ => None,
    }
}

fn image_file_to_data_url(path: &Path) -> Result<String, String> {
    let mime = image_mime_by_extension(path)
        .ok_or_else(|| "仅支持 png/svg/ico/jpg/webp 图标文件，或 exe/cmd/bat/ps1 可执行文件".to_string())?;
    let bytes = fs::read(path).map_err(|e| format!("读取图标文件失败: {e}"))?;
    if bytes.is_empty() {
        return Err("图标文件为空".to_string());
    }
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("图标文件过大（限制 2MB）".to_string());
    }
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:{mime};source=user-file-v1;base64,{encoded}"))
}

fn icon_data_url_from_user_file(path: &Path) -> Result<String, String> {
    let ext = path
        .extension()
        .and_then(|v| v.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    if matches!(ext.as_str(), "exe" | "cmd" | "bat" | "ps1") {
        let display_name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("app.exe");
        let source = resolve_icon_source_path(path, display_name);
        return extract_icon_from_exe(&source).ok_or_else(|| "从可执行文件提取图标失败".to_string());
    }

    image_file_to_data_url(path)
}

fn load_cached_ide_icon(store_file_path: &Path, ide_id: &str) -> Option<String> {
    let cache_dir = ide_icon_cache_dir(store_file_path);
    let candidates = [
        cache_dir.join(format!("{ide_id}.svg")),
        cache_dir.join(format!("{ide_id}.png")),
        cache_dir.join(format!("{ide_id}.ico")),
        cache_dir.join(format!("{ide_id}.webp")),
        cache_dir.join(format!("{ide_id}.jpg")),
    ];

    for path in candidates {
        let bytes = match fs::read(&path) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if bytes.is_empty() {
            continue;
        }
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        let mime = decode_ide_icon_cache_mime(&path);
        return Some(format!("data:{mime};source=web-cache-v1;base64,{encoded}"));
    }

    None
}

fn online_icon_urls_for_ide(ide: &IdeConfig) -> Vec<&'static str> {
    let id = ide.id.to_ascii_lowercase();
    let name = ide.name.to_ascii_lowercase();
    let executable = ide.executable.to_ascii_lowercase();
    let merged = format!("{id} {name} {executable}");

    if merged.contains("vscode") || merged.contains("visual studio code") || merged.contains("code.exe") {
        return vec![
            "https://code.visualstudio.com/favicon.ico",
            "https://code.visualstudio.com/assets/images/code-stable.png",
        ];
    }
    if merged.contains("cursor") {
        return vec![
            "https://cursor.com/favicon.ico",
            "https://www.cursor.com/favicon.ico",
        ];
    }
    if merged.contains("claude") {
        return vec![
            "https://claude.ai/favicon.ico",
            "https://www.anthropic.com/favicon.ico",
        ];
    }
    if merged.contains("opencode") {
        return vec![
            "https://opencode.ai/favicon.ico",
            "https://github.com/sst/opencode/raw/dev/packages/web/public/favicon.ico",
        ];
    }
    if merged.contains("codex") || merged.contains("openai") {
        return vec![
            "https://openai.com/favicon.ico",
            "https://chatgpt.com/favicon.ico",
        ];
    }

    vec![]
}

fn guess_icon_ext_by_content_type(content_type: &str) -> &'static str {
    let value = content_type.to_ascii_lowercase();
    if value.contains("image/svg+xml") {
        "svg"
    } else if value.contains("image/x-icon") || value.contains("image/vnd.microsoft.icon") {
        "ico"
    } else if value.contains("image/webp") {
        "webp"
    } else if value.contains("image/jpeg") || value.contains("image/jpg") {
        "jpg"
    } else {
        "png"
    }
}

fn download_and_cache_ide_icon(store_file_path: &Path, ide: &IdeConfig) -> Option<String> {
    let urls = online_icon_urls_for_ide(ide);
    if urls.is_empty() {
        return None;
    }

    let cache_dir = ide_icon_cache_dir(store_file_path);
    let _ = fs::create_dir_all(&cache_dir);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(6))
        .user_agent("dev-boom/0.1 ide-icon-fetch")
        .build()
        .ok()?;

    for url in urls {
        let response = match client.get(url).send() {
            Ok(v) => v,
            Err(_) => continue,
        };
        if !response.status().is_success() {
            continue;
        }
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/png")
            .to_string();
        if !content_type.to_ascii_lowercase().contains("image/") {
            continue;
        }
        let bytes = match response.bytes() {
            Ok(v) => v,
            Err(_) => continue,
        };
        if bytes.is_empty() || bytes.len() > 2 * 1024 * 1024 {
            continue;
        }

        let ext = guess_icon_ext_by_content_type(&content_type);
        let cache_path = cache_dir.join(format!("{}.{}", ide.id, ext));
        let _ = fs::write(&cache_path, &bytes);

        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        return Some(format!("data:{};source=web-v1;base64,{}", decode_ide_icon_cache_mime(&cache_path), encoded));
    }

    None
}

fn resolve_ide_icon(store_file_path: &Path, ide: &IdeConfig) -> Option<String> {
    let resolved = PathBuf::from(&ide.executable);
    if resolved.exists() {
        let source = resolve_icon_source_path(&resolved, &ide.executable);
        if let Some(icon) = extract_icon_from_exe(&source) {
            return Some(icon);
        }
    } else if let Some(path) = find_executable_in_path(&ide.executable) {
        let source = resolve_icon_source_path(&path, &ide.executable);
        if let Some(icon) = extract_icon_from_exe(&source) {
            return Some(icon);
        }
    }

    load_cached_ide_icon(store_file_path, &ide.id)
        .or_else(|| download_and_cache_ide_icon(store_file_path, ide))
}

fn load_store(path: &Path) -> AppStore {
    if !path.exists() {
        return AppStore {
            projects: vec![],
            ides: default_ides(),
        };
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            return AppStore {
                projects: vec![],
                ides: default_ides(),
            };
        }
    };

    match serde_json::from_str::<AppStore>(&content) {
        Ok(mut store) => {
            if store.ides.is_empty() {
                store.ides = default_ides();
            }
            for (idx, project) in store.projects.iter_mut().enumerate() {
                project.path = normalize_windows_path_for_ui(&project.path);
                if project.display_order == 0 {
                    project.display_order = idx as i64 + 1;
                }
            }
            for ide in &mut store.ides {
                ide.executable = normalize_windows_path_for_ui(&ide.executable);
            }
            store
        }
        Err(_) => AppStore {
            projects: vec![],
            ides: default_ides(),
        },
    }
}

fn save_store(path: &Path, store: &AppStore) -> Result<(), String> {
    let content = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

fn detect_project_type(path: &Path) -> ProjectType {
    if path.join("Cargo.toml").exists() {
        return ProjectType::Rust;
    }
    if path.join("package.json").exists() {
        return ProjectType::Nodejs;
    }
    if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
        return ProjectType::Python;
    }
    if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
        return ProjectType::Java;
    }
    if path.join("go.mod").exists() {
        return ProjectType::Go;
    }
    let has_dotnet_project = fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| {
            entry
                .path()
                .extension()
                .and_then(|v| v.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("sln") || ext.eq_ignore_ascii_case("csproj"))
                .unwrap_or(false)
        });
    if has_dotnet_project {
        return ProjectType::Dotnet;
    }

    ProjectType::Generic
}

fn is_project_root(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
        || path.join("package.json").exists()
        || path.join("requirements.txt").exists()
        || path.join("pyproject.toml").exists()
        || path.join("go.mod").exists()
        || path.join("pom.xml").exists()
        || path.join("build.gradle").exists()
        || path.join(".git").exists()
}

fn should_skip_dir(path: &Path) -> bool {
    let skip = [
        ".git",
        "node_modules",
        "target",
        ".venv",
        "venv",
        ".idea",
        ".vscode",
    ];
    match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => skip.contains(&name),
        None => false,
    }
}

fn scan_projects_rec(path: &Path, current_depth: u8, max_depth: u8, out: &mut Vec<PathBuf>) {
    if current_depth > max_depth || should_skip_dir(path) {
        return;
    }

    if is_project_root(path) {
        out.push(path.to_path_buf());
        return;
    }

    let entries = match fs::read_dir(path) {
        Ok(v) => v,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let child = entry.path();
        if child.is_dir() {
            scan_projects_rec(&child, current_depth + 1, max_depth, out);
        }
    }
}

fn split_args_template(args_template: &str) -> Vec<String> {
    let trimmed = args_template.trim();
    if trimmed.is_empty() {
        return vec![];
    }
    if let Some(parts) = shlex::split(trimmed) {
        if !parts.is_empty() {
            return parts;
        }
    }
    vec![trimmed.to_string()]
}

fn expand_args(args_template: &str, project: &Project) -> Vec<String> {
    // 先拆模板、后替换占位符，避免 Windows 路径中的反斜杠被 shell 解析破坏。
    split_args_template(args_template)
        .into_iter()
        .map(|arg| {
            arg.replace("{projectPath}", &project.path)
                .replace("{projectName}", &project.name)
        })
        .collect()
}

#[tauri::command]
fn get_projects(state: State<'_, AppState>) -> Vec<Project> {
    let mut store = state.store.lock().expect("store lock poisoned");
    for project in &mut store.projects {
        project.last_modified = file_mtime_iso(&project.path);
    }
    let mut projects = store.projects.clone();
    projects.sort_by(|a, b| b.last_modified.cmp(&a.last_modified).then_with(|| a.name.cmp(&b.name)));
    projects
}

#[tauri::command]
fn get_ides(state: State<'_, AppState>) -> Vec<IdeConfig> {
    let mut store = state.store.lock().expect("store lock poisoned");
    let mut dirty = false;
    for ide in &mut store.ides {
        let should_refresh_icon = match ide.icon.as_deref() {
            None => true,
            Some(icon) => icon.starts_with("data:image/png") && !is_cached_v3_icon(icon),
        };
        if !should_refresh_icon {
            continue;
        }
        let icon = resolve_ide_icon(&state.file_path, ide);
        if icon.is_some() {
            ide.icon = icon;
            dirty = true;
        }
    }
    if dirty {
        let _ = save_store(&state.file_path, &store);
    }
    let mut ides = store.ides.clone();
    ides.sort_by_key(|x| x.priority);
    ides
}

#[tauri::command]
fn add_project(input: NewProjectInput, state: State<'_, AppState>) -> Result<Project, String> {
    let path = PathBuf::from(&input.path);
    if !path.exists() || !path.is_dir() {
        return Err("项目路径不存在或不是目录".to_string());
    }

    let normalized_path = path
        .canonicalize()
        .map_err(|e| format!("无法读取项目路径: {e}"))?
        .to_string_lossy()
        .to_string();
    let normalized_path = normalize_windows_path_for_ui(&normalized_path);

    let mut store = state.store.lock().expect("store lock poisoned");
    if store.projects.iter().any(|p| p.path == normalized_path) {
        return Err("该项目路径已存在".to_string());
    }

    // 自动统计语言分布
    let language_stats = scan_project_languages(&path).ok().map(|lang_data| {
        let total_lines: u64 = lang_data.values().map(|(lines, _)| *lines).sum();
        let mut languages: Vec<LanguageEntry> = lang_data
            .into_iter()
            .map(|(language, (lines, files))| {
                let percentage = if total_lines > 0 {
                    (lines as f64 / total_lines as f64) * 100.0
                } else {
                    0.0
                };
                LanguageEntry {
                    language,
                    lines,
                    files,
                    percentage,
                }
            })
            .collect();
        languages.sort_by(|a, b| b.lines.cmp(&a.lines));
        LanguageStats {
            total_lines,
            languages,
            scanned_at: now_iso(),
        }
    });

    let created = Project {
        id: Uuid::new_v4().to_string(),
        name: if input.name.trim().is_empty() {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("untitled")
                .to_string()
        } else {
            input.name.trim().to_string()
        },
        path: normalized_path.clone(),
        project_type: input
            .project_type
            .unwrap_or_else(|| detect_project_type(&path)),
        favorite: input.favorite.unwrap_or(false),
        tags: input.tags.unwrap_or_default(),
        last_opened: None,
        last_modified: file_mtime_iso(&normalized_path),
        created_at: now_iso(),
        display_order: store
            .projects
            .iter()
            .map(|p| p.display_order)
            .max()
            .unwrap_or(0)
            + 1,
        metadata: ProjectMetadata {
            ide_preferences: input.ide_preferences.unwrap_or_default(),
            git_url: None,
            description: input.description,
            language_stats,
        },
    };

    store.projects.push(created.clone());
    save_store(&state.file_path, &store)?;
    Ok(created)
}

#[tauri::command]
fn remove_project(project_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut store = state.store.lock().expect("store lock poisoned");
    let before = store.projects.len();
    store.projects.retain(|p| p.id != project_id);
    if store.projects.len() == before {
        return Err("项目不存在".to_string());
    }
    save_store(&state.file_path, &store)
}

#[tauri::command]
fn toggle_project_favorite(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let mut store = state.store.lock().expect("store lock poisoned");
    let project = store
        .projects
        .iter_mut()
        .find(|p| p.id == project_id)
        .ok_or_else(|| "项目不存在".to_string())?;
    project.favorite = !project.favorite;
    let result = project.clone();
    save_store(&state.file_path, &store)?;
    Ok(result)
}

#[tauri::command]
fn scan_projects(
    root_path: String,
    max_depth: Option<u8>,
    state: State<'_, AppState>,
) -> Result<Vec<Project>, String> {
    let root = PathBuf::from(root_path);
    if !root.exists() || !root.is_dir() {
        return Err("扫描路径不存在或不是目录".to_string());
    }

    let mut found_paths = vec![];
    scan_projects_rec(&root, 0, max_depth.unwrap_or(3), &mut found_paths);

    let mut store = state.store.lock().expect("store lock poisoned");
    let mut existing_paths: HashSet<String> =
        store.projects.iter().map(|p| p.path.clone()).collect();
    let mut added = vec![];

    let mut next_order = store
        .projects
        .iter()
        .map(|p| p.display_order)
        .max()
        .unwrap_or(0)
        + 1;

    for item in found_paths {
        let canonical = match item.canonicalize() {
            Ok(v) => normalize_windows_path_for_ui(&v.to_string_lossy()),
            Err(_) => continue,
        };

        // 检查项目是否已存在
        let is_new = !existing_paths.contains(&canonical);
        if is_new {
            existing_paths.insert(canonical.clone());
        }

        // 自动统计语言分布（新项目和已有项目都更新）
        let language_stats = scan_project_languages(&item).ok().map(|lang_data| {
            let total_lines: u64 = lang_data.values().map(|(lines, _)| *lines).sum();
            let mut languages: Vec<LanguageEntry> = lang_data
                .into_iter()
                .map(|(language, (lines, files))| {
                    let percentage = if total_lines > 0 {
                        (lines as f64 / total_lines as f64) * 100.0
                    } else {
                        0.0
                    };
                    LanguageEntry {
                        language,
                        lines,
                        files,
                        percentage,
                    }
                })
                .collect();
            languages.sort_by(|a, b| b.lines.cmp(&a.lines));
            LanguageStats {
                total_lines,
                languages,
                scanned_at: now_iso(),
            }
        });

        if is_new {
            // 创建新项目
            let project = Project {
                id: Uuid::new_v4().to_string(),
                name: item
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("untitled")
                    .to_string(),
                path: canonical.clone(),
                project_type: detect_project_type(&item),
                favorite: false,
                tags: vec![],
                last_opened: None,
                last_modified: file_mtime_iso(&canonical),
                created_at: now_iso(),
                display_order: next_order,
                metadata: ProjectMetadata {
                    ide_preferences: vec![],
                    git_url: None,
                    description: None,
                    language_stats,
                },
            };
            next_order += 1;
            store.projects.push(project.clone());
            added.push(project);
        } else {
            // 更新已有项目的语言统计
            if let Some(project) = store.projects.iter_mut().find(|p| p.path == canonical) {
                project.metadata.language_stats = language_stats;
                added.push(project.clone());
            }
        }
    }

    if !added.is_empty() {
        save_store(&state.file_path, &store)?;
    }
    Ok(added)
}

#[tauri::command]
fn add_ide(input: NewIdeInput, state: State<'_, AppState>) -> Result<IdeConfig, String> {
    if input.name.trim().is_empty() {
        return Err("IDE 名称不能为空".to_string());
    }
    if input.executable.trim().is_empty() {
        return Err("可执行文件不能为空".to_string());
    }

    let mut store = state.store.lock().expect("store lock poisoned");
    let ide = IdeConfig {
        id: Uuid::new_v4().to_string(),
        name: input.name.trim().to_string(),
        executable: input.executable.trim().to_string(),
        args_template: input
            .args_template
            .unwrap_or_else(|| "{projectPath}".to_string()),
        icon: input.icon,
        category: input.category,
        priority: input.priority.unwrap_or(200),
        auto_detected: false,
    };
    store.ides.push(ide.clone());
    save_store(&state.file_path, &store)?;
    Ok(ide)
}

#[tauri::command]
fn remove_ide(ide_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut store = state.store.lock().expect("store lock poisoned");
    let before = store.ides.len();
    store.ides.retain(|x| x.id != ide_id);
    if store.ides.len() == before {
        return Err("IDE 不存在".to_string());
    }

    for project in &mut store.projects {
        project.metadata.ide_preferences.retain(|x| x != &ide_id);
    }

    save_store(&state.file_path, &store)
}

#[tauri::command]
fn set_ide_icon_from_file(
    ide_id: String,
    file_path: String,
    state: State<'_, AppState>,
) -> Result<IdeConfig, String> {
    let path = PathBuf::from(file_path);
    if !path.exists() || !path.is_file() {
        return Err("图标文件不存在".to_string());
    }

    let icon_data_url = icon_data_url_from_user_file(&path)?;
    let mut store = state.store.lock().expect("store lock poisoned");
    let ide = store
        .ides
        .iter_mut()
        .find(|x| x.id == ide_id)
        .ok_or_else(|| "IDE 不存在".to_string())?;
    ide.icon = Some(icon_data_url);
    let updated = ide.clone();
    save_store(&state.file_path, &store)?;
    Ok(updated)
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn scan_ides(state: State<'_, AppState>) -> Result<Vec<IdeConfig>, String> {
    let known_ides = get_known_ides();
    let mut detected = vec![];

    for ide_def in known_ides {
        // 检查是否已存在
        let store = state.store.lock().expect("store lock poisoned");
        let already_exists = store.ides.iter().any(|i| i.id == ide_def.id);
        drop(store);

        if already_exists {
            continue;
        }

        // 查找可执行文件：先固定路径，再从 PATH 命令发现
        let exe_path = resolve_ide_executable(&ide_def);

        if let Some(path) = exe_path {
            let icon_source = resolve_icon_source_path(&path, ide_def.executable_name);
            let icon = extract_icon_from_exe(&icon_source).or_else(|| {
                let placeholder = IdeConfig {
                    id: ide_def.id.to_string(),
                    name: ide_def.name.to_string(),
                    executable: path.to_string_lossy().to_string(),
                    args_template: ide_def.args_template.to_string(),
                    icon: None,
                    category: ide_def.category.clone(),
                    priority: ide_def.priority,
                    auto_detected: true,
                };
                load_cached_ide_icon(&state.file_path, ide_def.id)
                    .or_else(|| download_and_cache_ide_icon(&state.file_path, &placeholder))
            });

            detected.push(IdeConfig {
                id: ide_def.id.to_string(),
                name: ide_def.name.to_string(),
                executable: path.to_string_lossy().to_string(),
                args_template: ide_def.args_template.to_string(),
                icon,
                category: ide_def.category.clone(),
                priority: ide_def.priority,
                auto_detected: true,
            });
        }
    }

    Ok(detected)
}

#[tauri::command]
fn add_detected_ides(state: State<'_, AppState>) -> Result<Vec<IdeConfig>, String> {
    let detected_ides = scan_ides(state.clone())?;

    if detected_ides.is_empty() {
        return Ok(vec![]);
    }

    let mut store = state.store.lock().expect("store lock poisoned");
    let mut added = vec![];

    for ide in detected_ides {
        // 再次检查是否已存在（防止竞态条件）
        if !store.ides.iter().any(|i| i.id == ide.id) {
            store.ides.push(ide.clone());
            added.push(ide);
        }
    }

    if !added.is_empty() {
        save_store(&state.file_path, &store)?;
    }

    Ok(added)
}

#[tauri::command]
fn set_project_ide_preferences(
    project_id: String,
    ide_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let mut store = state.store.lock().expect("store lock poisoned");
    let valid_ide_ids: HashSet<&str> = store.ides.iter().map(|i| i.id.as_str()).collect();

    let mut seen: HashSet<String> = HashSet::new();
    let mut normalized: Vec<String> = ide_ids
        .into_iter()
        .filter(|id| valid_ide_ids.contains(id.as_str()))
        .filter(|id| seen.insert(id.clone()))
        .collect();
    normalized.truncate(3);

    let project = store
        .projects
        .iter_mut()
        .find(|p| p.id == project_id)
        .ok_or_else(|| "项目不存在".to_string())?;

    project.metadata.ide_preferences = normalized;
    let updated = project.clone();
    save_store(&state.file_path, &store)?;
    Ok(updated)
}

fn launch_with_ide(project: &Project, ide: &IdeConfig) -> Result<(), String> {
    let args = expand_args(&ide.args_template, project);
    let mut launched = false;

    if ide.category == IdeCategory::Cli || ide.category == IdeCategory::Terminal {
        #[cfg(target_os = "windows")]
        {
            let mut wt = Command::new("wt");
            wt.arg("-d").arg(&project.path).arg(&ide.executable).args(&args);
            if wt.spawn().is_ok() {
                launched = true;
            }
        }
    }

    if !launched {
        Command::new(&ide.executable)
            .current_dir(&project.path)
            .args(args)
            .spawn()
            .map_err(|e| format!("启动 {} 失败: {e}", ide.name))?;
    }

    Ok(())
}

#[tauri::command]
fn reorder_projects(project_ids: Vec<String>, state: State<'_, AppState>) -> Result<(), String> {
    let mut store = state.store.lock().expect("store lock poisoned");
    if project_ids.is_empty() {
        return Ok(());
    }

    let mut rank = std::collections::HashMap::new();
    for (idx, id) in project_ids.iter().enumerate() {
        rank.insert(id.clone(), idx as i64 + 1);
    }

    let mut max_rank = rank.len() as i64 + 1;
    for project in &mut store.projects {
        if let Some(order) = rank.get(&project.id) {
            project.display_order = *order;
        } else {
            project.display_order = max_rank;
            max_rank += 1;
        }
    }

    save_store(&state.file_path, &store)
}

#[tauri::command]
fn launch_project(
    project_id: String,
    ide_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut store = state.store.lock().expect("store lock poisoned");
    let project_idx = store
        .projects
        .iter()
        .position(|p| p.id == project_id)
        .ok_or_else(|| "项目不存在".to_string())?;
    let project = store.projects[project_idx].clone();

    let selected_ides: Vec<IdeConfig> = if let Some(requested) = ide_id {
        vec![store
            .ides
            .iter()
            .find(|i| i.id == requested)
            .cloned()
            .ok_or_else(|| "IDE 不存在".to_string())?]
    } else {
        let preferred: Vec<IdeConfig> = project
            .metadata
            .ide_preferences
            .iter()
            .take(3)
            .filter_map(|preferred_id| store.ides.iter().find(|i| i.id == *preferred_id).cloned())
            .collect();
        if !preferred.is_empty() {
            preferred
        } else {
            vec![store
                .ides
                .iter()
                .min_by_key(|i| i.priority)
                .cloned()
                .ok_or_else(|| "没有可用 IDE，请先添加 IDE 配置".to_string())?]
        }
    };

    let mut launched_count = 0usize;
    let mut errors: Vec<String> = Vec::new();
    for ide in &selected_ides {
        match launch_with_ide(&project, ide) {
            Ok(()) => launched_count += 1,
            Err(err) => errors.push(err),
        }
    }

    if launched_count == 0 {
        return Err(errors.join("；"));
    }

    store.projects[project_idx].last_opened = Some(now_iso());
    save_store(&state.file_path, &store)?;
    Ok(())
}

#[tauri::command]
fn open_in_file_manager(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {e}"))?;
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {e}"))?;
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {e}"))?;
        return Ok(());
    }
    #[allow(unreachable_code)]
    Err("当前系统不支持打开文件管理器".to_string())
}

#[tauri::command]
fn open_in_terminal(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        // 方案1: 直接启动 PowerShell，使用 CREATE_NEW_CONSOLE 标志创建新窗口
        let result = Command::new("powershell")
            .args([
                "-NoExit",
                "-NoLogo",
                "-Command",
                &format!("Set-Location '{}'", &path)
            ])
            .creation_flags(0x00000010) // CREATE_NEW_CONSOLE
            .spawn();

        if result.is_ok() {
            return Ok(());
        }

        // 方案2: Windows Terminal - 默认就是新窗口
        let result = Command::new("wt")
            .args([
                "powershell",
                "-NoExit",
                "-NoLogo",
                "-Command",
                &format!("Set-Location '{}'", &path)
            ])
            .spawn();

        if result.is_ok() {
            return Ok(());
        }

        // 方案3: CMD with CREATE_NEW_CONSOLE
        let result = Command::new("cmd")
            .args([
                "/k",
                &format!("cd /d \"{}\"", &path)
            ])
            .creation_flags(0x00000010) // CREATE_NEW_CONSOLE
            .spawn();

        if result.is_ok() {
            return Ok(());
        }

        return Err("无法打开终端".to_string());
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-a")
            .arg("Terminal")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开终端失败: {e}"))?;
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        // 尝试常见的 Linux 终端，使用 --new-window 或直接启动新实例
        let terminals = [
            ("gnome-terminal", vec!["--working-directory", &path]),
            ("konsole", vec!["--new-window", "--workdir", &path]),
            ("xfce4-terminal", vec!["--working-directory", &path]),
            ("xterm", vec!["-e", &format!("cd '{}'", &path)]),
        ];

        for (term, args) in terminals {
            let mut cmd = Command::new(term);
            for arg in args {
                cmd.arg(arg);
            }
            if cmd.spawn().is_ok() {
                return Ok(());
            }
        }
        return Err("未找到可用的终端".to_string());
    }
    #[allow(unreachable_code)]
    Err("当前系统不支持打开终端".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MiniWindowPosition {
    x: i32,
    y: i32,
}

#[tauri::command]
fn save_mini_window_position(x: i32, y: i32, state: State<'_, AppState>) -> Result<(), String> {
    let pos_path = state
        .file_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("mini-window-position.json");
    let pos = MiniWindowPosition { x, y };
    let content = serde_json::to_string(&pos).map_err(|e| e.to_string())?;
    fs::write(pos_path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_mini_window_position(state: State<'_, AppState>) -> Option<MiniWindowPosition> {
    let pos_path = state
        .file_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("mini-window-position.json");
    let content = fs::read_to_string(pos_path).ok()?;
    serde_json::from_str(&content).ok()
}

#[tauri::command]
fn switch_to_mini_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(main_win) = app.get_webview_window("main") {
        main_win.hide().map_err(|e| e.to_string())?;
    }
    if let Some(mini_win) = app.get_webview_window("mini") {
        mini_win.show().map_err(|e| e.to_string())?;
        mini_win.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn switch_to_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(mini_win) = app.get_webview_window("mini") {
        mini_win.hide().map_err(|e| e.to_string())?;
    }
    if let Some(main_win) = app.get_webview_window("main") {
        main_win.show().map_err(|e| e.to_string())?;
        main_win.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

// 文件扩展名到语言的映射
fn get_language_from_extension(ext: &str) -> Option<&'static str> {
    let ext_lower = ext.to_ascii_lowercase();
    match ext_lower.as_str() {
        // Rust
        "rs" => Some("Rust"),
        // JavaScript/TypeScript
        "js" | "jsx" | "mjs" => Some("JavaScript"),
        "ts" | "tsx" => Some("TypeScript"),
        // Python
        "py" | "pyi" | "pyw" => Some("Python"),
        // Java
        "java" | "kt" | "kts" => Some("Java"),
        // Go
        "go" => Some("Go"),
        // C/C++
        "c" | "h" => Some("C"),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some("C++"),
        // C#
        "cs" => Some("C#"),
        // Web
        "html" | "htm" | "xhtml" => Some("HTML"),
        "css" | "scss" | "sass" | "less" => Some("CSS"),
        "vue" => Some("Vue"),
        "svelte" => Some("Svelte"),
        // Shell
        "sh" | "bash" | "zsh" | "fish" => Some("Shell"),
        "ps1" | "psm1" | "psd1" => Some("PowerShell"),
        // Config/Data
        "json" => Some("JSON"),
        "yaml" | "yml" => Some("YAML"),
        "toml" => Some("TOML"),
        "xml" => Some("XML"),
        "md" | "markdown" => Some("Markdown"),
        // Other
        "sql" => Some("SQL"),
        "rb" => Some("Ruby"),
        "php" => Some("PHP"),
        "swift" => Some("Swift"),
        "dart" => Some("Dart"),
        "r" => Some("R"),
        "scala" => Some("Scala"),
        "lua" => Some("Lua"),
        "ex" | "exs" => Some("Elixir"),
        "erl" | "hrl" => Some("Erlang"),
        "fs" | "fsi" | "fsx" => Some("F#"),
        _ => None,
    }
}

// 检查目录是否应该被跳过
fn should_skip_dir_for_stats(path: &Path) -> bool {
    let skip = [
        ".git",
        "node_modules",
        "target",
        ".venv",
        "venv",
        ".idea",
        ".vscode",
        "dist",
        "build",
        "out",
        "bin",
        "obj",
        ".next",
        ".nuxt",
        "vendor",
        "coverage",
        ".cache",
        "temp",
        "tmp",
        "__pycache__",
        ".pytest_cache",
        "node_modules_cache",
        ".gradle",
        "gradle",
        "mvn",
        ".m2",
    ];
    match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => skip.contains(&name) || name.starts_with('.') && name != ".vscode" && name != ".idea",
        None => false,
    }
}

// 统计单个文件的语言信息
fn count_file_lines(path: &Path) -> Option<(&'static str, u64)> {
    let ext = path.extension()?.to_str()?;
    let language = get_language_from_extension(ext)?;

    // 读取文件内容并计算行数
    let content = fs::read_to_string(path).ok()?;
    let lines = content.lines().count() as u64;

    Some((language, lines))
}

// 递归扫描项目目录统计语言
fn scan_project_languages(path: &Path) -> Result<HashMap<String, (u64, u32)>, String> {
    let mut language_data: HashMap<String, (u64, u32)> = HashMap::new();

    fn scan_dir(
        dir: &Path,
        language_data: &mut HashMap<String, (u64, u32)>,
        depth: u32,
        max_depth: u32,
    ) -> Result<(), String> {
        if depth > max_depth || should_skip_dir_for_stats(dir) {
            return Ok(());
        }

        let entries = fs::read_dir(dir)
            .map_err(|e| format!("无法读取目录 {}: {}", dir.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("无法读取目录项: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                scan_dir(&path, language_data, depth + 1, max_depth)?;
            } else if path.is_file() {
                if let Some((language, lines)) = count_file_lines(&path) {
                    let entry = language_data.entry(language.to_string()).or_insert((0, 0));
                    entry.0 += lines;
                    entry.1 += 1;
                }
            }
        }

        Ok(())
    }

    scan_dir(path, &mut language_data, 0, 50)?;
    Ok(language_data)
}

#[tauri::command]
fn scan_project_language_stats(project_id: String, state: State<'_, AppState>) -> Result<LanguageStats, String> {
    let mut store = state.store.lock().expect("store lock poisoned");

    let project = store
        .projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| "项目不存在".to_string())?;

    let project_path = Path::new(&project.path);
    if !project_path.exists() || !project_path.is_dir() {
        return Err("项目路径不存在或不是目录".to_string());
    }

    // 扫描语言统计
    let language_data = scan_project_languages(project_path)
        .map_err(|e| format!("扫描语言统计失败: {}", e))?;

    let total_lines: u64 = language_data.values().map(|(lines, _)| *lines).sum();

    let mut languages: Vec<LanguageEntry> = language_data
        .into_iter()
        .map(|(language, (lines, files))| {
            let percentage = if total_lines > 0 {
                (lines as f64 / total_lines as f64) * 100.0
            } else {
                0.0
            };
            LanguageEntry {
                language,
                lines,
                files,
                percentage,
            }
        })
        .collect();

    // 按行数降序排序
    languages.sort_by(|a, b| b.lines.cmp(&a.lines));

    let stats = LanguageStats {
        total_lines,
        languages,
        scanned_at: now_iso(),
    };

    // 更新项目的语言统计信息
    let project_idx = store
        .projects
        .iter()
        .position(|p| p.id == project_id)
        .ok_or_else(|| "项目不存在".to_string())?;

    store.projects[project_idx].metadata.language_stats = Some(stats.clone());
    save_store(&state.file_path, &store)?;

    Ok(stats)
}

#[tauri::command]
fn get_project_language_stats(project_id: String, state: State<'_, AppState>) -> Result<Option<LanguageStats>, String> {
    let store = state.store.lock().expect("store lock poisoned");

    let project = store
        .projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| "项目不存在".to_string())?;

    Ok(project.metadata.language_stats.clone())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("无法获取应用数据目录: {e}"))?;
            fs::create_dir_all(&app_data_dir).map_err(|e| format!("无法创建应用数据目录: {e}"))?;
            let store_path = app_data_dir.join("store.json");
            let store = load_store(&store_path);
            app.manage(AppState {
                file_path: store_path,
                store: Mutex::new(store),
            });

            tray::create_tray(app).map_err(|e| format!("创建托盘失败: {e}"))?;

            if let Some(main_win) = app.get_webview_window("main") {
                let win = main_win.clone();
                main_win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win.hide();
                    }
                });
            }

            if let Some(mini_win) = app.get_webview_window("mini") {
                let win = mini_win.clone();
                mini_win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win.hide();
                    }
                });
            }

            #[cfg(desktop)]
            {
                // 全局快捷键功能已移除
                // 如需重新启用，请确保正确处理热键注册冲突
            }

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_projects,
            get_ides,
            add_project,
            remove_project,
            toggle_project_favorite,
            scan_projects,
            add_ide,
            remove_ide,
            set_ide_icon_from_file,
            reorder_projects,
            launch_project,
            open_in_file_manager,
            open_in_terminal,
            scan_ides,
            add_detected_ides,
            set_project_ide_preferences,
            save_mini_window_position,
            load_mini_window_position,
            switch_to_mini_window,
            switch_to_main_window,
            scan_project_language_stats,
            get_project_language_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
