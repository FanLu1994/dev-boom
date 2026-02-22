<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Project, IdeConfig } from "../types/project";
import { TYPE_COLORS } from "../types/project";
import { useTheme } from "../composables/useTheme";

const { applyTheme } = useTheme();

const projects = ref<Project[]>([]);
const ides = ref<IdeConfig[]>([]);
const searchText = ref("");
const loading = ref(true);

const appWindow = getCurrentWindow();

const filteredProjects = computed(() => {
  const q = searchText.value.trim().toLowerCase();
  let list = [...projects.value];
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
  return list.slice(0, 10);
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
      </div>
      <button class="mini-close" @click="hideMini" title="隐藏">✕</button>
    </header>

    <div class="mini-search">
      <input
        v-model="searchText"
        type="text"
        placeholder="搜索项目…"
        class="mini-search-input"
      />
    </div>

    <div class="mini-list">
      <p v-if="loading" class="mini-hint">加载中...</p>
      <p v-else-if="!filteredProjects.length" class="mini-hint">无匹配项目</p>
      <div
        v-for="project in filteredProjects"
        :key="project.id"
        class="mini-item"
      >
        <div class="mini-item-info">
          <span
            class="mini-type-dot"
            :style="{ background: TYPE_COLORS[project.projectType] }"
          ></span>
          <div class="mini-item-text">
            <span class="mini-item-name">
              {{ project.name }}
              <span v-if="project.favorite" class="mini-star">★</span>
            </span>
            <span class="mini-item-path">{{ project.path }}</span>
          </div>
        </div>
        <button
          class="mini-launch-btn"
          @click.stop="quickLaunch(project)"
          :title="getPreferredIde(project)?.name ?? '启动'"
        >
          ▶
        </button>
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
  border-radius: 14px;
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
  padding: 10px 12px 6px;
  flex-shrink: 0;
}

.mini-brand {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mini-dot {
  width: 10px;
  height: 10px;
  border-radius: 999px;
  background: #2563eb;
}

.mini-title {
  font-family: "Manrope", sans-serif;
  font-size: 14px;
  font-weight: 700;
  color: var(--text);
}

.mini-close {
  border: none;
  background: transparent;
  color: var(--text-soft);
  cursor: pointer;
  font-size: 14px;
  padding: 2px 6px;
  border-radius: 6px;
  line-height: 1;
}
.mini-close:hover {
  background: color-mix(in srgb, var(--text-soft) 14%, transparent);
  color: var(--text);
}

.mini-search {
  padding: 4px 12px 8px;
  flex-shrink: 0;
}

.mini-search-input {
  width: 100%;
  padding: 7px 10px;
  border-radius: 8px;
  border: 1px solid color-mix(in srgb, var(--text-soft) 20%, transparent);
  background: color-mix(in srgb, var(--panel) 60%, transparent);
  color: var(--text);
  font-size: 13px;
  outline: none;
  font-family: inherit;
}
.mini-search-input:focus {
  border-color: #2563eb;
}

.mini-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px 8px;
}

.mini-hint {
  text-align: center;
  color: var(--text-soft);
  font-size: 13px;
  padding: 24px 0;
}

.mini-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 7px 8px;
  border-radius: 8px;
  cursor: default;
  transition: background 0.15s;
}
.mini-item:hover {
  background: color-mix(in srgb, var(--text-soft) 8%, transparent);
}

.mini-item-info {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex: 1;
}

.mini-type-dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  flex-shrink: 0;
}

.mini-item-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.mini-item-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mini-star {
  color: #f59e0b;
  font-size: 11px;
  margin-left: 3px;
}

.mini-item-path {
  font-size: 11px;
  color: var(--text-soft);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mini-launch-btn {
  border: none;
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  color: var(--accent);
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 12px;
  flex-shrink: 0;
  transition: background 0.15s;
}
.mini-launch-btn:hover {
  background: color-mix(in srgb, var(--accent) 18%, transparent);
}

.mini-list::-webkit-scrollbar {
  width: 4px;
}
.mini-list::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--text-soft) 30%, transparent);
  border-radius: 2px;
}
</style>
