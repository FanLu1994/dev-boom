import { ref, watch, onMounted, onUnmounted } from "vue";
import type { ThemeMode } from "../types/project";

const STORAGE_KEY = "pm-theme";

export function useTheme() {
  const theme = ref<ThemeMode>((localStorage.getItem(STORAGE_KEY) as ThemeMode) || "light");

  function applyTheme() {
    document.documentElement.setAttribute("data-theme", theme.value);
    localStorage.setItem(STORAGE_KEY, theme.value);
  }

  function toggleTheme() {
    theme.value = theme.value === "light" ? "dark" : "light";
  }

  function onStorageChange(e: StorageEvent) {
    if (e.key === STORAGE_KEY && e.newValue) {
      theme.value = e.newValue as ThemeMode;
    }
  }

  watch(theme, applyTheme);

  onMounted(() => window.addEventListener("storage", onStorageChange));
  onUnmounted(() => window.removeEventListener("storage", onStorageChange));

  return { theme, applyTheme, toggleTheme };
}
