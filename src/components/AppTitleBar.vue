<script setup lang="ts">
import { ref } from "vue";
import type { ThemeMode } from "../types/project";

defineProps<{
  theme: ThemeMode;
}>();

const showQuickAdd = ref(false);

function toggleQuickAdd() {
  showQuickAdd.value = !showQuickAdd.value;
}

function openProject() {
  showQuickAdd.value = false;
  emit("openProjectDialog");
}

function openIde() {
  showQuickAdd.value = false;
  emit("openIdeDialog");
}

const emit = defineEmits<{
  toggleTheme: [];
  minimize: [];
  maximize: [];
  close: [];
  dragStart: [event: MouseEvent];
  openProjectDialog: [];
  openIdeDialog: [];
}>();
</script>

<template>
  <header class="titlebar" data-tauri-drag-region @mousedown.left="$emit('dragStart', $event)">
    <div class="title-left" data-tauri-drag-region>
      <span class="brand-dot"></span>
      <div>
        <p class="brand-sub">PROJECT MANAGER</p>
        <h1>项目管理器</h1>
      </div>
    </div>

    <div class="title-actions">
      <div class="quick-add-wrap">
        <button class="icon-pill" @click="toggleQuickAdd">+</button>
        <div v-if="showQuickAdd" class="quick-add-panel">
          <button class="quick-add-item" @click="openProject">添加项目</button>
          <button class="quick-add-item" @click="openIde">添加 IDE</button>
        </div>
      </div>
      <button class="icon-pill" @click="$emit('toggleTheme')">{{ theme === "light" ? "深" : "浅" }}</button>
      <button class="icon-pill" @click="$emit('minimize')">-</button>
      <button class="icon-pill" @click="$emit('maximize')">□</button>
      <button class="icon-pill danger" @click="$emit('close')">×</button>
    </div>
  </header>
</template>
