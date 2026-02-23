import { invoke } from "@tauri-apps/api/core";
import type { IdeConfig, IdeForm, LanguageStats, Project } from "../types/project";

export async function getProjects() {
  return invoke<Project[]>("get_projects");
}

export async function getIdes() {
  return invoke<IdeConfig[]>("get_ides");
}

export async function addIde(input: IdeForm) {
  return invoke("add_ide", { input });
}

export async function scanProjects(rootPath: string, maxDepth = 3) {
  return invoke("scan_projects", { rootPath, maxDepth });
}

export async function removeProject(projectId: string) {
  return invoke("remove_project", { projectId });
}

export async function toggleProjectFavorite(projectId: string) {
  return invoke("toggle_project_favorite", { projectId });
}

export async function launchProject(projectId: string, ideId?: string) {
  return invoke("launch_project", { projectId, ideId: ideId ?? null });
}

export async function setProjectIdePreferences(projectId: string, ideIds: string[]) {
  return invoke("set_project_ide_preferences", { projectId, ideIds });
}

export async function openInFileManager(path: string) {
  return invoke("open_in_file_manager", { path });
}

export async function openInTerminal(path: string) {
  return invoke("open_in_terminal", { path });
}

export async function scanIdes() {
  return invoke<IdeConfig[]>("scan_ides");
}

export async function addDetectedIdes() {
  return invoke<IdeConfig[]>("add_detected_ides");
}

export async function setIdeIconFromFile(ideId: string, filePath: string) {
  return invoke<IdeConfig>("set_ide_icon_from_file", { ideId, filePath });
}

export async function removeIde(ideId: string) {
  return invoke("remove_ide", { ideId });
}

export async function scanProjectLanguageStats(projectId: string) {
  return invoke<LanguageStats>("scan_project_language_stats", { projectId });
}

export async function getProjectLanguageStats(projectId: string) {
  return invoke<LanguageStats | null>("get_project_language_stats", { projectId });
}
