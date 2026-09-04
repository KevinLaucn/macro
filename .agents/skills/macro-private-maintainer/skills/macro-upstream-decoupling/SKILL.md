---
name: macro-upstream-decoupling
description: Preserve upstream compatibility while slimming KevinLaucn/macro Email production. Use for feature/module/service decoupling, Cargo or frontend dependency pruning, workspace/build-closure reduction, Docker/runtime slimming, Email Profile isolation, decisions about deleting upstream code, and post-upstream-merge re-coupling audits. Retain upstream source by default; exclude unused functionality from UI, dependency/workspace, build, image, and runtime boundaries. Physical source deletion is a last resort.
---

# Macro Upstream-Compatible Decoupling Standard
# (上游兼容解耦与生产瘦身规范)

## 核心法则

> **Preserve upstream source; remove production dependency.**  
> **保留上游源码，解除生产依赖。**
> 
> **删除依赖关系，而不是删除上游源码；缩小生产闭包，而不是追求源码仓库最小化。**

在 `KevinLaucn/macro` 长期跟随官方 `macro-inc/macro` upstream 的架构路线中：
1. **源码树完整性**：尽可能保持 Macro 官方源码树和目录结构完整；
2. **最小化分叉差异**：最大程度降低我们与 upstream 的 fork divergence，保障后续 `git fetch upstream` 与 `git merge upstream/main` / cherry-pick 的低成本平滑合并；
3. **生产闭包隔离**：不需要的功能可以继续存在于 Git 源码仓库，但必须从我们的 **Email Profile** 生产环境中彻底退出；
4. **源码存在 ≠ 生产依赖**：`Repository source existence is NOT production dependency.` 只要一个模块没有进入生产打包和运行时，它的源码存在对生产零影响。

---

## 一、默认强制决策顺序 (Decision Hierarchy)

当用户提出“删除某功能”、“裁剪某模块”、“缩减依赖”、“精简镜像”时，严禁直接 `rm` 源码。必须按以下决策链路自上而下依次判断：

```text
需要裁减某官方功能 / 模块
       │
       ▼
① 能否通过 UI / Navigation / Route 隐藏或隔离？
       │
       ├─ YES → no-route / no-nav / conditional route / profile-specific route registry
       │
       ▼
② 能否通过配置 / Profile 不启动？
       │
       ├─ YES → Compose Profile (full / email) 隔离，容器不启动
       │
       ▼
③ 能否从生产构建图中排除 (Build Closure)？
       │
       ├─ YES → Nix / Cargo / Web build closure 隔离，生产包不构建
       │
       ▼
④ 能否解除代码与工作区依赖 (Dependency Inversion / Feature Gate)？
       │
       ├─ YES → Cargo feature / optional dependency / profile import isolation / workspace union 排除
       │
       ▼
⑤ 能否通过 Adapter / Port 隔离？
       │
       ├─ YES → 抽象 Trait/Port，分化 FullAdapter 与 EmailAdapter
       │
       ▼
⑥ 能否隔离为 Full-only dependency？
       │
       ├─ YES → 归入 upstream/full profile 保留，Email profile 零引用
       │
       ▼
⑦ 能否通过小范围 Thin Patch 解决？
       │
       ├─ YES → 最小化 patch，避免触碰 upstream shared domain
       │
       ▼
⑧ 严格满足两阶段物理删除门禁（Physical Deletion Gate）？
       │
       ├─ YES → 记录 divergence 审计日志与恢复策略后方可执行物理删除
       └─ NO  → 驳回物理删除，退回上方阶段实施解耦
```

---

## 二、完整生产边界矩阵 (Production Boundary Matrix)

严禁以“页面看不到”或“没有启动”作为功能裁干净的标准。必须逐层核验完整生产边界：

```text
Source (源码保留) ────────► 允许存在于 Git 仓库，upstream 可平滑更新
   ↓
UI / Route       ────────► Email Profile 必须切断路由与界面入口
   ↓
Dependency / Import ─────► Cargo 依赖图 / 前端 Import Graph 必须切断
   ↓
Workspace / Resolution ──► 根 Cargo / Nix / Bun 工作区解析必须切断，不参与 Email 预构建
   ↓
Build Closure    ────────► Nix crane / Web bundle 编译闭包必须排除
   ↓
Image            ────────► 生产 Docker 镜像中严禁打包相关二进制与静态产物
   ↓
Runtime          ────────► 运行时容器、后台进程、网络请求、轮询必须为 0
```

