<script setup lang="ts">
import { ref } from "vue";
import type { IdeCategory, IdeConfig, IdeForm } from "../types/project";

defineProps<{
  visible: boolean;
  form: IdeForm;
  ides: IdeConfig[];
}>();

defineEmits<{
  close: [];
  submit: [];
  scan: [];
  setIcon: [ideId: string];
  chooseExecutable: [];
  "update:name": [value: string];
  "update:executable": [value: string];
  "update:argsTemplate": [value: string];
  "update:category": [value: IdeCategory];
  "update:priority": [value: number];
}>();

const brokenIconIds = ref<Record<string, boolean>>({});

function markIconBroken(ideId: string) {
  brokenIconIds.value[ideId] = true;
}

function ideShortName(name: string) {
  return name.trim().slice(0, 1).toUpperCase();
}
</script>

<template>
  <div v-if="visible" class="dialog-mask" @click.self="$emit('close')">
    <div class="dialog glass ide-manager-dialog">
      <div class="ide-manager-header">
        <h2>IDE 管理</h2>
        <button type="button" class="btn ghost small" @click="$emit('scan')">扫描 IDE</button>
      </div>

      <div class="ide-table-wrap">
        <table class="ide-table">
          <thead>
            <tr>
              <th>图标</th>
              <th>名称</th>
              <th>类型</th>
              <th>优先级</th>
              <th>可执行路径</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="ide in ides" :key="ide.id">
              <td>
                <img
                  v-if="ide.icon && !brokenIconIds[ide.id]"
                  :src="ide.icon"
                  :alt="ide.name"
                  class="ide-icon"
                  @error="markIconBroken(ide.id)"
                />
                <span v-else class="ide-fallback">{{ ideShortName(ide.name) }}</span>
              </td>
              <td>{{ ide.name }}</td>
              <td>{{ ide.category }}</td>
              <td>{{ ide.priority }}</td>
              <td class="ide-executable">{{ ide.executable }}</td>
              <td>
                <button type="button" class="btn ghost small" @click="$emit('setIcon', ide.id)">设置图标</button>
              </td>
            </tr>
            <tr v-if="!ides.length">
              <td colspan="6" class="ide-empty-row">暂无 IDE，点击“扫描 IDE”或手动添加。</td>
            </tr>
          </tbody>
        </table>
      </div>

      <form class="ide-create-form" @submit.prevent="$emit('submit')">
        <h3>手动添加 IDE</h3>
        <input :value="form.name" class="input" placeholder="IDE 名称（如 VSCode）" required @input="$emit('update:name', ($event.target as HTMLInputElement).value)" />
        <button type="button" class="btn ghost" @click="$emit('chooseExecutable')">从文件选择可执行程序</button>
        <input
          :value="form.executable"
          class="input"
          placeholder="尚未选择可执行文件"
          required
          readonly
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
          <button type="submit" class="btn primary">添加 IDE</button>
        </div>
      </form>

      <div class="dialog-actions">
        <button type="button" class="btn ghost" @click="$emit('close')">取消</button>
      </div>
    </div>
  </div>
</template>
