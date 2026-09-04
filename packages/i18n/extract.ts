import fs from "node:fs";
import path from "node:path";
import { parse } from "@babel/parser";
import traverseModule from "@babel/traverse";

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
      if (file !== "node_modules" && file !== "dist" && file !== ".vite") {
        getAllFiles(full, ext, list);
      }
    } else if (ext.test(file) && !file.endsWith(".d.ts") && !file.endsWith(".test.ts") && !file.endsWith(".test.tsx")) {
      list.push(full);
    }
  }
  return list;
}

function normalizeKey(str: string): string {
  return str.trim().replace(/\s+/g, " ");
}

async function run() {
  console.log("🔍 Scanning apps/web/src for explicit t() and __t() calls...");
  const files = getAllFiles(webSrcDir, /\.[tj]sx?$/);

  const currentCalls = new Map<string, { occurrences: string[]; context?: string }>();
  const parseFailures: { file: string; error: string }[] = [];

  for (const file of files) {
    const rel = path.relative(webSrcDir, file);
    const code = fs.readFileSync(file, "utf-8");

    // Quick filter: check if file contains 't(' or '__t('
    if (!code.includes("t(") && !code.includes("__t(")) {
      continue;
    }

    try {
      const ast = parse(code, {
        sourceType: "module",
        plugins: ["jsx", "typescript"],
      });

      traverse(ast, {
        CallExpression(p: any) {
          const callee = p.node.callee;
          const isT =
            (callee.type === "Identifier" && (callee.name === "t" || callee.name === "__t")) ||
            (callee.type === "MemberExpression" &&
              callee.property?.type === "Identifier" &&
              (callee.property.name === "t" || callee.property.name === "__t"));

          if (isT && p.node.arguments.length > 0) {
            const firstArg = p.node.arguments[0];
            let rawKey: string | undefined;

            if (firstArg.type === "StringLiteral") {
              rawKey = firstArg.value;
            } else if (firstArg.type === "TemplateLiteral" && firstArg.quasis.length === 1) {
              rawKey = firstArg.quasis[0].value.raw;
            }

            if (rawKey) {
              const key = normalizeKey(rawKey);
              let context: string | undefined;

              // Extract context if present in 2nd or 3rd argument
              if (p.node.arguments.length >= 2) {
                const secondArg = p.node.arguments[1];
                if (secondArg.type === "StringLiteral") {
                  context = secondArg.value;
                } else if (secondArg.type === "ObjectExpression") {
                  for (const prop of secondArg.properties) {
                    if (
                      prop.type === "ObjectProperty" &&
                      prop.key?.name === "context" &&
                      prop.value?.type === "StringLiteral"
                    ) {
                      context = prop.value.value;
                    }
                  }
                }
              }

              const dictKey = context ? `${key}@@${context}` : key;
              const existing = currentCalls.get(dictKey) || { occurrences: [] };
              existing.occurrences.push(rel);
              if (context) existing.context = context;
              currentCalls.set(dictKey, existing);
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
  const inUse: Record<string, string[]> = {};
  const ambiguous: Record<string, string[]> = {};
  const obsolete: Record<string, string> = {};

  for (const [key, meta] of currentCalls.entries()) {
    const uniqueOccurrences = Array.from(new Set(meta.occurrences));
    inUse[key] = uniqueOccurrences;
    if (!existingZh[key]) {
      // Check if base key exists when context key is searched
      const baseKey = key.includes("@@") ? key.split("@@")[0] : key;
      if (!existingZh[baseKey]) {
        missing[key] = "";
      }
    }
    if (uniqueOccurrences.length > 2) {
      ambiguous[key] = uniqueOccurrences;
    }
  }

  for (const key of Object.keys(existingZh)) {
    if (!currentCalls.has(key)) {
      const baseKey = key.includes("@@") ? key.split("@@")[0] : key;
      if (!currentCalls.has(baseKey)) {
        obsolete[key] = existingZh[key];
      }
    }
  }

  if (!fs.existsSync(diffDir)) {
    fs.mkdirSync(diffDir, { recursive: true });
  }

  fs.writeFileSync(path.join(diffDir, "missing.json"), JSON.stringify(missing, null, 2), "utf-8");
  fs.writeFileSync(path.join(diffDir, "in-use.json"), JSON.stringify(inUse, null, 2), "utf-8");
  fs.writeFileSync(path.join(diffDir, "parse-failures.json"), JSON.stringify(parseFailures, null, 2), "utf-8");
  fs.writeFileSync(path.join(diffDir, "obsolete.json"), JSON.stringify(obsolete, null, 2), "utf-8");
  fs.writeFileSync(path.join(diffDir, "ambiguous.json"), JSON.stringify(ambiguous, null, 2), "utf-8");

  console.log("==========================================");
  console.log(`✅ Explicit t() Extractor Completed:`);
  console.log(`  - Explicit t() Keys Found: ${currentCalls.size}`);
  console.log(`  - Missing in zh-CN:       ${Object.keys(missing).length}`);
  console.log(`  - Parse Failures:          ${parseFailures.length}`);
  console.log(`Reports saved in packages/i18n/diff/`);
  console.log("==========================================");
}

run();
