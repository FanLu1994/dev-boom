import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();

export function useWindowControls() {
  async function minimizeWindow() {
    try {
      await appWindow.minimize();
    } catch {
      // noop when not in tauri runtime
    }
  }

  async function toggleMaximizeWindow() {
    try {
      await appWindow.toggleMaximize();
    } catch {
      // noop when not in tauri runtime
    }
  }

  async function closeWindow() {
    try {
      await appWindow.close();
    } catch {
      // noop when not in tauri runtime
    }
  }

  async function startWindowDrag(event: MouseEvent) {
    const target = event.target as HTMLElement | null;
    if (target?.closest(".title-actions")) return;
    try {
      await appWindow.startDragging();
    } catch {
      // noop when not in tauri runtime
    }
  }

  return { minimizeWindow, toggleMaximizeWindow, closeWindow, startWindowDrag };
}
