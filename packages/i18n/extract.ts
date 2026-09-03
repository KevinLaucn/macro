import fs from "node:fs";
import path from "node:path";
import { parse } from "@babel/parser";
import traverseModule from "@babel/traverse";
import {
  TRANSLATABLE_ATTRIBUTES,
  IGNORED_TAGS,
  isIgnoredPath,
  normalizeText,
  shouldTranslateText,
  getContextKey,
  parseMixedChildren,
  parseSimpleTemplateLiteral,
} from "./ast-utils";

const traverse = (traverseModule as any).default || traverseModule;

const webSrcDir = path.resolve(__dirname, "../../apps/web/src");
const zhDictPath = path.resolve(__dirname, "./locales/zh-CN.json");
const diffDir = path.resolve(__dirname, "./diff");

function getAllFiles(dir: string, ext: RegExp, list: string[] = []): string[] {
  const files = fs.readdirSync(dir);
  for (const file of files) {
    const full = path.join(dir, file);
    const stat = fs.statSync(full);
    if (stat.isDirectory()) {
      if (file !== "node_modules" && file !== "i18n" && file !== "dist" && file !== ".vite") {
        getAllFiles(full, ext, list);
      }
    } else if (ext.test(file) && !isIgnoredPath(full)) {
      list.push(full);
    }
  }
  return list;
}

async function run() {
  console.log("🔍 Scanning apps/web/src for UI literals across .ts and .tsx...");
  const files = getAllFiles(webSrcDir, /\.[tj]sx?$/);
  console.log(`Found ${files.length} production UI source files.`);

  const currentStrings = new Map<string, string[]>(); // normalizedKey -> occurrences
  const parseFailures: { file: string; error: string }[] = [];

  for (const file of files) {
    const rel = path.relative(webSrcDir, file);
    const code = fs.readFileSync(file, "utf-8");
    const isTsx = file.endsWith(".tsx") || file.endsWith(".jsx");
    const isActivityDesc = file.includes("describe-action") || file.includes("activity");

    try {
      const ast = parse(code, {
        sourceType: "module",
        plugins: ["jsx", "typescript"],
      });

      traverse(ast, {
        JSXElement(p: any) {
          if (IGNORED_TAGS.has(p.node.openingElement.name.name)) {
            p.skip();
            return;
          }
          const unit = parseMixedChildren(p.node.children);
          if (unit) {
            currentStrings.set(unit.template, [...(currentStrings.get(unit.template) || []), rel]);
            p.skip();
            return;
          }
        },
        TemplateLiteral(p: any) {
          const unit = parseSimpleTemplateLiteral(p.node);
          if (unit) {
            currentStrings.set(unit.template, [...(currentStrings.get(unit.template) || []), rel]);
          }
        },
        JSXText(p: any) {
          const raw = p.node.value;
          const normalized = normalizeText(raw);
          if (shouldTranslateText(normalized)) {
            currentStrings.set(normalized, [...(currentStrings.get(normalized) || []), rel]);
          }
        },
        JSXAttribute(p: any) {
          const attr = p.node.name?.name;
          if (TRANSLATABLE_ATTRIBUTES.has(attr)) {
            if (p.node.value?.type === "StringLiteral") {
              const val = normalizeText(p.node.value.value);
              if (shouldTranslateText(val)) {
                currentStrings.set(val, [...(currentStrings.get(val) || []), rel]);
              }
            }
          }
        },
        CallExpression(p: any) {
          // Detect toast.success("..."), toast.error("..."), toast.info("..."), toast("...")
          const callee = p.node.callee;
          const isToast =
            (callee.type === "MemberExpression" &&
              callee.object?.name === "toast" &&
              ["success", "error", "info", "warning", "loading", "message"].includes(
                callee.property?.name
              )) ||
            (callee.type === "Identifier" && callee.name === "toast");

          if (isToast && p.node.arguments.length > 0) {
            const firstArg = p.node.arguments[0];
            if (firstArg.type === "StringLiteral") {
              const text = normalizeText(firstArg.value);
              if (shouldTranslateText(text)) {
                currentStrings.set(text, [...(currentStrings.get(text) || []), rel]);
              }
            }
          }
        },
        ReturnStatement(p: any) {
          // Detect user-facing action description returns (e.g. describe-action.ts)
          if (isActivityDesc && p.node.argument?.type === "StringLiteral") {
            const text = normalizeText(p.node.argument.value);
            if (shouldTranslateText(text)) {
              currentStrings.set(text, [...(currentStrings.get(text) || []), rel]);
            }
          }
        },
        ArrowFunctionExpression(p: any) {
          if (isActivityDesc && p.node.body?.type === "StringLiteral") {
            const text = normalizeText(p.node.body.value);
            if (shouldTranslateText(text)) {
              currentStrings.set(text, [...(currentStrings.get(text) || []), rel]);
            }
          }
        },
      });
    } catch (err: any) {
      parseFailures.push({ file: rel, error: err?.message || String(err) });
    }
  }

  const existingZh: Record<string, string> = fs.existsSync(zhDictPath)
    ? JSON.parse(fs.readFileSync(zhDictPath, "utf-8"))
    : {};

  const missing: Record<string, string> = {};
  const obsolete: Record<string, string> = {};
  const ambiguous: Record<string, string[]> = {};
  const changed: Record<string, { occurrences: number; sampleFiles: string[] }> = {};

  for (const [key, occurrences] of currentStrings.entries()) {
    if (!existingZh[key]) {
      missing[key] = "";
    }
    const uniqueOccurrences = Array.from(new Set(occurrences));
    if (uniqueOccurrences.length > 2) {
      ambiguous[key] = uniqueOccurrences;
    }
  }

  for (const key of Object.keys(existingZh)) {
    if (!currentStrings.has(key)) {
      obsolete[key] = existingZh[key];
    }
  }

  if (!fs.existsSync(diffDir)) {
    fs.mkdirSync(diffDir, { recursive: true });
  }

  fs.writeFileSync(path.join(diffDir, "missing.json"), JSON.stringify(missing, null, 2), "utf-8");
  fs.writeFileSync(path.join(diffDir, "obsolete.json"), JSON.stringify(obsolete, null, 2), "utf-8");
  fs.writeFileSync(path.join(diffDir, "ambiguous.json"), JSON.stringify(ambiguous, null, 2), "utf-8");
  fs.writeFileSync(path.join(diffDir, "changed.json"), JSON.stringify(changed, null, 2), "utf-8");
  fs.writeFileSync(
    path.join(diffDir, "parse-failures.json"),
    JSON.stringify(parseFailures, null, 2),
    "utf-8"
  );

  console.log("==========================================");
  console.log(`✅ AST Scan Completed:`);
  console.log(`  - Total UI Literals Found: ${currentStrings.size}`);
  console.log(`  - Missing Translations:    ${Object.keys(missing).length}`);
  console.log(`  - Obsolete Translations:   ${Object.keys(obsolete).length}`);
  console.log(`  - Ambiguous / Multiverse:  ${Object.keys(ambiguous).length}`);
  console.log(`  - Parse Failures:          ${parseFailures.length}`);
  if (parseFailures.length > 0) {
    console.warn(`  ⚠️  Warning: ${parseFailures.length} files had parse errors (logged in parse-failures.json)`);
  }
  console.log(`Diff reports saved in packages/i18n/diff/`);
  console.log("==========================================");
}

run();
