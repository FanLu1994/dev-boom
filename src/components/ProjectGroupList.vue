<script setup lang="ts">
import ProjectCard from "./ProjectCard.vue";
import type { IdeConfig, Project } from "../types/project";

defineProps<{
  projects: Project[];
  ides: IdeConfig[];
  formatLastModified: (value: string | null) => string;
}>();

defineEmits<{
  toggleFavorite: [projectId: string];
  remove: [projectId: string];
  launch: [project: Project];
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
        :format-last-modified="formatLastModified"
        @toggle-favorite="(projectId) => $emit('toggleFavorite', projectId)"
        @remove="(projectId) => $emit('remove', projectId)"
        @launch="(project) => $emit('launch', project)"
        @open-folder="(path) => $emit('openFolder', path)"
      />
    </div>
  </section>
</template>
