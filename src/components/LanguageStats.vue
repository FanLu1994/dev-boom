<script setup lang="ts">
import { computed } from "vue";
import type { LanguageStats } from "../types/project";
import { IconCode, IconRefresh, IconX } from "@tabler/icons-vue";

const props = defineProps<{
  projectId: string;
  stats: LanguageStats | null;
  loading?: boolean;
}>();

const emit = defineEmits<{
  refresh: [projectId: string];
  close: [];
}>();

// 语言颜色映射
const LANGUAGE_COLORS: Record<string, string> = {
  "Rust": "#f97316",
  "JavaScript": "#f7df1e",
  "TypeScript": "#3178c6",
  "Python": "#2563eb",
  "Java": "#dc2626",
  "Go": "#0891b2",
  "C": "#555555",
  "C++": "#00599c",
  "C#": "#178600",
  "HTML": "#e34c26",
  "CSS": "#563d7c",
  "Vue": "#42b883",
  "Shell": "#89e051",
  "PowerShell": "#5391fe",
  "JSON": "#f7df1e",
  "YAML": "#cb171e",
  "TOML": "#9c4221",
  "XML": "#0060ac",
  "Markdown": "#083fa1",
  "SQL": "#cc2927",
  "Ruby": "#701516",
  "PHP": "#777bb4",
  "Swift": "#f05138",
  "Dart": "#0175c2",
  "R": "#198ce7",
  "Scala": "#dc322f",
  "Lua": "#000080",
  "Elixir": "#6e4a7e",
  "Erlang": "#b83998",
  "F#": "#b845fc",
  "Svelte": "#ff3e00",
};

const languageColor = computed(() => (lang: string) => {
  return LANGUAGE_COLORS[lang] || "#64748b";
});

const formatDate = computed(() => (dateStr: string) => {
  if (!dateStr) return "";
  const date = new Date(dateStr);
  return date.toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
});

function handleRefresh() {
  emit("refresh", props.projectId);
}

function handleClose() {
  emit("close");
}
</script>

<template>
  <div class="language-stats">
    <div class="stats-header">
      <div class="header-title">
        <IconCode :size="20" />
        <h3>开发语言统计</h3>
      </div>
      <div class="header-actions">
        <button
          class="btn ghost small"
          @click="handleRefresh"
          :disabled="loading"
          title="刷新统计"
        >
          <IconRefresh :size="16" :class="{ spinning: loading }" />
        </button>
        <button
          class="btn ghost small"
          @click="handleClose"
          title="关闭"
        >
          <IconX :size="16" />
        </button>
      </div>
    </div>

    <div v-if="loading" class="stats-loading">
      <div class="spinner"></div>
      <p>正在统计语言分布...</p>
    </div>

    <div v-else-if="!stats" class="stats-empty">
      <p>暂无语言统计数据</p>
      <button class="btn primary small" @click="handleRefresh">
        <IconRefresh :size="16" />
        开始统计
      </button>
    </div>

    <div v-else class="stats-content">
      <div class="stats-summary">
        <div class="summary-item">
          <span class="label">总行数</span>
          <span class="value">{{ stats.totalLines.toLocaleString() }} 行</span>
        </div>
        <div class="summary-item">
          <span class="label">扫描时间</span>
          <span class="value">{{ formatDate(stats.scannedAt) }}</span>
        </div>
      </div>

      <div class="language-list">
        <div
          v-for="lang in stats.languages"
          :key="lang.language"
          class="language-item"
        >
          <div class="language-info">
            <span
              class="language-dot"
              :style="{ backgroundColor: languageColor(lang.language) }"
            ></span>
            <span class="language-name">{{ lang.language }}</span>
            <span class="language-meta">{{ lang.files }} 个文件</span>
          </div>
          <div class="language-bar-container">
            <div class="language-bar-bg">
              <div
                class="language-bar-fill"
                :style="{
                  width: `${lang.percentage}%`,
                  backgroundColor: languageColor(lang.language)
                }"
              ></div>
            </div>
            <div class="language-stats">
              <span class="lines">{{ lang.lines.toLocaleString() }} 行</span>
              <span class="percentage">{{ lang.percentage.toFixed(1) }}%</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.language-stats {
  background: var(--bg-card);
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.stats-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-title h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--fg);
}

.header-actions {
  display: flex;
  gap: 4px;
}

.stats-loading,
.stats-empty {
  text-align: center;
  padding: 40px 20px;
  color: var(--fg-muted);
}

.stats-loading .spinner {
  width: 32px;
  height: 32px;
  margin: 0 auto 16px;
  border: 3px solid var(--border);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.spinning {
  animation: spin 0.8s linear infinite;
}

.stats-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.stats-summary {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 12px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
}

.summary-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.summary-item .label {
  font-size: 12px;
  color: var(--fg-muted);
}

.summary-item .value {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg);
}

.language-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.language-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
  transition: transform 0.2s;
}

.language-item:hover {
  transform: translateX(2px);
}

.language-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.language-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.language-name {
  font-weight: 600;
  color: var(--fg);
  font-size: 14px;
}

.language-meta {
  font-size: 12px;
  color: var(--fg-muted);
  margin-left: auto;
}

.language-bar-container {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.language-bar-bg {
  height: 6px;
  background: var(--border);
  border-radius: 3px;
  overflow: hidden;
}

.language-bar-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s ease;
}

.language-stats {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
}

.language-stats .lines {
  color: var(--fg-muted);
}

.language-stats .percentage {
  font-weight: 600;
  color: var(--fg);
}
</style>
