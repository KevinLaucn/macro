export const TRANSLATABLE_ATTRIBUTES = new Set([
  "placeholder",
  "title",
  "aria-label",
  "label",
  "tooltip",
  "emptyText",
  "heading",
  "description",
  "fallback",
]);

export const IGNORED_TAGS = new Set([
  "code",
  "pre",
  "script",
  "style",
  "svg",
  "path",
]);

export function shouldTranslateText(text: string): boolean {
  const trimmed = text.trim();
  if (!trimmed || trimmed.length < 2) return false;
  // Must contain at least one ASCII letter
  if (!/[a-zA-Z]/.test(trimmed)) return false;
  // Ignore URLs, file paths, css classes, html tags
  if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) return false;
  if (trimmed.startsWith("/") || trimmed.startsWith("./") || trimmed.startsWith("../")) return false;
  if (/^[a-z0-9-_]+:[a-z0-9-_]+$/i.test(trimmed)) return false;
  return true;
}
