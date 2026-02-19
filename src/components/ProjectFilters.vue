<script setup lang="ts">
import type { ProjectType } from "../types/project";

defineProps<{
  selectedType: "All" | ProjectType;
  projectTypes: ProjectType[];
}>();

defineEmits<{
  "update:selectedType": [value: "All" | ProjectType];
  openIdeDialog: [];
  openProjectDialog: [];
}>();
</script>

<template>
  <section class="top-actions">
    <div class="filters">
      <button class="chip" :class="{ active: selectedType === 'All' }" @click="$emit('update:selectedType', 'All')">
        全部
      </button>
      <button
        v-for="type in projectTypes"
        :key="type"
        class="chip"
        :class="{ active: selectedType === type }"
        @click="$emit('update:selectedType', type)"
      >
        {{ type }}
      </button>
    </div>

    <div class="action-row">
      <button class="btn ghost" @click="$emit('openIdeDialog')">+ 添加 IDE</button>
      <button class="btn primary" @click="$emit('openProjectDialog')">+ 添加项目</button>
    </div>
  </section>
</template>
