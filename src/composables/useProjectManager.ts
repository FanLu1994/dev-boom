import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import {
  addIde,
  addDetectedIdes,
  getIdes,
  getProjects,
  launchProject,
  openInFileManager,
  removeProject,
  scanProjects as scanProjectsApi,
  setProjectIdePreferences,
  toggleProjectFavorite,
} from "../api/projectApi";
import type { IdeConfig, IdeForm, Project, ProjectForm } from "../types/project";

const EMPTY_PROJECT_FORM: ProjectForm = {
  path: "",
  maxDepth: 3,
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

  async function createProject() {
    if (!projectForm.value.path) {
      errorMessage.value = "请先选择扫描根目录";
      return;
    }

    try {
      const added = await scanProjectsApi(projectForm.value.path, projectForm.value.maxDepth);
      const total = Array.isArray(added) ? added.length : 0;
      projectForm.value = { ...EMPTY_PROJECT_FORM };
      showProjectDialog.value = false;
      errorMessage.value = total ? `扫描完成，新增 ${total} 个项目` : "扫描完成，未发现新项目";
      await loadData();
    } catch (error) {
      setError("扫描导入失败", error);
    }
  }

  async function createIde() {
    try {
      await addIde(ideForm.value);
      ideForm.value = { ...EMPTY_IDE_FORM };
      showIdeDialog.value = false;
      await loadData();
    } catch (error) {
      setError("添加 IDE 失败", error);
    }
  }

  async function autoScanIdes() {
    try {
      const added = await addDetectedIdes();
      const count = Array.isArray(added) ? added.length : 0;
      if (count > 0) {
        await loadData();
        errorMessage.value = `成功添加 ${count} 个 IDE`;
      } else {
        errorMessage.value = "未发现新的 IDE";
      }
    } catch (error) {
      setError("扫描 IDE 失败", error);
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

  return {
    ides,
    loading,
    errorMessage,
    showProjectDialog,
    showIdeDialog,
    showLaunchDialog,
    launchProjectTarget,
    launchSelectedIdeIds,
    searchText,
    favoritesOnly,
    projectForm,
    ideForm,
    filteredProjects,
    loadData,
    chooseProjectFolders,
    createProject,
    createIde,
    autoScanIdes,
    onRemoveProject,
    onToggleFavorite,
    openLaunchDialog,
    closeLaunchDialog,
    confirmLaunchProject,
    onOpenFolder,
  };
}
