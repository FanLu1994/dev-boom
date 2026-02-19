<script setup lang="ts">
import ProjectCard from "./ProjectCard.vue";
import type { IdeConfig, Project } from "../types/project";

defineProps<{
  projects: Project[];
  ides: IdeConfig[];
  selectedIdeLabel: (project: Project) => string;
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
  <section class="group-wrap">
    <div class="grid">
      <ProjectCard
        v-for="project in projects"
        :key="project.id"
        :project="project"
        :ides="ides"
        :selected-ide-label="selectedIdeLabel(project)"
        :format-last-modified="formatLastModified"
        @toggle-favorite="(projectId) => $emit('toggleFavorite', projectId)"
        @remove="(projectId) => $emit('remove', projectId)"
        @launch="(project, ideId) => $emit('launch', project, ideId)"
        @open-folder="(path) => $emit('openFolder', path)"
      />
    </div>
  </section>
</template>
