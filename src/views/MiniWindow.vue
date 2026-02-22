<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Project, IdeConfig } from "../types/project";
import { TYPE_COLORS } from "../types/project";
import { useTheme } from "../composables/useTheme";

const { theme, applyTheme, toggleTheme } = useTheme();

const projects = ref<Project[]>([]);
const ides = ref<IdeConfig[]>([]);
const searchText = ref("");
const favoritesOnly = ref(false);
const loading = ref(true);

const appWindow = getCurrentWindow();

const brokenIconIds = ref<Record<string, boolean>>({});

function markIconBroken(ideId: string) {
  brokenIconIds.value[ideId] = true;
}

const totalCount = computed(() => projects.value.length);
const favoriteCount = computed(() => projects.value.filter((p) => p.favorite).length);

const filteredProjects = computed(() => {
  const q = searchText.value.trim().toLowerCase();
  let list = [...projects.value];
  if (favoritesOnly.value) {
    list = list.filter((p) => p.favorite);
  }
  list.sort((a, b) => {
    if (a.favorite !== b.favorite) return a.favorite ? -1 : 1;
    return (b.lastOpened ?? "").localeCompare(a.lastOpened ?? "");
  });
  if (q) {
    list = list.filter(
      (p) =>
        p.name.toLowerCase().includes(q) ||
        p.path.toLowerCase().includes(q) ||
        p.tags.some((t) => t.toLowerCase().includes(q))
    );
  }
  return list.slice(0, 15);
});

async function loadData() {
  try {
    const [p, i] = await Promise.all([
      invoke<Project[]>("get_projects"),
      invoke<IdeConfig[]>("get_ides"),
    ]);
    projects.value = p;
    ides.value = i;
  } catch {
    // silent
  } finally {
    loading.value = false;
  }
}

function getPreferredIde(project: Project): IdeConfig | undefined {
  const prefId = project.metadata.idePreferences?.[0];
  if (prefId) return ides.value.find((i) => i.id === prefId);
  return ides.value.length ? ides.value[0] : undefined;
}

function ideShortName(name: string) {
  return name.trim().slice(0, 1).toUpperCase();
}

async function quickLaunch(project: Project) {
  const ide = getPreferredIde(project);
  try {
    await invoke("launch_project", {
      projectId: project.id,
      ideId: ide?.id ?? null,
    });
  } catch {
    // silent
  }
}

async function toggleFavorite(projectId: string) {
  try {
    await invoke("toggle_project_favorite", { projectId });
    await loadData();
  } catch {
    // silent
  }
}

async function openFolder(path: string) {
  try {
    await invoke("open_in_file_manager", { path });
  } catch {
    // silent
  }
}

async function startDrag(event: MouseEvent) {
  const target = event.target as HTMLElement | null;
  if (target?.closest("input") || target?.closest("button")) return;
  try {
    await appWindow.startDragging();
  } catch {
    // noop
  }
}

async function hideMini() {
  try {
    await appWindow.hide();
  } catch {
    // noop
  }
}

async function switchToMain() {
  try {
    await invoke("switch_to_main_window");
  } catch {
    // noop
  }
}

function formatTime(value: string | null): string {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);
  if (minutes < 1) return "刚刚";
  if (minutes < 60) return `${minutes}分钟前`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}小时前`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}天前`;
  return date.toLocaleDateString("zh-CN");
}

onMounted(async () => {
  applyTheme();
  await loadData();

  try {
    const pos = await invoke<{ x: number; y: number } | null>(
      "load_mini_window_position"
    );
    if (pos) {
      await appWindow.setPosition(
        new (await import("@tauri-apps/api/dpi")).PhysicalPosition(pos.x, pos.y)
      );
    }
  } catch {
    // noop
  }
});
</script>

