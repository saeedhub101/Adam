export const SUPPORTED_CHARACTER_MAIN = /\.(glb|gltf|fbx)$/i;
export const SUPPORTED_CHARACTER_ASSETS = /\.(glb|gltf|fbx|bin|png|jpe?g|webp|ktx2)$/i;
export const MAX_CHARACTER_BYTES = 200 * 1024 * 1024;

export type CharacterImportResult = { ok: true; main: File; files: File[] } | { ok: false; error: string };

export function validateCharacterFiles(input: File[] | FileList): CharacterImportResult {
  const files = Array.from(input);
  const main = files.find(file => SUPPORTED_CHARACTER_MAIN.test(file.name));
  if (!main) return { ok: false, error: "Select a GLB, GLTF, or FBX character model." };
  if (main.size > MAX_CHARACTER_BYTES) return { ok: false, error: "Character file is larger than 200MB." };
  const unsupported = files.filter(file => !SUPPORTED_CHARACTER_ASSETS.test(file.name));
  if (unsupported.length) return { ok: false, error: `Unsupported character asset: ${unsupported[0].name}` };
  return { ok: true, main, files };
}

export function characterName(file: File): string {
  return file.name.replace(/\.(glb|gltf|fbx)$/i, "");
}

export function normalizedBoneName(name: string): string {
  return name.toLowerCase().replace(/mixamorig[:_]?/g, "").replace(/[^a-z0-9]/g, "");
}

export const PHASE_0_2_REQUIREMENTS = Object.freeze({
  desktop: ["window-position", "window-size", "click-through", "multi-display-safe-bounds", "dpi-aware-rendering"],
  character: ["glb", "gltf", "fbx", "persistent-library", "asset-validation", "rig-detection", "animation-detection", "facial-morph-detection"],
  animation: ["idle", "walk", "run", "gesture", "head-look", "eye-look", "blink", "emotion-layer", "gesture-queue", "locomotion"],
  safety: ["permission-modes", "activity-log", "emergency-stop", "allowlisted-computer-control"],
  packaging: ["nsis", "msi", "install-smoke-test", "uninstall-cleanup"]
} as const);
