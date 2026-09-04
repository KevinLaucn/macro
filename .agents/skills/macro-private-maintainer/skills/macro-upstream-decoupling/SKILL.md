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

### 技术定义：Profile-driven Reachability Isolation

Macro 二开解耦采用 **Profile-driven Reachability Isolation**：

> 在指定 Product Profile 下，通过切断 feature、module/import、composition root、router/worker registry、build inventory 与 runtime composition 的 activation edges，使目标 Domain 不进入实际编译闭包、生产 artifact、Docker image 与运行时图，同时尽量保留 upstream Domain implementation 原样，以最小化 Fork maintenance surface。

中文简单版：

> **源码可以留在仓库，但我们的产品不编译它、不打包它、不放进镜像、不启动它。**

更形式化地说，Macro 可以视为一张有向图：

```text
G = (V, E)

V = module / crate / service / route / worker / frontend feature / artifact
E = import / dependency / feature activation / DI / route registration / worker registration / build inventory / runtime call
```

解耦不是删除目标 Domain 的全部节点，而是找到尽量小的 **Cut Set**，切断 Product Profile 到目标 Domain 的可达路径：

```text
Reachable(<product_profile>, <target_domain>) = false
```

目标功能源码可以继续存在于仓库中，但在指定 Product Profile 下，不进入编译闭包、不进入生产二进制、不进入 Docker 镜像、不注册运行时服务/路由，也不产生相关运行时依赖。

在 `KevinLaucn/macro` 长期跟随官方 `macro-inc/macro` upstream 的架构路线中：
1. **源码树完整性**：尽可能保持 Macro 官方源码树和目录结构完整；
2. **最小化分叉差异**：最大程度降低我们与 upstream 的 fork divergence，保障后续 `git fetch upstream` 与 `git merge upstream/main` / cherry-pick 的低成本平滑合并；
3. **生产闭包隔离**：不需要的功能可以继续存在于 Git 源码仓库，但必须从我们的 **Email Profile** 生产环境中彻底退出；
4. **源码存在 ≠ 生产依赖**：`Repository source existence is NOT production dependency.` 只要一个模块没有进入生产打包和运行时，它的源码存在对生产零影响。
5. **Product Profile 是根节点**：任何解耦判断都必须从 `full`、`self-host-email` 等 Product Profile 出发，验证目标 Domain 对该 Profile 的可达性，而不是只看源码是否存在。

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

验收链必须按 Product Profile 向下穿透：

```text
Product Profile
      ↓
Feature / Activation
      ↓
Compile Graph
      ↓
Binary / Web Artifact
      ↓
Docker Image
      ↓
Runtime
```

目标 Domain 必须在后四层都不可达：

```text
Source Repository
Calendar                EXISTS

Self-host Email:
Compile Reachability     FALSE
Binary Reachability      FALSE
Image Reachability       FALSE
Runtime Reachability     FALSE
```

### 五层可达性标准

```text
1. Activation Reachability
   Product Profile 是否激活目标功能。

2. Compile Reachability
   Cargo / Web build graph 是否包含目标功能。

3. Artifact Reachability
   Binary / frontend bundle / Nix closure 是否包含目标功能。

4. Image Reachability
   Docker image 是否包含目标 binary / assets / dependencies。

5. Runtime Reachability
   Compose / Route / Worker / Queue 是否启动或调用目标功能。
```

另有维护性指标：

```text
6. Fork Maintenance Surface
   为实现解耦修改了多少 upstream 原文件。
```

### 理想状态指标
- 源码存在：`YES`（保持 upstream 干净可合并）
- Email UI 引用：`EXCLUDED`
- Email 依赖图（Cargo / NPM）：`EXCLUDED`
- Email Workspace / Package Resolution：`EXCLUDED`（不触发无谓解析与构建）
- Email Build 闭包：`EXCLUDED`
- Email Docker 镜像：`EXCLUDED`
- Email Runtime 容器与进程：`EXCLUDED`

### Image 层必须单独验收

Cargo 不再编译目标 Domain 不代表生产制品已经干净。必须单独检查 Dockerfile、Nix closure、镜像构建 inventory 与静态资源拷贝规则，避免出现：

```text
Cargo 已经不编译 Calendar
Dockerfile / Nix closure 仍然 COPY calendar config
Dockerfile / Nix closure 仍然 COPY 不需要的 binary
Dockerfile / Nix closure 仍然 COPY 整个 workspace
Dockerfile / Nix closure 仍然安装 Calendar 专属资源
Frontend image 仍然包含不需要的 JS bundle
```

如果 `calendar_service` binary 没有在 Compose 中启动，但仍然被打进 `macro-services-email:latest`，也不能标记为彻底 Production Decoupling。

只有同时满足：

```text
Cargo tree         无目标依赖
Production binary  无目标 binary / code path
Docker image       无目标 binary / assets / dependencies
Runtime service    无目标 service / route / worker / queue
```

才允许标记：

```text
FULLY DECOUPLED
```

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

## 三、二开削减计划器 (Change Planner: cargo x slim-plan 与 CodeGraph 协同)

在实施任何依赖瘦身与功能裁剪前，严禁凭直觉删除依赖。必须使用 Macro 二开专用依赖剖析工具 `cargo x slim-plan` 计算反事实边际削减（Counterfactual Marginal Reduction），并与 CodeGraph 协同定位代码修改点。

