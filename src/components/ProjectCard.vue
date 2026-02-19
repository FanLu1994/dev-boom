<script setup lang="ts">
import type { IdeConfig, Project } from "../types/project";

defineProps<{
  project: Project;
  ides: IdeConfig[];
  selectedIdeLabel: string;
  formatLastModified: (value: string | null) => string;
}>();

defineEmits<{
  toggleFavorite: [projectId: string];
  remove: [projectId: string];
  launch: [project: Project, ideId?: string];
  openFolder: [path: string];
}>();
</script>

<template>
  <article class="card glass">
    <div class="card-top">
      <span class="project-bullet" :style="{ background: project.favorite ? '#f59e0b' : '#2563eb' }"></span>
      <button class="icon-btn" @click="$emit('toggleFavorite', project.id)">{{ project.favorite ? "★" : "☆" }}</button>
    </div>

    <h3 class="project-name">{{ project.name }}</h3>
    <p class="path">{{ project.path }}</p>
    <p class="meta">最近修改：{{ formatLastModified(project.lastModified) }}</p>
    <p v-if="project.metadata.description" class="desc">{{ project.metadata.description }}</p>

    <div v-if="project.tags.length" class="tags">
      <span v-for="tag in project.tags" :key="tag" class="mini-tag">{{ tag }}</span>
    </div>

    <div class="card-footer">
      <button class="btn primary small" @click="$emit('launch', project)">启动</button>
      <button class="btn ghost small" @click="$emit('openFolder', project.path)">文件夹</button>
      <button class="btn ghost small danger-text" @click="$emit('remove', project.id)">移除</button>
    </div>

    <div class="quick-ide">
      <label>默认IDE</label>
      <select
        class="select"
        @change="$emit('launch', project, (($event.target as HTMLSelectElement).value || undefined) as string | undefined)"
      >
        <option value="">使用默认（{{ selectedIdeLabel }}）</option>
        <option v-for="ide in ides" :key="ide.id" :value="ide.id">{{ ide.name }}</option>
      </select>
    </div>
  </article>
</template>
