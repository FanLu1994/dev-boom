import { ref, watch } from "vue";
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

  watch(theme, applyTheme);

  return { theme, applyTheme, toggleTheme };
}
