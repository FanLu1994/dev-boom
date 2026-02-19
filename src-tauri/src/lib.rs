use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use uuid::Uuid;

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
                if project.display_order == 0 {
                    project.display_order = idx as i64 + 1;
                }
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
    let store = state.store.lock().expect("store lock poisoned");
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
            Ok(v) => v.to_string_lossy().to_string(),
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

    let selected_ide = if let Some(requested) = ide_id {
        store
            .ides
            .iter()
            .find(|i| i.id == requested)
            .cloned()
            .ok_or_else(|| "IDE 不存在".to_string())?
    } else if let Some(preferred_id) = project.metadata.ide_preferences.first() {
        store
            .ides
            .iter()
            .find(|i| i.id == *preferred_id)
            .cloned()
            .ok_or_else(|| "项目首选 IDE 不存在".to_string())?
    } else {
        store
            .ides
            .iter()
            .min_by_key(|i| i.priority)
            .cloned()
            .ok_or_else(|| "没有可用 IDE，请先添加 IDE 配置".to_string())?
    };

    let args = expand_args(&selected_ide.args_template, &project);
    let mut launched = false;

    if selected_ide.category == IdeCategory::Cli || selected_ide.category == IdeCategory::Terminal {
        #[cfg(target_os = "windows")]
        {
            let mut wt = Command::new("wt");
            wt.arg("-d")
                .arg(&project.path)
                .arg(&selected_ide.executable)
                .args(&args);
            if wt.spawn().is_ok() {
                launched = true;
            }
        }
    }

    if !launched {
        Command::new(&selected_ide.executable)
            .current_dir(&project.path)
            .args(args)
            .spawn()
            .map_err(|e| format!("启动失败: {e}"))?;
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
            open_in_file_manager
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
