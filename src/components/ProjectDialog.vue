<script setup lang="ts">
import type { ProjectForm } from "../types/project";

defineProps<{
  visible: boolean;
  form: ProjectForm;
}>();

defineEmits<{
  close: [];
  submit: [];
  chooseFolders: [];
  "update:maxDepth": [value: number];
}>();
</script>

<template>
  <div v-if="visible" class="dialog-mask" @click.self="$emit('close')">
    <form class="dialog glass" @submit.prevent="$emit('submit')">
      <h2>扫描导入项目</h2>
      <button type="button" class="btn ghost" @click="$emit('chooseFolders')">
        选择扫描根目录
      </button>
      <div class="folder-list">
        <p v-if="!form.path" class="hint">尚未选择目录</p>
        <p v-else class="path-item">{{ form.path }}</p>
      </div>
      <label class="checkline">
        扫描深度
        <input
          :value="form.maxDepth"
          class="input depth-input"
          type="number"
          min="1"
          max="8"
          @input="$emit('update:maxDepth', Number(($event.target as HTMLInputElement).value || 3))"
        />
      </label>
      <div class="dialog-actions">
        <button type="button" class="btn ghost" @click="$emit('close')">取消</button>
        <button type="submit" class="btn primary">开始扫描</button>
      </div>
    </form>
  </div>
</template>
