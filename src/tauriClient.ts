import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { CommandArgs } from "./appTypes";
import { previewCommand } from "./previewCommands";

export const isTauriRuntime = () =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export async function callCommand<T>(cmd: string, args?: CommandArgs): Promise<T> {
  if (isTauriRuntime()) return invoke<T>(cmd, args);
  return previewCommand<T>(cmd);
}

export async function openExternalUrl(url: string) {
  if (isTauriRuntime()) {
    await openUrl(url);
    return;
  }
  window.open(url, "_blank", "noopener,noreferrer");
}