<template>
  <div class="mini-shell" @mousedown="startDrag">
    <header class="mini-header">
      <div class="mini-brand">
        <span class="mini-dot"></span>
        <span class="mini-title">Dev Boom</span>
        <span class="mini-stats">{{ totalCount }}项 · {{ favoriteCount }}★</span>
      </div>
      <div class="mini-header-actions">
        <button
          class="mini-btn"
          @click="toggleTheme"
          :title="theme === 'light' ? '深色模式' : '浅色模式'"
        >{{ theme === "light" ? "☾" : "☀" }}</button>
        <button class="mini-btn" @click="switchToMain" title="切换到主窗口">⇄</button>
        <button class="mini-close" @click="hideMini" title="隐藏">✕</button>
      </div>
    </header>

    <div class="mini-toolbar">
      <input
        v-model="searchText"
        type="text"
        placeholder="搜索项目…"
        class="mini-search-input"
      />
      <button
        class="mini-filter-btn"
        :class="{ active: favoritesOnly }"
        @click="favoritesOnly = !favoritesOnly"
        title="只看收藏"
      >★</button>
    </div>

    <div class="mini-list">
      <p v-if="loading" class="mini-hint">加载中...</p>
      <p v-else-if="!filteredProjects.length" class="mini-hint">无匹配项目</p>
      <div
        v-for="project in filteredProjects"
        :key="project.id"
        class="mini-item"
      >
        <div class="mini-item-main">
          <div class="mini-item-top">
            <span
              class="mini-type-badge"
              :style="{ background: TYPE_COLORS[project.projectType], color: '#fff' }"
            >{{ project.projectType }}</span>
            <span v-if="project.lastOpened" class="mini-time">{{ formatTime(project.lastOpened) }}</span>
          </div>
          <div class="mini-item-name-row">
            <span class="mini-item-name">{{ project.name }}</span>
            <span v-if="project.favorite" class="mini-star">★</span>
          </div>
          <span class="mini-item-path">{{ project.path }}</span>
          <div v-if="project.tags.length" class="mini-tags">
            <span v-for="tag in project.tags.slice(0, 3)" :key="tag" class="mini-tag">{{ tag }}</span>
            <span v-if="project.tags.length > 3" class="mini-tag more">+{{ project.tags.length - 3 }}</span>
          </div>
        </div>
        <div class="mini-item-actions">
          <div v-if="getPreferredIde(project)" class="mini-ide-icon" :title="getPreferredIde(project)!.name">
            <img
              v-if="getPreferredIde(project)!.icon && !brokenIconIds[getPreferredIde(project)!.id]"
              :src="getPreferredIde(project)!.icon!"
              :alt="getPreferredIde(project)!.name"
              class="ide-img"
              @error="markIconBroken(getPreferredIde(project)!.id)"
            />
            <span v-else class="ide-fallback">{{ ideShortName(getPreferredIde(project)!.name) }}</span>
          </div>
          <button
            class="mini-act-btn"
            @click.stop="toggleFavorite(project.id)"
            :title="project.favorite ? '取消收藏' : '收藏'"
          >{{ project.favorite ? "★" : "☆" }}</button>
          <button
            class="mini-act-btn"
            @click.stop="openFolder(project.path)"
            title="打开文件夹"
          >📂</button>
          <button
            class="mini-launch-btn"
            @click.stop="quickLaunch(project)"
            :title="getPreferredIde(project)?.name ?? '启动'"
          >▶</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mini-shell {
  width: 100%;
  height: 100%;
  background: var(--panel-strong);
  backdrop-filter: blur(20px) saturate(120%);
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--text-soft) 16%, transparent);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  user-select: none;
}

.mini-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 7px 10px 5px;
  flex-shrink: 0;
}

.mini-brand {
  display: flex;
  align-items: center;
  gap: 6px;
}

.mini-header-actions {
  display: flex;
  align-items: center;
  gap: 2px;
}

.mini-btn {
  border: none;
  background: transparent;
  color: var(--text-soft);
  cursor: pointer;
  font-size: 13px;
  padding: 2px 5px;
  border-radius: 5px;
  line-height: 1;
}
.mini-btn:hover {
  background: color-mix(in srgb, var(--text-soft) 14%, transparent);
  color: var(--text);
}

.mini-dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  background: #2563eb;
}

.mini-title {
  font-family: "Manrope", sans-serif;
  font-size: 13px;
  font-weight: 700;
  color: var(--text);
}

.mini-stats {
  font-size: 10px;
  color: var(--text-soft);
  opacity: 0.7;
}

.mini-close {
  border: none;
  background: transparent;
  color: var(--text-soft);
  cursor: pointer;
  font-size: 13px;
  padding: 2px 5px;
  border-radius: 5px;
  line-height: 1;
}
.mini-close:hover {
  background: color-mix(in srgb, var(--text-soft) 14%, transparent);
  color: var(--text);
}

