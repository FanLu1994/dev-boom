use std::{
    collections::HashSet,
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
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
        CreateDIBSection, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
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
        Some(format!("data:image/png;base64,{}", icon))
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
        let width = (if icon_info.xHotspot > 0 { icon_info.xHotspot as i32 } else { 32 }).clamp(1, 512);
        let height = (if icon_info.yHotspot > 0 { icon_info.yHotspot as i32 } else { 32 }).clamp(1, 512);

        let mut ppv_bits: *mut std::ffi::c_void = null_mut();

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: height * 2, // DIB section height is doubled for icons
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
                DeleteDC(mem_dc);
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
            DeleteDC(mem_dc);
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
        for &pixel in pixels_slice {
            let b = (pixel & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let r = ((pixel >> 16) & 0xFF) as u8;
            let a = ((pixel >> 24) & 0xFF) as u8;
            rgba_pixels.push(r);
            rgba_pixels.push(g);
            rgba_pixels.push(b);
            rgba_pixels.push(a);
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
            DeleteDC(mem_dc);
            ReleaseDC(None, hdc);
            use base64::Engine;
            return Some(base64::engine::general_purpose::STANDARD.encode(&png_bytes));
        }
    }

    DeleteDC(mem_dc);
    ReleaseDC(None, hdc);
    None
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

fn split_args(args: &str) -> Vec<String> {
    shlex::split(args).unwrap_or_default()
}

fn expand_args(args_template: &str, project: &Project) -> Vec<String> {
    let replaced = args_template
        .replace("{projectPath}", &project.path)
        .replace("{projectName}", &project.name);
    split_args(&replaced)
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
        if ide.icon.is_some() {
            continue;
        }
        let resolved = PathBuf::from(&ide.executable);
        let icon = if resolved.exists() {
            let source = resolve_icon_source_path(&resolved, &ide.executable);
            extract_icon_from_exe(&source)
        } else if let Some(path) = find_executable_in_path(&ide.executable) {
            let source = resolve_icon_source_path(&path, &ide.executable);
            extract_icon_from_exe(&source)
        } else {
            None
        };
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
        if existing_paths.contains(&canonical) {
            continue;
        }
        existing_paths.insert(canonical.clone());

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
            },
        };
        next_order += 1;
        store.projects.push(project.clone());
        added.push(project);
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
            let icon = extract_icon_from_exe(&icon_source);

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
            reorder_projects,
            launch_project,
            open_in_file_manager,
            scan_ides,
            add_detected_ides,
            set_project_ide_preferences,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
