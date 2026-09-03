export const TRANSLATABLE_ATTRIBUTES = new Set([
  "placeholder",
  "title",
  "aria-label",
  "label",
  "tooltip",
  "emptyText",
  "heading",
  "subheading",
  "description",
  "fallback",
  "allDayText",
  "confirmText",
  "cancelText",
  "buttonText",
  "message",
  "error",
  "success",
  "helperText",
  "loadingText",
  "alt",
]);

export const IGNORED_TAGS = new Set([
  "code",
  "pre",
  "script",
  "style",
  "svg",
  "path",
]);

export const IGNORED_PATH_PATTERNS = [
  /playground/i,
  /debugger/i,
  /\.d\.ts$/,
  /\.stories\.[tj]sx?$/,
  /\.test\.[tj]sx?$/,
  /\.spec\.[tj]sx?$/,
  /node_modules/,
  /test-utils/,
  /fixtures/,
];

export function isIgnoredPath(filePath: string): boolean {
  return IGNORED_PATH_PATTERNS.some((pat) => pat.test(filePath));
}

export function normalizeText(text: string): string {
  return text.trim().replace(/\s+/g, " ");
}

export function shouldTranslateText(text: string): boolean {
  const normalized = normalizeText(text);
  if (!normalized || normalized.length < 2) return false;
  // Must contain at least one ASCII letter
  if (!/[a-zA-Z]/.test(normalized)) return false;
  // Ignore URLs, file paths, CSS selectors, MIME types, technical IDs
  if (normalized.startsWith("http://") || normalized.startsWith("https://")) return false;
  if (normalized.startsWith("/") || normalized.startsWith("./") || normalized.startsWith("../")) return false;
  if (/^[a-z0-9-_]+:[a-z0-9-_]+$/i.test(normalized)) return false;
  if (/^[a-z0-9_-]+\/[a-z0-9_.-]+$/i.test(normalized)) return false; // e.g. application/json
  if (/^(--|\$|\.)[a-z0-9_-]+/i.test(normalized)) return false; // css variables or classes
  return true;
}

export function getContextKey(filePath: string): string | undefined {
  const norm = filePath.replace(/\\/g, "/");
  if (norm.includes("crm")) return "crm";
  if (norm.includes("settings/team") || norm.includes("team-settings")) return "team";
  if (norm.includes("call") || norm.includes("call-panel")) return "call";
  if (norm.includes("activity")) return "activity";
  if (norm.includes("email")) return "email";
  if (norm.includes("chat") || norm.includes("channel")) return "chat";
  return undefined;
}
