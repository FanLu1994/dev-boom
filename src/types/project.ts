export type ProjectType = "Rust" | "Nodejs" | "Python" | "Java" | "Go" | "Dotnet" | "Generic";
export type IdeCategory = "Gui" | "Cli" | "Terminal" | "Browser";
export type ThemeMode = "light" | "dark";

export interface ProjectMetadata {
  idePreferences: string[];
  gitUrl: string | null;
  description: string | null;
}

export interface Project {
  id: string;
  name: string;
  path: string;
  projectType: ProjectType;
  favorite: boolean;
  tags: string[];
  lastOpened: string | null;
  lastModified: string | null;
  createdAt: string;
  metadata: ProjectMetadata;
}

export interface IdeConfig {
  id: string;
  name: string;
  executable: string;
  argsTemplate: string;
  icon: string | null;
  category: IdeCategory;
  priority: number;
  autoDetected: boolean;
}

export interface ProjectForm {
  path: string;
  maxDepth: number;
}

export interface IdeForm {
  name: string;
  executable: string;
  argsTemplate: string;
  category: IdeCategory;
  priority: number;
}

export const TYPE_COLORS: Record<ProjectType, string> = {
  Rust: "#f97316",
  Nodejs: "#16a34a",
  Python: "#2563eb",
  Java: "#dc2626",
  Go: "#0891b2",
  Dotnet: "#0f766e",
  Generic: "#64748b",
};
