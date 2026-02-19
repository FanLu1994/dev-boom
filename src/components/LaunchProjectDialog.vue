<script setup lang="ts">
import type { IdeConfig, Project } from "../types/project";

const props = defineProps<{
  visible: boolean;
  project: Project | null;
  ides: IdeConfig[];
  selectedIdeIds: string[];
}>();

const emit = defineEmits<{
  close: [];
  confirm: [];
  "update:selectedIdeIds": [value: string[]];
}>();

function toggleIde(ideId: string) {
  if (props.selectedIdeIds.includes(ideId)) {
    emit(
      "update:selectedIdeIds",
      props.selectedIdeIds.filter((id) => id !== ideId),
    );
    return;
  }
  if (props.selectedIdeIds.length >= 3) return;
  emit("update:selectedIdeIds", [...props.selectedIdeIds, ideId]);
}

function ideShortName(name: string) {
  return name.trim().slice(0, 1).toUpperCase();
}
</script>

<template>
  <div v-if="visible" class="dialog-mask" @click.self="$emit('close')">
    <form class="dialog glass" @submit.prevent="$emit('confirm')">
      <h2>打开项目</h2>
      <p class="hint">{{ project?.name ?? "未选择项目" }}</p>
      <p class="hint">选择 IDE（最多 3 个）：{{ selectedIdeIds.length }}/3</p>

      <div class="ide-tile-grid">
        <button
          v-for="ide in ides"
          :key="ide.id"
          type="button"
          class="ide-tile"
          :class="{ selected: selectedIdeIds.includes(ide.id), disabled: !selectedIdeIds.includes(ide.id) && selectedIdeIds.length >= 3 }"
          :disabled="!selectedIdeIds.includes(ide.id) && selectedIdeIds.length >= 3"
          @click="toggleIde(ide.id)"
        >
          <img v-if="ide.icon" :src="ide.icon" :alt="ide.name" class="ide-icon" />
          <span v-else class="ide-fallback">{{ ideShortName(ide.name) }}</span>
          <span class="ide-tile-name">{{ ide.name }}</span>
          <span v-if="selectedIdeIds.includes(ide.id)" class="ide-tile-check">✓</span>
        </button>
      </div>

      <div class="dialog-actions">
        <button type="button" class="btn ghost" @click="$emit('close')">取消</button>
        <button type="submit" class="btn primary">一键打开</button>
      </div>
    </form>
  </div>
</template>