### 理想状态指标
- 源码存在：`YES`（保持 upstream 干净可合并）
- Email UI 引用：`EXCLUDED`
- Email 依赖图（Cargo / NPM）：`EXCLUDED`
- Email Workspace / Package Resolution：`EXCLUDED`（不触发无谓解析与构建）
- Email Build 闭包：`EXCLUDED`
- Email Docker 镜像：`EXCLUDED`
- Email Runtime 容器与进程：`EXCLUDED`

---

## 三、前端 (SolidJS / Web) 解耦规范

### 1. 明确区别运行时隔离与构建排除
> **Lazy loading is not equivalent to build exclusion.**  
> 动态懒加载（如 `lazy(() => import("./Calendar"))`）只能保证运行时首屏不立即加载，但该模块的 Chunk 依然会被打包进生产 `dist/`，依然进入静态镜像构建产物。

### 2. 彻底排除手段
若目标是将某功能彻底从 Email 生产构建闭包中剔除，优先采用：
- **Profile-specific route registry**：按构建 target 或 profile 注册路由表；
- **Compile-time condition / env**：编译时条件分支，触发摇树优化（Tree-shaking）；
- **Email-specific entrypoint**：独立的前端入口或构建目标；
- **Virtual / Replacement module**：在 Email 构建时通过 build-time alias 替换为空桩。

### 3. 前端阻断检查清单
对于非 Email 核心功能（Calendar、Calls、Tasks、Agents、Docs、Canvas 等）：
- 不注册到 Email Profile 的 router 表中；
- 不出现在侧边栏、顶部导航和快捷跳转中；
- 不进入主页面初始 render 逻辑；
- 不注册全局快捷键监听；
- 不挂载无用功能的 Context Provider；
- 不产生定时拉取、后台轮询请求；
- 不订阅该功能的 WebSocket realtime channels；
- 不产生该功能的埋点与外部上报。

---

## 四、后端 (Rust / Cargo) 解耦规范

### 1. 核心原则
> **如果 Email 不需要 Calendar，不要去修改 Calendar 内部实现；应该让 Email dependency graph 根本不知道 Calendar 存在。**

### 2. 依赖控制机制
当共享服务（如 `document_storage_service`、`search_processing_service`）同时服务多种业务时，通过 Cargo features 细粒度控制闭包：
```toml
[features]
default = ["full"]

email = [
    "email",
    "attachments",
]

full = [
    "email",
    "attachments",
    "calendar",
    "calls",
    "channels",
    "ai",
]

[dependencies]
calendar_service_client = { path = "../calendar", optional = true }
call_service_client     = { path = "../calls", optional = true }
ai_tools                = { path = "../ai_tools", optional = true }
```
Email 编译模式：
```bash
cargo build -p <service> --no-default-features --features email
```
*注：严禁机械化一刀切重构，必须先使用 CodeGraph 检查真实 dependency graph，选择侵入最小的方案。*

### 3. Hexagonal 架构与 Adapter 隔离
若官方某服务同时承担 Full 功能和 Email 功能，严禁直接 fork 魔改。优先遵循 Hexagonal 架构：
```text
Domain
  │
  ▼
Port / Trait
  │
 ┌┴───────────────────────┐
 ▼                        ▼
FullStorageAdapter       EmailStorageAdapter (如 Gmail Lazy Fetch)
```

---

## 五、Full Profile 保留原则 (Full Profile Preservation)

> **Decouple Email without unnecessarily breaking Full.**  
> 裁减 Email Profile 绝不能以破坏 Macro Full Profile 为代价！

保持以下架构拓扑：
```text
                 Macro Source
                     │
          ┌──────────┴──────────┐
          ▼                     ▼
        Full                  Email
   upstream-compatible      lightweight
```
- 源码共存，Full Profile 保持开箱可用，供开发调试与全功能对照；
- Email Profile 拥有独立、可审计、最小化的生产依赖闭包。

---

## 六、重耦合守卫 (Re-coupling Guard)

本规范不仅用于初始裁剪，更用于**防止后续 upstream merge 或依赖升级时，已被剔除的功能被静默重新引入**。

### 1. 触发审计场景
当发生以下任一变更时，必须自动激活 Re-coupling Guard 审查：
- `git merge upstream/main` / upstream upgrade
- `Cargo.lock` / `bun.lock` 更新
- 根或 crate 的 `Cargo.toml` 变更
- `package.json` 变更
- `nix/cloud-storage.nix` 或 `.github/workspace-dep-closures.json` 变更
- 前端全局 Router、Navigation、Shared Provider 变更
- `self-host/docker-compose.yml` 变更

