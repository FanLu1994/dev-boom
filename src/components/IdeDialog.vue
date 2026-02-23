<script setup lang="ts">
import { ref } from "vue";
import type { IdeCategory, IdeConfig, IdeForm } from "../types/project";
import { IconSearch, IconFolder, IconSettings, IconPlus, IconDeviceDesktop, IconTerminal, IconBrowser, IconCode, IconX } from "@tabler/icons-vue";

defineProps<{
  visible: boolean;
  form: IdeForm;
  ides: IdeConfig[];
}>();

defineEmits<{
  close: [];
  submit: [];
  scan: [];
  remove: [ideId: string];
  setIcon: [ideId: string];
  chooseExecutable: [];
  "update:name": [value: string];
  "update:executable": [value: string];
  "update:argsTemplate": [value: string];
  "update:category": [value: IdeCategory];
  "update:priority": [value: number];
}>();

const showAddDialog = ref(false);

const brokenIconIds = ref<Record<string, boolean>>({});

function markIconBroken(ideId: string) {
  brokenIconIds.value[ideId] = true;
}

function ideShortName(name: string) {
  return name.trim().slice(0, 1).toUpperCase();
}

function openAddDialog() {
  showAddDialog.value = true;
}

function closeAddDialog() {
  showAddDialog.value = false;
}

function getCategoryIcon(category: IdeCategory) {
  switch (category) {
    case 'Gui':
      return IconDeviceDesktop;
    case 'Cli':
      return IconCode;
    case 'Terminal':
      return IconTerminal;
    case 'Browser':
      return IconBrowser;
    default:
      return IconSettings;
  }
}

function getCategoryLabel(category: IdeCategory) {
  const labels = {
    'Gui': '桌面应用',
    'Cli': '命令行',
    'Terminal': '终端',
    'Browser': '浏览器'
  };
  return labels[category] || category;
}

function getCategoryColor(category: IdeCategory) {
  const colors = {
    'Gui': 'color-mix(in srgb, #3b82f6 15%, transparent)',
    'Cli': 'color-mix(in srgb, #10b981 15%, transparent)',
    'Terminal': 'color-mix(in srgb, #f59e0b 15%, transparent)',
    'Browser': 'color-mix(in srgb, #8b5cf6 15%, transparent)'
  };
  return colors[category] || 'color-mix(in srgb, var(--text-soft) 10%, transparent)';
}
</script>