.mini-toolbar {
  display: flex;
  gap: 5px;
  padding: 0 10px 6px;
  flex-shrink: 0;
}

.mini-search-input {
  flex: 1;
  padding: 5px 8px;
  border-radius: 6px;
  border: 1px solid color-mix(in srgb, var(--text-soft) 20%, transparent);
  background: color-mix(in srgb, var(--panel) 60%, transparent);
  color: var(--text);
  font-size: 12px;
  outline: none;
  font-family: inherit;
}
.mini-search-input:focus {
  border-color: #2563eb;
}

.mini-filter-btn {
  border: 1px solid color-mix(in srgb, var(--text-soft) 20%, transparent);
  background: color-mix(in srgb, var(--panel) 60%, transparent);
  color: var(--text-soft);
  cursor: pointer;
  font-size: 12px;
  padding: 3px 7px;
  border-radius: 6px;
  line-height: 1;
  transition: all 0.15s;
}
.mini-filter-btn.active {
  background: #f59e0b;
  color: #fff;
  border-color: #f59e0b;
}
.mini-filter-btn:hover:not(.active) {
  background: color-mix(in srgb, var(--text-soft) 10%, transparent);
}

.mini-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 6px 6px;
}

.mini-hint {
  text-align: center;
  color: var(--text-soft);
  font-size: 12px;
  padding: 20px 0;
}

.mini-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  padding: 5px 6px;
  border-radius: 8px;
  cursor: default;
  transition: background 0.15s;
}
.mini-item:hover {
  background: color-mix(in srgb, var(--text-soft) 8%, transparent);
}
.mini-item + .mini-item {
  border-top: 1px solid color-mix(in srgb, var(--text-soft) 6%, transparent);
}

.mini-item-main {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
}

.mini-item-top {
  display: flex;
  align-items: center;
  gap: 5px;
}

.mini-type-badge {
  font-size: 9px;
  font-weight: 600;
  padding: 0 5px;
  border-radius: 3px;
  line-height: 1.5;
  flex-shrink: 0;
}

.mini-time {
  font-size: 9px;
  color: var(--text-soft);
  opacity: 0.7;
}

.mini-item-name-row {
  display: flex;
  align-items: center;
  gap: 3px;
  min-width: 0;
}

.mini-item-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mini-star {
  color: #f59e0b;
  font-size: 10px;
  flex-shrink: 0;
}

.mini-item-path {
  font-size: 10px;
  color: var(--text-soft);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mini-tags {
  display: flex;
  gap: 3px;
  flex-wrap: wrap;
}

.mini-tag {
  font-size: 9px;
  padding: 0 4px;
  border-radius: 3px;
  background: color-mix(in srgb, var(--accent, #2563eb) 12%, transparent);
  color: var(--accent, #2563eb);
  line-height: 1.5;
}

.mini-tag.more {
  background: color-mix(in srgb, var(--text-soft) 10%, transparent);
  color: var(--text-soft);
}

.mini-item-actions {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
}

.mini-ide-icon {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  overflow: hidden;
  background: color-mix(in srgb, var(--text-soft) 8%, transparent);
}

.ide-img {
  width: 14px;
  height: 14px;
  object-fit: contain;
}

.ide-fallback {
  font-size: 9px;
  font-weight: 700;
  color: var(--text-soft);
}

.mini-act-btn {
  border: none;
  background: transparent;
  color: var(--text-soft);
  cursor: pointer;
  font-size: 11px;
  padding: 2px 3px;
  border-radius: 4px;
  line-height: 1;
  transition: all 0.15s;
}
.mini-act-btn:hover {
  background: color-mix(in srgb, var(--text-soft) 14%, transparent);
  color: var(--text);
}

.mini-launch-btn {
  border: none;
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  color: var(--accent);
  cursor: pointer;
  padding: 2px 5px;
  border-radius: 4px;
  font-size: 10px;
  flex-shrink: 0;
  transition: background 0.15s;
}
.mini-launch-btn:hover {
  background: color-mix(in srgb, var(--accent) 22%, transparent);
}

.mini-list::-webkit-scrollbar {
  width: 3px;
}
.mini-list::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--text-soft) 30%, transparent);
  border-radius: 2px;
}
</style>
