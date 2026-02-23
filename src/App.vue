<script setup lang="ts">
import { onMounted } from "vue";
import AppTitleBar from "./components/AppTitleBar.vue";
import IdeDialog from "./components/IdeDialog.vue";
import LaunchProjectDialog from "./components/LaunchProjectDialog.vue";
import ProjectDialog from "./components/ProjectDialog.vue";
import ProjectGroupList from "./components/ProjectGroupList.vue";
import ProjectToolbar from "./components/ProjectToolbar.vue";
import { useProjectManager } from "./composables/useProjectManager";
import { useTheme } from "./composables/useTheme";
import { useWindowControls } from "./composables/useWindowControls";

const manager = useProjectManager();
const { theme, applyTheme, toggleTheme } = useTheme();
const { minimizeWindow, toggleMaximizeWindow, closeWindow, startWindowDrag } = useWindowControls();
const {
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
} = manager;

function formatLastModified(value: string | null) {
  if (!value) return "未知";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "未知";
  return date.toLocaleString("zh-CN", { hour12: false });
}

onMounted(async () => {
  applyTheme();
  await loadData();
});
</script>

<template>
  <main class="window-shell">
    <AppTitleBar
      :theme="theme"
      @toggle-theme="toggleTheme"
      @minimize="minimizeWindow"
      @maximize="toggleMaximizeWindow"
      @close="closeWindow"
      @drag-start="startWindowDrag"
      @open-project-dialog="showProjectDialog = true"
      @open-ide-dialog="showIdeDialog = true"
    />

    <section class="content-pane">
      <ProjectToolbar
        :search-text="searchText"
        :favorites-only="favoritesOnly"
        @update:search-text="searchText = $event"
        @update:favorites-only="favoritesOnly = $event"
      />

      <p v-if="errorMessage" class="error">{{ errorMessage }}</p>
      <p v-if="loading" class="hint">正在加载项目数据...</p>
      <p v-else-if="!filteredProjects.length" class="hint">暂无匹配项目，可以先“添加项目”。</p>
      <ProjectGroupList
        v-else
        :projects="filteredProjects"
        :ides="ides"
        :format-last-modified="formatLastModified"
        @toggle-favorite="onToggleFavorite"
        @remove="onRemoveProject"
        @launch="openLaunchDialog"
        @open-folder="onOpenFolder"
        @open-terminal="onOpenTerminal"
      />
    </section>

    <ProjectDialog
      :visible="showProjectDialog"
      :form="projectForm"
      @close="showProjectDialog = false"
      @choose-folders="chooseProjectFolders"
      @submit="createProject"
      @update:max-depth="projectForm.maxDepth = $event"
    />

    <IdeDialog
      :visible="showIdeDialog"
      :form="ideForm"
      :ides="ides"
      @close="showIdeDialog = false"
      @submit="createIde"
      @remove="onRemoveIde"
      @scan="autoScanIdes"
      @set-icon="chooseAndSetIdeIcon"
      @choose-executable="chooseIdeExecutable"
      @update:name="ideForm.name = $event"
      @update:args-template="ideForm.argsTemplate = $event"
      @update:category="ideForm.category = $event"
      @update:priority="ideForm.priority = $event"
    />

    <LaunchProjectDialog
      :visible="showLaunchDialog"
      :project="launchProjectTarget"
      :ides="ides"
      :selected-ide-ids="launchSelectedIdeIds"
      @close="closeLaunchDialog"
      @confirm="confirmLaunchProject"
      @update:selected-ide-ids="launchSelectedIdeIds = $event"
    />
  </main>
</template>
