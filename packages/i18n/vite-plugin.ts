import type { Plugin } from "vite";
import { parse } from "@babel/parser";
import traverseModule from "@babel/traverse";
import MagicString from "magic-string";
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

export function i18nAstPlugin(): Plugin {
  return {
    name: "vite-plugin-i18n-ast",
    enforce: "pre",
    transform(code, id) {
      if (!/\.[tj]sx?$/.test(id)) return null;
      if (
        id.includes("node_modules") ||
        id.includes(".test.") ||
        id.includes(".spec.") ||
        id.includes("packages/i18n") ||
        isIgnoredPath(id)
      ) {
        return null;
      }

      const isTsx = /\.[tj]sx$/.test(id);
      const isActivityDesc = id.includes("describe-action") || id.includes("activity");
      const hasToast = code.includes("toast");

      // Quick early exit for plain .ts files that don't have toasts or action descriptions
      if (!isTsx && !isActivityDesc && !hasToast) {
        return null;
      }

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
      const contextKey = getContextKey(id);
      const ctxArg = contextKey ? `, ${JSON.stringify(contextKey)}` : "";

      traverse(ast, {
        JSXElement(path: any) {
          const tagName = path.node.openingElement.name.name;
          if (IGNORED_TAGS.has(tagName)) {
            path.skip();
            return;
          }
          const unit = parseMixedChildren(path.node.children);
          if (unit && path.node.children.length > 0) {
            const firstChild = path.node.children[0];
            const lastChild = path.node.children[path.node.children.length - 1];
            const escaped = JSON.stringify(unit.template);
            const varObj = `{ ${unit.variables
              .map((v) => `${v.name}: ${code.slice(v.start, v.end)}`)
              .join(", ")} }`;
            s.overwrite(
              firstChild.start,
              lastChild.end,
              `{__t(${escaped}, ${varObj}${ctxArg})}`
            );
            transformed = true;
            path.skip();
            return;
          }
        },
        JSXExpressionContainer(path: any) {
          if (path.node.expression?.type === "TemplateLiteral") {
            const unit = parseSimpleTemplateLiteral(path.node.expression);
            if (unit) {
              const escaped = JSON.stringify(unit.template);
              const varObj = `{ ${unit.variables
                .map((v) => `${v.name}: ${code.slice(v.start, v.end)}`)
                .join(", ")} }`;
              s.overwrite(
                path.node.expression.start,
                path.node.expression.end,
                `__t(${escaped}, ${varObj}${ctxArg})`
              );
              transformed = true;
              path.skip();
            }
          }
        },
        JSXText(path: any) {
          const raw = path.node.value;
          const normalized = normalizeText(raw);
          if (shouldTranslateText(normalized)) {
            const escaped = JSON.stringify(normalized);
            const leadingWs = raw.match(/^\s*/)?.[0] || "";
            const trailingWs = raw.match(/\s*$/)?.[0] || "";
            s.overwrite(path.node.start, path.node.end, `${leadingWs}{__t(${escaped}${ctxArg})}${trailingWs}`);
            transformed = true;
          }
        },
        JSXAttribute(path: any) {
          const attrName = path.node.name?.name;
          if (TRANSLATABLE_ATTRIBUTES.has(attrName) && path.node.value?.type === "StringLiteral") {
            const val = normalizeText(path.node.value.value);
            if (shouldTranslateText(val)) {
              const escaped = JSON.stringify(val);
              s.overwrite(path.node.value.start, path.node.value.end, `{__t(${escaped}${ctxArg})}`);
              transformed = true;
            }
          }
        },
        CallExpression(path: any) {
          const callee = path.node.callee;
          const isToast =
            (callee.type === "MemberExpression" &&
              callee.object?.name === "toast" &&
              ["success", "error", "info", "warning", "loading", "message"].includes(
                callee.property?.name
              )) ||
            (callee.type === "Identifier" && callee.name === "toast");

          if (isToast && path.node.arguments.length > 0) {
            const firstArg = path.node.arguments[0];
            if (firstArg.type === "StringLiteral") {
              const text = normalizeText(firstArg.value);
              if (shouldTranslateText(text)) {
                const escaped = JSON.stringify(text);
                s.overwrite(firstArg.start, firstArg.end, `__t(${escaped}${ctxArg})`);
                transformed = true;
              }
            }
          }
        },
        ReturnStatement(path: any) {
          if (isActivityDesc && path.node.argument?.type === "StringLiteral") {
            const text = normalizeText(path.node.argument.value);
            if (shouldTranslateText(text)) {
              const escaped = JSON.stringify(text);
              s.overwrite(path.node.argument.start, path.node.argument.end, `__t(${escaped}${ctxArg})`);
              transformed = true;
            }
          }
        },
        ArrowFunctionExpression(path: any) {
          if (isActivityDesc && path.node.body?.type === "StringLiteral") {
            const text = normalizeText(path.node.body.value);
            if (shouldTranslateText(text)) {
              const escaped = JSON.stringify(text);
              s.overwrite(path.node.body.start, path.node.body.end, `__t(${escaped}${ctxArg})`);
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
