<script setup lang="ts">
import { onMounted } from "vue";
import AppTitleBar from "./components/AppTitleBar.vue";
import IdeDialog from "./components/IdeDialog.vue";
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
  searchText,
  favoritesOnly,
  projectForm,
  ideForm,
  filteredProjects,
  loadData,
  chooseProjectFolders,
  createProject,
  createIde,
  onRemoveProject,
  onToggleFavorite,
  onLaunchProject,
  onOpenFolder,
  ideNameById,
  selectedIdeId,
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
        :selected-ide-label="(project) => ideNameById(selectedIdeId(project))"
        :format-last-modified="formatLastModified"
        @toggle-favorite="onToggleFavorite"
        @remove="onRemoveProject"
        @launch="onLaunchProject"
        @open-folder="onOpenFolder"
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
      @close="showIdeDialog = false"
      @submit="createIde"
      @update:name="ideForm.name = $event"
      @update:executable="ideForm.executable = $event"
      @update:args-template="ideForm.argsTemplate = $event"
      @update:category="ideForm.category = $event"
      @update:priority="ideForm.priority = $event"
    />
  </main>
</template>

