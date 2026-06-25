import { getCurrentWindow } from "@tauri-apps/api/window";

export type WindowSurface = "dock" | "launcher" | "preview";

export function surfaceFromWindowLabel(label: string | undefined): WindowSurface {
  if (label === "dock" || label === "launcher") {
    return label;
  }

  return "preview";
}

export function getWindowSurface(): WindowSurface {
  if (typeof window === "undefined") {
    return "preview";
  }

  const querySurface = new URLSearchParams(window.location.search).get("surface");
  const surface = surfaceFromWindowLabel(querySurface ?? undefined);
  if (surface !== "preview") {
    return surface;
  }

  if ("__TAURI_INTERNALS__" in window) {
    return surfaceFromWindowLabel(getCurrentWindow().label);
  }

  return "preview";
}