### 2. 防守检查点
重点检查已排除的业务（如 Calendar、Calls、Tasks、Agents 等）是否通过：
- 新增的 shared crate 依赖；
- 新增的顶层 import；
- 新增的全局 provider 注入；
重新渗入到 Email Profile 的生产镜像或构建图中。

---

## 七、两阶段物理删除门禁 (Two-Stage Physical Deletion Gate)

> **User request alone does not bypass dependency analysis.**  
> 严禁因为用户一句“帮我删掉”就直接在仓库中物理删除 upstream 源码！

物理删除必须严格通过以下两阶段门禁（Gate A 与 Gate B）：

### Gate A：全部条件必须满足
1. **完成依赖链分析**：已使用 CodeGraph 确认无任何活跃上游调用；
2. **影响面排查**：已核查 callers / callees / impact；
3. **方案穷尽**：已客观评估 UI 隐藏、Compose Profile、Cargo Feature、Adapter 隔离等解耦方案；
4. **技术阻碍确认**：已确认保留源码确实对 Email 构建或运行造成了无法绕过的实质技术冲突；
5. **分叉记录落盘**：已记录完整的 `UPSTREAM-DIVERGENCE` 说明；
6. **提供恢复策略**：明确说明未来 upstream 更新该模块时如何还原或合并。

### Gate B：至少满足以下一项条件
- **上游已删除**：`macro-inc/macro` 官方主干已物理移除该模块；
- **无法隔离的安全风险**：包含严重安全隐患且无法通过 disable/profile 隔离；
- **官方云专有且完全替代**：Macro Cloud 闭源专有逻辑已由自托管本地 adapter 100% 替代；
- **工具链致命冲突**：现代编译工具链彻底报错且 Cargo/Nix/TS 无法隔离；
- **用户在充分知悉 upstream merge 成本后仍然坚持删除**。

---

## 八、CodeGraph 有条件强制原则 (Evidence Before Deletion)

严禁对一行简单的配置变更机械跑全量图谱扫描；但在涉及以下**高风险结构改动**时，必须强制执行 CodeGraph 语法级关联分析：
- 移除或停用某微服务（Service removal）
- 清理 Cargo crate 依赖或重构 feature flags
- 变更公共共享模块（Shared module change）
- 前端依赖或公共 Provider 清理
- 路由表大规模重构
- 物理删除任何 upstream 文件

**优先使用命令**：
- `codegraph callers <symbol>`：审查是否有核心 Email 业务还在调用该符号；
- `codegraph callees <symbol>`：核查该模块下游带入了哪些沉重依赖；
- `codegraph impact <symbol>`：精确评估解耦后的改动波及面；
- `codegraph node <filepath>`：查看文件的依赖导入关系。

---

## 九、按波及层级等比验证 (Proportional Validation)

验证范围严格与修改所触碰的层级等比挂钩（Validation must be proportional to touched layers），避免盲目全量跑测试：

| 修改范围 / 涉及层级 | 强制验证动作 |
|---|---|
| **前端 Route / Import 修改** | `bun check` + 生产 Web build 构建检查 |
| **Cargo Feature / 后端依赖** | `cargo check -p <crate>` + 涉及 crate 测试 + Email Nix/build closure 检查 |
| **Compose / Profile 调整** | `docker compose config` + 检查生成的 service 列表 |
| **Dockerfile / 镜像构建** | 本地或 CI 镜像构建 + 检查产物包含的二进制与体积 |
| **运行时服务逻辑** | 容器健康检查 + 日志检查 + 浏览器 Network 零异常请求核验 |
| **修改 upstream shared code** | 必须同时验证 Email 路径 与 受影响的 Full 路径 |

---

## 十、标准解耦报告规范 (Decoupling Report)

完成任何裁剪、解耦、瘦身任务后，必须严格按照以下标准化格式汇报：

```text
Feature: <裁剪功能名称，例如 Calendar>

Decision: KEEP SOURCE / DECOUPLE / DELETE

Method: <解耦手段简述，例如 Route gate + Cargo feature + Email build closure exclusion>

Boundary:
  Source: RETAINED / REMOVED
  UI: EXCLUDED / INCLUDED
  Dependency: EXCLUDED / INCLUDED
  Workspace: EXCLUDED FROM EMAIL PROFILE / INCLUDED
  Build: EXCLUDED / INCLUDED
  Image: EXCLUDED / INCLUDED
  Runtime: EXCLUDED / INCLUDED

Upstream Merge Risk: LOW / MEDIUM / HIGH

Physical deletion: NONE / <列出物理删除的文件清单>

Validation:
  <命令 1>: PASS / FAIL
  <命令 2>: PASS / FAIL
```
