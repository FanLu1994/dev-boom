import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import {
  addIde,
  addDetectedIdes,
  getIdes,
  getProjects,
  launchProject,
  openInFileManager,
  openInTerminal,
  removeIde,
  removeProject,
  scanProjects as scanProjectsApi,
  scanProjectLanguageStats,
  setIdeIconFromFile,
  setProjectIdePreferences,
  toggleProjectFavorite,
} from "../api/projectApi";
import type { IdeConfig, IdeForm, Project, ProjectForm } from "../types/project";

const EMPTY_PROJECT_FORM: ProjectForm = {
  path: "",
  maxDepth: 1,
};

const EMPTY_IDE_FORM: IdeForm = {
  name: "",
  executable: "",
  argsTemplate: "{projectPath}",
  category: "Gui",
  priority: 200,
};

function projectModifiedTs(project: Project) {
  const value = project.lastModified ?? project.lastOpened ?? project.createdAt;
  const ts = value ? new Date(value).getTime() : 0;
  return Number.isFinite(ts) ? ts : 0;
}

export function useProjectManager() {
  const projects = ref<Project[]>([]);
  const ides = ref<IdeConfig[]>([]);
  const loading = ref(true);
  const errorMessage = ref("");

  const showProjectDialog = ref(false);
  const showIdeDialog = ref(false);
  const showLaunchDialog = ref(false);
  const launchProjectTarget = ref<Project | null>(null);
  const launchSelectedIdeIds = ref<string[]>([]);

  // 语言统计相关状态
  const showLanguageStatsDialog = ref(false);
  const languageStatsProjectId = ref<string | null>(null);
  const scanningLanguageStats = ref(false);

  // IDE 扫描相关状态
  const scanningIdes = ref(false);
  const scanResults = ref<IdeConfig[]>([]);
  const scanMessage = ref("");

  const searchText = ref("");
  const favoritesOnly = ref(false);

  const projectForm = ref<ProjectForm>({ ...EMPTY_PROJECT_FORM });
  const ideForm = ref<IdeForm>({ ...EMPTY_IDE_FORM });

  const filteredProjects = computed(() => {
    const q = searchText.value.trim().toLowerCase();
    return [...projects.value]
      .filter((project) => {
        if (favoritesOnly.value && !project.favorite) return false;
        if (!q) return true;
        return (
          project.name.toLowerCase().includes(q) ||
          project.path.toLowerCase().includes(q) ||
          project.tags.some((tag) => tag.toLowerCase().includes(q))
        );
      })
      .sort((a, b) => projectModifiedTs(b) - projectModifiedTs(a));
  });

  function setError(prefix: string, error: unknown) {
    errorMessage.value = `${prefix}: ${String(error)}`;
  }

  async function loadData() {
    loading.value = true;
    errorMessage.value = "";
    try {
      const [projectData, ideData] = await Promise.all([getProjects(), getIdes()]);
      projects.value = projectData;
      ides.value = ideData;
    } catch (error) {
      setError("加载失败", error);
    } finally {
      loading.value = false;
    }
  }

  async function chooseProjectFolders() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "选择扫描根目录",
      });
      if (!selected) return;
      projectForm.value.path = Array.isArray(selected) ? selected[0] ?? "" : selected;
    } catch (error) {
      setError("选择文件夹失败", error);
    }
  }

  async function chooseIdeExecutable() {
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        title: "选择 IDE 可执行文件",
        filters: [
          { name: "Executable", extensions: ["exe", "cmd", "bat"] },
          { name: "All files", extensions: ["*"] },
        ],
      });
      if (!selected) return;
      const filePath = Array.isArray(selected) ? selected[0] ?? "" : selected;
      ideForm.value.executable = filePath;

      // 自动填充 IDE 名称（如果当前为空）
      if (!ideForm.value.name.trim() && filePath) {
        // 从路径中提取文件名（不含扩展名）
        const fileName = filePath.split(/[/\\]/).pop() ?? "";
        const nameWithoutExt = fileName.replace(/\.(exe|cmd|bat|ps1)$/i, "");
        // 美化名称：将常见的 IDE 名称格式化
        const prettyName = prettifyIdeName(nameWithoutExt);
        ideForm.value.name = prettyName;
      }
    } catch (error) {
      setError("选择可执行文件失败", error);
    }
  }

  function prettifyIdeName(name: string): string {
    // 常见 IDE 名称映射
    const knownNames: Record<string, string> = {
      code: "Visual Studio Code",
      "code-insiders": "VS Code Insiders",
      cursor: "Cursor",
      windsurf: "Windsurf",
      idea: "IntelliJ IDEA",
      idea64: "IntelliJ IDEA",
      webstorm: "WebStorm",
      webstorm64: "WebStorm",
      pycharm: "PyCharm",
      pycharm64: "PyCharm",
      goland: "GoLand",
      goland64: "GoLand",
      clion: "CLion",
      clion64: "CLion",
      rider: "Rider",
      rider64: "Rider",
      rustrover: "RustRover",
      rustrover64: "RustRover",
      datagrip: "DataGrip",
      datagrip64: "DataGrip",
      phpstorm: "PhpStorm",
      phpstorm64: "PhpStorm",
      sublime_text: "Sublime Text",
      notepad: "Notepad++",
      "notepad++": "Notepad++",
      atom: "Atom",
      vim: "Vim",
      nvim: "Neovim",
      neovim: "Neovim",
      emacs: "Emacs",
      fleet: "Fleet",
      zed: "Zed",
      lapce: "Lapce",
      helix: "Helix",
    };

    const lowerName = name.toLowerCase();
    if (knownNames[lowerName]) {
      return knownNames[lowerName];
    }

    // 默认：首字母大写，将下划线和连字符转为空格
    return name
      .replace(/[-_]/g, " ")
      .replace(/\b\w/g, (c) => c.toUpperCase());
  }

  async function chooseAndSetIdeIcon(ideId: string) {
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        title: "选择 IDE 图标或可执行文件",
        filters: [
          { name: "Icon & Executable", extensions: ["png", "svg", "ico", "jpg", "jpeg", "webp", "exe", "cmd", "bat", "ps1"] },
          { name: "All files", extensions: ["*"] },
        ],
      });
      if (!selected) return;
      const filePath = Array.isArray(selected) ? selected[0] ?? "" : selected;
      if (!filePath) return;
      await setIdeIconFromFile(ideId, filePath);
      errorMessage.value = "图标已更新";
      await loadData();
    } catch (error) {
      setError("设置图标失败", error);
    }
  }

  async function createProject() {
    if (!projectForm.value.path) {
      errorMessage.value = "请先选择扫描根目录";
      return;
    }

    try {
      const added = await scanProjectsApi(projectForm.value.path, projectForm.value.maxDepth);
      const projects = Array.isArray(added) ? added : [];
      projectForm.value = { ...EMPTY_PROJECT_FORM };
      showProjectDialog.value = false;

      // 统计新项目和更新的项目
      const now = Date.now();
      const newCount = projects.filter((p: Project) => {
        const createdAt = new Date(p.createdAt).getTime();
        return Math.abs(createdAt - now) < 5000; // 5秒内创建的算是新项目
      }).length;
      const updatedCount = projects.length - newCount;

      let message = "";
      if (newCount > 0 && updatedCount > 0) {
        message = `扫描完成，新增 ${newCount} 个项目，更新 ${updatedCount} 个项目的语言统计`;
      } else if (newCount > 0) {
        message = `扫描完成，新增 ${newCount} 个项目`;
      } else if (updatedCount > 0) {
        message = `扫描完成，更新了 ${updatedCount} 个项目的语言统计`;
      } else {
        message = "扫描完成，未发现新项目";
      }
      errorMessage.value = message;
      await loadData();
    } catch (error) {
      setError("扫描导入失败", error);
    }
  }

  async function createIde() {
    try {
      await addIde(ideForm.value);
      ideForm.value = { ...EMPTY_IDE_FORM };
      errorMessage.value = "IDE 添加成功";
      await loadData();
    } catch (error) {
      setError("添加 IDE 失败", error);
    }
  }

  async function autoScanIdes() {
    try {
      scanningIdes.value = true;
      scanResults.value = [];
      scanMessage.value = "";

      const added = await addDetectedIdes();
      const count = Array.isArray(added) ? added.length : 0;

      if (count > 0) {
        scanResults.value = added;
        scanMessage.value = `成功发现 ${count} 个新的 IDE`;
        await loadData();
      } else {
        scanMessage.value = "未发现新的 IDE，请尝试手动添加";
      }
    } catch (error) {
      scanMessage.value = "扫描 IDE 失败";
      setError("扫描 IDE 失败", error);
    } finally {
      scanningIdes.value = false;
    }
  }

  async function onRemoveProject(projectId: string) {
    try {
      await removeProject(projectId);
      await loadData();
    } catch (error) {
      setError("删除项目失败", error);
    }
  }

  async function onToggleFavorite(projectId: string) {
    try {
      await toggleProjectFavorite(projectId);
      await loadData();
    } catch (error) {
      setError("收藏状态更新失败", error);
    }
  }

  async function onRemoveIde(ideId: string) {
    try {
      await removeIde(ideId);
      await loadData();
    } catch (error) {
      setError("删除 IDE 失败", error);
    }
  }

  function openLaunchDialog(project: Project) {
    launchProjectTarget.value = project;
    const preferred = project.metadata.idePreferences.filter((id) => ides.value.some((ide) => ide.id === id));
    launchSelectedIdeIds.value = preferred.length > 0 ? preferred : ides.value.slice(0, 1).map((ide) => ide.id);
    showLaunchDialog.value = true;
  }

  function closeLaunchDialog() {
    showLaunchDialog.value = false;
    launchProjectTarget.value = null;
    launchSelectedIdeIds.value = [];
  }

  async function confirmLaunchProject() {
    const project = launchProjectTarget.value;
    if (!project) return;
    if (launchSelectedIdeIds.value.length === 0) {
      errorMessage.value = "请至少选择一个 IDE";
      return;
    }

    try {
      await setProjectIdePreferences(project.id, launchSelectedIdeIds.value);
      await launchProject(project.id);
      closeLaunchDialog();
      await loadData();
    } catch (error) {
      setError("启动失败", error);
    }
  }

  async function onOpenFolder(path: string) {
    try {
      await openInFileManager(path);
    } catch (error) {
      setError("打开文件夹失败", error);
    }
  }

  async function onOpenTerminal(path: string) {
    try {
      await openInTerminal(path);
    } catch (error) {
      setError("打开终端失败", error);
    }
  }

  function openLanguageStatsDialog(projectId: string) {
    languageStatsProjectId.value = projectId;
    showLanguageStatsDialog.value = true;
  }

  function closeLanguageStatsDialog() {
    showLanguageStatsDialog.value = false;
    languageStatsProjectId.value = null;
  }

  async function refreshLanguageStats(projectId: string) {
    if (!projectId) return;

    scanningLanguageStats.value = true;
    errorMessage.value = "";
    try {
      await scanProjectLanguageStats(projectId);
      await loadData();
      errorMessage.value = "语言统计已更新";
    } catch (error) {
      setError("统计语言分布失败", error);
    } finally {
      scanningLanguageStats.value = false;
    }
  }

  const languageStatsProject = computed(() => {
    if (!languageStatsProjectId.value) return null;
    return projects.value.find(p => p.id === languageStatsProjectId.value) || null;
  });

  return {
    ides,
    loading,
    errorMessage,
    showProjectDialog,
    showIdeDialog,
    showLaunchDialog,
    launchProjectTarget,
    launchSelectedIdeIds,
    showLanguageStatsDialog,
    languageStatsProject,
    scanningLanguageStats,
    scanningIdes,
    scanResults,
    scanMessage,
    searchText,
    favoritesOnly,
    projectForm,
    ideForm,
    filteredProjects,
    loadData,
    chooseProjectFolders,
    chooseIdeExecutable,
    chooseAndSetIdeIcon,
    createProject,
    createIde,
    autoScanIdes,
    onRemoveProject,
    onRemoveIde,
    onToggleFavorite,
    openLaunchDialog,
    closeLaunchDialog,
    confirmLaunchProject,
    onOpenFolder,
    onOpenTerminal,
    openLanguageStatsDialog,
    closeLanguageStatsDialog,
    refreshLanguageStats,
  };
}
