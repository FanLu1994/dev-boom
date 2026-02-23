<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ThemeMode } from "../types/project";
import { IconMinus, IconSquare, IconX, IconMoon, IconSun, IconRepeat } from "@tabler/icons-vue";

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

async function switchToMini() {
  try {
    await invoke("switch_to_mini_window");
  } catch {
    // noop
  }
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
          <button class="quick-add-item primary" @click="openIde">IDE 管理</button>
        </div>
      </div>
      <button
        class="icon-pill"
        aria-label="切换到小窗口"
        title="切换到小窗口"
        @click="switchToMini"
      >
        <IconRepeat :size="14" />
      </button>
      <button
        class="icon-pill"
        :aria-label="theme === 'light' ? '切换到深色主题' : '切换到浅色主题'"
        :title="theme === 'light' ? '切换到深色主题' : '切换到浅色主题'"
        @click="$emit('toggleTheme')"
      >
        <component :is="theme === 'light' ? IconMoon : IconSun" :size="14" />
      </button>
      <button class="icon-pill" @click="$emit('minimize')">
        <IconMinus :size="14" />
      </button>
      <button class="icon-pill" @click="$emit('maximize')">
        <IconSquare :size="14" />
      </button>
      <button class="icon-pill danger" @click="$emit('close')">
        <IconX :size="14" />
      </button>
    </div>
  </header>
</template>
