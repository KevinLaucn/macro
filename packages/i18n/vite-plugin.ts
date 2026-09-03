import type { Plugin } from "vite";
import { parse } from "@babel/parser";
import traverseModule from "@babel/traverse";
import MagicString from "magic-string";
import { TRANSLATABLE_ATTRIBUTES, IGNORED_TAGS, shouldTranslateText } from "./ast-utils";

const traverse = (traverseModule as any).default || traverseModule;

export function i18nAstPlugin(): Plugin {
  return {
    name: "vite-plugin-i18n-ast",
    enforce: "pre",
    transform(code, id) {
      if (!/\.[tj]sx$/.test(id)) return null;
      if (
        id.includes("node_modules") ||
        id.includes(".test.") ||
        id.includes(".spec.") ||
        id.includes("packages/i18n")
      ) {
        return null;
      }

      if (!code.includes("<") || !code.includes(">")) return null;

      let ast: any;
      try {
        ast = parse(code, {
          sourceType: "module",
          plugins: ["jsx", "typescript"],
        });
      } catch {
        return null;
      }

      const s = new MagicString(code);
      let transformed = false;

      traverse(ast, {
        JSXElement(path: any) {
          const tagName = path.node.openingElement.name.name;
          if (IGNORED_TAGS.has(tagName)) {
            path.skip();
          }
        },
        JSXText(path: any) {
          const raw = path.node.value;
          const trimmed = raw.trim();
          if (shouldTranslateText(trimmed)) {
            const escaped = JSON.stringify(trimmed);
            // Preserve leading/trailing whitespace around the translated expression
            const leadingWs = raw.match(/^\s*/)?.[0] || "";
            const trailingWs = raw.match(/\s*$/)?.[0] || "";
            s.overwrite(path.node.start, path.node.end, `${leadingWs}{__t(${escaped})}${trailingWs}`);
            transformed = true;
          }
        },
        JSXAttribute(path: any) {
          const attrName = path.node.name?.name;
          if (TRANSLATABLE_ATTRIBUTES.has(attrName) && path.node.value?.type === "StringLiteral") {
            const val = path.node.value.value;
            if (shouldTranslateText(val)) {
              const escaped = JSON.stringify(val);
              s.overwrite(path.node.value.start, path.node.value.end, `{__t(${escaped})}`);
              transformed = true;
            }
          }
        },
      });

      if (transformed) {
        if (!code.includes("@macro/i18n")) {
          s.prepend(`import { __t } from "@macro/i18n";\n`);
        }
        return {
          code: s.toString(),
          map: s.generateMap({ hires: true }),
        };
      }

      return null;
    },
  };
}

export default i18nAstPlugin;