### 1. 核心命令与使用模式
```bash
# 模式 A：分析指定根包对目标依赖的切断收益与阻碍
cargo x slim-plan -p <root_crate> <target_crate>
# 示例：分析在 email_service 中切除 calendar_events
cargo x slim-plan -p email_service calendar_events

# 模式 B：全依赖树权重扫描，按真实边际削减体积降序排列
cargo x slim-plan -p <root_crate> --top-heavy
```

### 2. 反事实边际削减（Counterfactual Elimination）算法原理
在具有复杂依赖交织的 Monorepo 中：
- **Raw Transitive Closure（原始传递闭包）**：目标 crate 下游引用的所有依赖集合。如 `calendar_events` 下游包含 31 个 crate。
- **Real Marginal Removable Closure（真实边际可削减闭包）**：
  $$\text{Marginal} = \text{BaseClosure}(Root) \setminus \text{NewClosure}(Root \setminus \{Target\})$$
  模拟从 Root 的直接依赖中切断 Target 边后，真正能从 Root 构建图中彻底消失的 crate。
- **Shared Blockers（共享阻碍者）**：目标 crate 的下游依赖同时也被 Root 的其它分支（如 crm, email, auth）直接或间接依赖。只有所有引用分支全部切断，Shared Blockers 才会级联退出构建闭包。

### 3. Change Planner 标准工作流 (4 步法)
1. **第一步：边际削减测算**  
   执行 `cargo x slim-plan -p <service> <target>`，获取：
   - 依赖类型（mandatory / optional / feature-gated）
   - 激活该依赖的 Cargo Feature 集合
   - 真实净减 crate 数量与 Shared Blockers 清单。
2. **第二步：语法级调用排查 (CodeGraph)**  
   执行 `codegraph explore "<target>"` 与 `codegraph callers "<symbol>"`，秒级定位 service 中所有涉及该依赖的注入、路由、上下文与后台 Worker。
3. **第三步：条件编译与 Feature Gate (Zero-Overhead Decoupling)**  
   - 在 `Cargo.toml` 中设为 `optional = true`，并创建对应 feature（如 `calendar = ["dep:calendar_events"]`）；
   - 在 Rust 源码中使用 `#[cfg(feature = "...")]` 封装调用，并在 `not(feature = "...")` 分支提供轻量 stub 或空占位；
   - 对仅限该特性的子二进制（如 worker、openapi），在 `Cargo.toml` 中配置 `required-features`。
4. **第四步：双轨闭包与零漂移验证**  
   - 精简构建验证：`cargo check -p <service> --bin <service> --no-default-features`
   - 依赖树清零验证：`cargo tree -p <service> --no-default-features -i <target>`（确认无匹配）
   - Workspace 闭包同步与检查：`cargo run -p xtask -- deps` & `cargo run -p xtask -- deps --check`

---

## 四、真实实战样例：email_service 解耦 calendar_events

以 `email_service` 切断 `calendar_events` 为例：
1. **依赖可选化**：在 `services/email_service/Cargo.toml` 将 `calendar_events` 设为 `optional = true`，并引入 `calendar = ["dep:calendar_events"]`。
2. **路由与上下文 Gate**：
   - `src/api.rs`：使用 `#[cfg(feature = "calendar")]` 挂载 `/calendar` 路由；
   - `src/api/context.rs`：`CalendarGrantService` 与 `CalendarMutationSvc` 受 feature gate 保护；
   - `src/api/email/links.rs`：`/{link_id}/calendar` 仅在启用 calendar 时挂载；
   - `src/api/swagger.rs`：为有无 calendar 分别派生包含与精简版的 OpenAPI `ApiDoc`。
3. **Worker 与事件 Gate**：
   - `src/pubsub/context.rs`：`CalendarBackfillServices` 在未开启时提供 0 依赖空实现；
   - `src/pubsub/backfill/process.rs`：`BackfillOperation::CalendarGoogleBackfill` 在未开启时快速通过；
   - `pubsub_workers` binary：在 `Cargo.toml` 标记 `required-features = ["calendar"]`。
4. **效果**：在精简构建（未激活 `calendar` feature）下，`calendar_events` 从 `email_service` 编译依赖图中彻底剔除。

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

Status: PARTIALLY DECOUPLED / FULLY DECOUPLED

Method: <解耦手段简述，例如 Route gate + Cargo feature + Email build closure exclusion>

Boundary:
  Source: RETAINED / REMOVED
  UI: EXCLUDED / INCLUDED
  Dependency: EXCLUDED / INCLUDED
  Workspace: EXCLUDED FROM EMAIL PROFILE / INCLUDED
  Build: EXCLUDED / INCLUDED
  Image: EXCLUDED / INCLUDED
  Runtime: EXCLUDED / INCLUDED

Reachability:
  Compile: FALSE / TRUE
  Binary/Web Artifact: FALSE / TRUE
  Docker Image: FALSE / TRUE
  Runtime: FALSE / TRUE

Upstream Merge Risk: LOW / MEDIUM / HIGH

Physical deletion: NONE / <列出物理删除的文件清单>

Validation:
  <命令 1>: PASS / FAIL
  <命令 2>: PASS / FAIL
```

`Status: FULLY DECOUPLED` 只能在 `Reachability` 四项全部为 `FALSE`，且对应验证命令通过时使用。否则必须标记为 `PARTIALLY DECOUPLED`，并说明剩余可达层。
