<script setup lang="ts">
import type { IdeCategory, IdeForm } from "../types/project";

defineProps<{
  visible: boolean;
  form: IdeForm;
}>();

defineEmits<{
  close: [];
  submit: [];
  "update:name": [value: string];
  "update:executable": [value: string];
  "update:argsTemplate": [value: string];
  "update:category": [value: IdeCategory];
  "update:priority": [value: number];
}>();
</script>

<template>
  <div v-if="visible" class="dialog-mask" @click.self="$emit('close')">
    <form class="dialog glass" @submit.prevent="$emit('submit')">
      <h2>添加 IDE</h2>
      <input :value="form.name" class="input" placeholder="IDE 名称（如 VSCode）" required @input="$emit('update:name', ($event.target as HTMLInputElement).value)" />
      <input
        :value="form.executable"
        class="input"
        placeholder="可执行命令或绝对路径（如 code）"
        required
        @input="$emit('update:executable', ($event.target as HTMLInputElement).value)"
      />
      <input
        :value="form.argsTemplate"
        class="input"
        placeholder="参数模板，如 {projectPath}"
        @input="$emit('update:argsTemplate', ($event.target as HTMLInputElement).value)"
      />
      <select :value="form.category" class="select" @change="$emit('update:category', ($event.target as HTMLSelectElement).value as IdeCategory)">
        <option value="Gui">GUI</option>
        <option value="Cli">CLI</option>
        <option value="Terminal">Terminal</option>
        <option value="Browser">Browser</option>
      </select>
      <input
        :value="form.priority"
        class="input"
        type="number"
        placeholder="优先级，越小越优先"
        @input="$emit('update:priority', Number(($event.target as HTMLInputElement).value))"
      />
      <div class="dialog-actions">
        <button type="button" class="btn ghost" @click="$emit('close')">取消</button>
        <button type="submit" class="btn primary">保存</button>
      </div>
    </form>
  </div>
</template>
