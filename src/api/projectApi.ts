import { invoke } from "@tauri-apps/api/core";
import type { IdeConfig, IdeForm, Project } from "../types/project";

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

export async function openInFileManager(path: string) {
  return invoke("open_in_file_manager", { path });
}
