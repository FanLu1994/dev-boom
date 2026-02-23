<script setup lang="ts">
import { computed, ref } from "vue";
import type { IdeConfig, Project } from "../types/project";
import { IconStar, IconStarFilled, IconFolder, IconTerminal, IconTrash, IconCode } from "@tabler/icons-vue";

defineEmits<{
  toggleFavorite: [projectId: string];
  remove: [projectId: string];
  launch: [project: Project];
  openFolder: [path: string];
  openTerminal: [path: string];
  showLanguageStats: [projectId: string];
}>();

const props = defineProps<{
  project: Project;
  ides: IdeConfig[];
  formatLastModified: (value: string | null) => string;
}>();

const selectedIdeConfigs = computed(() =>
  props.project.metadata.idePreferences
    .map((id) => props.ides.find((ide) => ide.id === id))
    .filter((ide): ide is IdeConfig => Boolean(ide))
    .slice(0, 3)
);
const brokenIconIds = ref<Record<string, boolean>>({});

function ideShortName(name: string) {
  return name.trim().slice(0, 1).toUpperCase();
}

function markIconBroken(ideId: string) {
  brokenIconIds.value[ideId] = true;
}

function getLanguageSummary() {
  const stats = props.project.metadata.languageStats;
  if (!stats || stats.languages.length === 0) return null;

  const topLanguages = stats.languages.slice(0, 3);
  return topLanguages.map(lang => `${lang.language} ${lang.percentage.toFixed(0)}%`).join(" · ");
}
</script>

<template>
  <article
    class="card glass card-clickable"
    role="button"
    tabindex="0"
    @click="$emit('launch', project)"
    @keydown.enter.prevent="$emit('launch', project)"
    @keydown.space.prevent="$emit('launch', project)"
  >
    <div class="card-top">
      <span class="project-bullet" :style="{ background: project.favorite ? '#f59e0b' : '#2563eb' }"></span>
      <button class="icon-btn" @click.stop="$emit('toggleFavorite', project.id)">
        <component :is="project.favorite ? IconStarFilled : IconStar" :size="16" />
      </button>
    </div>

    <h3 class="project-name">{{ project.name }}</h3>
    <p class="path">{{ project.path }}</p>
    <p class="meta">最近修改：{{ formatLastModified(project.lastModified) }}</p>
    <p v-if="project.metadata.description" class="desc">{{ project.metadata.description }}</p>

    <div v-if="project.tags.length" class="tags">
      <span v-for="tag in project.tags" :key="tag" class="mini-tag">{{ tag }}</span>
    </div>

    <div class="card-footer">
      <button class="btn primary small" @click.stop="$emit('launch', project)">打开</button>
      <button class="btn ghost small" @click.stop="$emit('openTerminal', project.path)" title="终端">
        <IconTerminal :size="15" />
      </button>
      <button class="btn ghost small" @click.stop="$emit('openFolder', project.path)" title="文件夹">
        <IconFolder :size="15" />
      </button>
      <button
        class="btn ghost small"
        @click.stop="$emit('showLanguageStats', project.id)"
        title="语言统计"
      >
        <IconCode :size="15" />
      </button>
      <button class="btn ghost small danger-text" @click.stop="$emit('remove', project.id)" title="移除">
        <IconTrash :size="15" />
      </button>
    </div>

    <div v-if="getLanguageSummary()" class="language-summary">
      <IconCode :size="12" />
      <span>{{ getLanguageSummary() }}</span>
    </div>

    <div class="quick-ide compact">
      <label>上次使用的 IDE</label>
      <div class="ide-selected-row">
        <div v-for="ide in selectedIdeConfigs" :key="ide.id" class="ide-pill" :title="ide.name">
          <img
            v-if="ide.icon && !brokenIconIds[ide.id]"
            :src="ide.icon"
            :alt="ide.name"
            class="ide-icon"
            @error="markIconBroken(ide.id)"
          />
          <span v-else class="ide-fallback">{{ ideShortName(ide.name) }}</span>
        </div>
        <span v-if="!selectedIdeConfigs.length" class="ide-empty">未设置</span>
      </div>
    </div>
  </article>
</template>

<style scoped>
.language-summary {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  margin-top: 8px;
  background: var(--bg-secondary);
  border-radius: 6px;
  font-size: 12px;
  color: var(--fg-muted);
}

.language-summary svg {
  flex-shrink: 0;
  opacity: 0.7;
}

.language-summary span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