<template>
  <!-- 主对话框：IDE 列表 -->
  <div v-if="visible" class="dialog-mask" @click.self="$emit('close')">
    <div class="dialog glass ide-manager-dialog">
      <div class="ide-manager-header">
        <div class="header-left">
          <h2>IDE 管理</h2>
          <span class="header-subtitle">{{ ides.length }} 个 IDE 已配置</span>
        </div>
        <button type="button" class="icon-btn close-btn" @click="$emit('close')" title="关闭">
          <IconX :size="16" />
        </button>
      </div>

      <div class="ide-manager-actions">
        <button type="button" class="btn ghost small" @click="$emit('scan')">
          <IconSearch :size="14" style="margin-right: 4px;" />
          扫描 IDE
        </button>
        <button type="button" class="btn primary small" @click="openAddDialog">
          <IconPlus :size="14" style="margin-right: 4px;" />
          添加 IDE
        </button>
      </div>

      <!-- IDE 卡片网格 -->
      <div class="ide-grid-container">
        <div v-if="ides.length" class="ide-grid">
          <div
            v-for="ide in ides"
            :key="ide.id"
            class="ide-card"
          >
            <div class="ide-card-header">
              <div class="ide-icon-wrapper">
                <img
                  v-if="ide.icon && !brokenIconIds[ide.id]"
                  :src="ide.icon"
                  :alt="ide.name"
                  class="ide-card-icon"
                  @error="markIconBroken(ide.id)"
                />
                <span v-else class="ide-fallback">{{ ideShortName(ide.name) }}</span>
              </div>
              <button
                type="button"
                class="ide-delete-btn"
                @click="$emit('remove', ide.id)"
                title="删除 IDE"
              >
                <IconX :size="14" />
              </button>
            </div>

            <div class="ide-card-body">
              <h4 class="ide-card-name">{{ ide.name }}</h4>

              <div class="ide-card-meta">
                <span
                  class="ide-category-tag"
                  :style="{ background: getCategoryColor(ide.category) }"
                >
                  <component :is="getCategoryIcon(ide.category)" :size="11" style="margin-right: 4px;" />
                  {{ getCategoryLabel(ide.category) }}
                </span>
                <span class="ide-priority-tag">优先级 {{ ide.priority }}</span>
              </div>

              <p class="ide-card-path" :title="ide.executable">{{ ide.executable }}</p>
            </div>

            <div class="ide-card-footer">
              <button
                type="button"
                class="btn ghost small full-width"
                @click="$emit('setIcon', ide.id)"
              >
                <IconSettings :size="13" style="margin-right: 4px;" />
                设置图标
              </button>
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-else class="empty-state-large">
          <IconDeviceDesktop :size="48" style="opacity: 0.2;" />
          <h3>暂无 IDE</h3>
          <p>点击"扫描 IDE"自动检测，或点击"添加 IDE"手动添加开发工具</p>
        </div>
      </div>
    </div>
  </div>

  <!-- 二级弹窗：添加 IDE -->
  <div v-if="showAddDialog" class="dialog-mask dialog-mask-secondary" @click.self="closeAddDialog">
    <div class="dialog glass ide-add-dialog">
      <div class="dialog-header">
        <h3>添加 IDE</h3>
        <button type="button" class="btn ghost small" @click="closeAddDialog" title="关闭">
          <IconX :size="16" />
        </button>
      </div>

      <form class="ide-add-form" @submit.prevent="(closeAddDialog(), $emit('submit'))">
        <div class="form-group">
          <label class="form-label">IDE 名称</label>
          <input
            :value="form.name"
            class="input"
            placeholder="例如：VSCode, WebStorm, IntelliJ IDEA"
            required
            @input="$emit('update:name', ($event.target as HTMLInputElement).value)"
          />
        </div>

        <div class="form-group">
          <label class="form-label">可执行文件</label>
          <button type="button" class="btn ghost full-width" @click="$emit('chooseExecutable')">
            <IconFolder :size="14" style="margin-right: 4px;" />
            {{ form.executable ? '更换文件' : '选择可执行文件' }}
          </button>
          <input
            v-if="form.executable"
            :value="form.executable"
            class="input"
            style="margin-top: 8px;"
            readonly
          />
        </div>

        <div class="form-row-inline">
          <div class="form-group" style="flex: 1;">
            <label class="form-label">类型</label>
            <select
              :value="form.category"
              class="select"
              @change="$emit('update:category', ($event.target as HTMLSelectElement).value as IdeCategory)"
            >
              <option value="Gui">桌面应用</option>
              <option value="Cli">命令行</option>
              <option value="Terminal">终端</option>
              <option value="Browser">浏览器</option>
            </select>
          </div>
          <div class="form-group" style="flex: 1;">
            <label class="form-label">优先级</label>
            <input
              :value="form.priority"
              class="input"
              type="number"
              placeholder="越小越优先"
              @input="$emit('update:priority', Number(($event.target as HTMLInputElement).value))"
            />
          </div>
        </div>

        <div class="form-group">
          <label class="form-label">参数模板（可选）</label>
          <input
            :value="form.argsTemplate"
            class="input"
            placeholder="例如：{projectPath}"
            @input="$emit('update:argsTemplate', ($event.target as HTMLInputElement).value)"
          />
          <p class="form-hint">可用变量：{projectPath} - 项目路径</p>
        </div>

        <div class="dialog-actions">
          <button type="button" class="btn ghost" @click="closeAddDialog">取消</button>
          <button type="submit" class="btn primary" :disabled="!form.name || !form.executable">
            <IconPlus :size="14" style="margin-right: 4px;" />
            添加 IDE
          </button>
        </div>
      </form>
    </div>
  </div>
</template>
