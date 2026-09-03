import fs from "node:fs";
import path from "node:path";
import { parse } from "@babel/parser";
import traverseModule from "@babel/traverse";
import { TRANSLATABLE_ATTRIBUTES, IGNORED_TAGS, shouldTranslateText } from "./ast-utils";

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
      if (file !== "node_modules" && file !== "i18n") {
        getAllFiles(full, ext, list);
      }
    } else if (ext.test(file) && !file.includes(".test.") && !file.includes(".spec.")) {
      list.push(full);
    }
  }
  return list;
}

async function run() {
  console.log("🔍 Scanning apps/web/src for UI literals...");
  const files = getAllFiles(webSrcDir, /\.[tj]sx$/);
  console.log(`Found ${files.length} UI source files.`);

  const currentStrings = new Map<string, string[]>(); // key -> occurrences

  for (const file of files) {
    const code = fs.readFileSync(file, "utf-8");
    if (!code.includes("<") || !code.includes(">")) continue;
    try {
      const ast = parse(code, {
        sourceType: "module",
        plugins: ["jsx", "typescript"],
      });
      traverse(ast, {
        JSXElement(p: any) {
          if (IGNORED_TAGS.has(p.node.openingElement.name.name)) {
            p.skip();
          }
        },
        JSXText(p: any) {
          const text = p.node.value.trim();
          if (shouldTranslateText(text)) {
            const rel = path.relative(webSrcDir, file);
            currentStrings.set(text, [...(currentStrings.get(text) || []), rel]);
          }
        },
        JSXAttribute(p: any) {
          const attr = p.node.name?.name;
          if (TRANSLATABLE_ATTRIBUTES.has(attr) && p.node.value?.type === "StringLiteral") {
            const val = p.node.value.value;
            if (shouldTranslateText(val)) {
              const rel = path.relative(webSrcDir, file);
              currentStrings.set(val, [...(currentStrings.get(val) || []), rel]);
            }
          }
        },
      });
    } catch {
      // skip parse errors
    }
  }

  const existingZh: Record<string, string> = fs.existsSync(zhDictPath)
    ? JSON.parse(fs.readFileSync(zhDictPath, "utf-8"))
    : {};

  const missing: Record<string, string> = {};
  const obsolete: Record<string, string> = {};
  const ambiguous: Record<string, string[]> = {};

  for (const [key, occurrences] of currentStrings.entries()) {
    if (!existingZh[key]) {
      missing[key] = "";
    }
    if (occurrences.length > 3) {
      ambiguous[key] = Array.from(new Set(occurrences));
    }
  }

  for (const key of Object.keys(existingZh)) {
    if (!currentStrings.has(key)) {
      obsolete[key] = existingZh[key];
    }
  }

  fs.writeFileSync(path.join(diffDir, "missing.json"), JSON.stringify(missing, null, 2), "utf-8");
  fs.writeFileSync(path.join(diffDir, "obsolete.json"), JSON.stringify(obsolete, null, 2), "utf-8");
  fs.writeFileSync(path.join(diffDir, "ambiguous.json"), JSON.stringify(ambiguous, null, 2), "utf-8");

  console.log("==========================================");
  console.log(`✅ AST Scan Completed:`);
  console.log(`  - Total UI Literals Found: ${currentStrings.size}`);
  console.log(`  - Missing Translations:    ${Object.keys(missing).length}`);
  console.log(`  - Obsolete Translations:   ${Object.keys(obsolete).length}`);
  console.log(`  - Ambiguous / Multiverse:  ${Object.keys(ambiguous).length}`);
  console.log(`Diff reports saved in packages/i18n/diff/`);
  console.log("==========================================");
}

run();
