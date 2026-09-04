# i18n 国际化与渐进式二开规范指南

本文档记录 `KevinLaucn/macro` 自研与魔改二开模块的 i18n 国际化标准工作流。

---

## 核心架构原则

1. **彻底摆脱构建期 AST 自动注入**：
   - 历史架构通过 Babel 插件在构建阶段对所有 JSX 文本自动插入 `__t()`，导致源码不可见、二开容易产生构建副作用、混淆动态模板与静态字面量。
   - 新架构全面采用显式 `t()` 运行时：`import { t } from '@macro/i18n';`。
2. **保留现有轻量运行时与翻译资产**：
   - 轻量运行时位于 `packages/i18n/runtime.ts`，支持响应式切换、对象选项（`{ context, fallback }`）、变量插值以及常用日期/数字格式化。
   - 翻译资产集中于 `packages/i18n/locales/zh-CN.json`，以英文原文作为 fallback key，无需额外拆分为零散的语义 key 文件。
3. **渐进式排除与物理退役（Exclude & Sunset）**：
   - 在 `apps/web/vite.base.ts` 中通过 `i18nAstPlugin({ excludePatterns: [...] })` 将已改造或自研的二开模块显式排除在 AST 转换之外。
   - 随着二开范围覆盖所有业务模块，最终直接移除 Babel AST 插件与转换逻辑。

---

## 二开组件多语言开发标准

### 1. 显式引入与调用
在任何二开或重构的 SolidJS 组件中：
```tsx
import { t } from '@macro/i18n';

// 基础文本
<Button>{t('Save Draft')}</Button>

// 属性文本
<Input placeholder={t('Search contacts...')} />

// 带上下文消歧
<span>{t('Email', { context: 'nav' })}</span>

// 动态插值
<span>{t('Page {current} of {total}', { current: 1, total: 10 })}</span>
```

### 2. 改造后加入排除清单
每次完成某个二开文件或目录的显式化改造后，打开 `apps/web/vite.base.ts`，将其路径追加到 `excludePatterns`：
```ts
i18nAstPlugin({
  excludePatterns: [
    '/features/settings/Settings.tsx',
    '/features/settings/Appearance.tsx',
    '/features/settings/Shortcuts.tsx',
    '/features/settings/Crm.tsx',
    '/features/block-email/component/compose/ComposeToolbar.tsx',
    // 在此追加新模块路径...
  ],
})
```

---

## 工具链使用指南

### 1. 语法树审计检测 (`audit.ts`)
用于只读扫描代码中遗漏未包裹 `t()` 的 JSX 文本或属性：
```bash
# 全局扫描
bun run packages/i18n/audit.ts

# 定向扫描某个二开模块（如联系人）
bun run packages/i18n/audit.ts apps/web/src/features/contacts

# 单文件检查
bun run packages/i18n/audit.ts apps/web/src/features/settings/Settings.tsx
```
- 扫描结果自动汇总于终端，并输出 Top 10 未翻译密集文件；
- 详细行号和片段保存在 `packages/i18n/diff/audit-untranslated.json`。

### 2. 词条精准提取 (`extract.ts`)
当添加了新的 `t('...')` 后，运行提取脚本：
```bash
bun run packages/i18n/extract.ts
```
- 自动提取全局所有显式 `t()` 词条；
- 检测是否在 `zh-CN.json` 中缺失；若有缺失，会在 `packages/i18n/diff/missing.json` 中列出，并在终端提示数量。补齐中文翻译后重新执行验证直到 `Missing in zh-CN: 0`。

### 3. 单元测试校验
```bash
bun test packages/i18n
```

---

## Git 增量审计防回退规范 (Pre-commit Audit Hook)

为防止团队在已排除在 AST 之外的文件中无意提交未包裹 `t()` 的裸英文字符串，可在开发环境或 CI 中运行增量审计：

```bash
# 检查当前 git 暂存区中已排除文件的裸英文字符串
for file in $(git diff --cached --name-only --diff-filter=ACM | grep -E '\.tsx$'); do
  bun run packages/i18n/audit.ts "$file"
done
```
若发现未包裹的裸字符串且该文件位于 `excludePatterns`，必须将其用 `t()` 包裹并补齐翻译后再行提交。
