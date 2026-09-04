---
name: macro-private-maintainer
description: Master orchestration skill for maintaining, auditing, developing, and deploying KevinLaucn/macro. Enforces zero Macro cloud dependency, upstream compatibility, private VPS/Docker deployment, Gmail API architecture, and coordinates repo-local skills.
---

# Macro Private Maintainer

## Skill 定位

这是专门用于维护、审查、二开和部署以下 GitHub 仓库的总控 / 编排 Skill：

- **Repository**: `KevinLaucn/macro`
- **Default branch**: `main`

这是 Macro 官方开源项目的长期二开 fork。

### 主要目标

1. 保持与 Macro upstream 架构兼容
2. 实现 VPS 私有化部署
3. 尽可能消除对 Macro 官方云服务的运行时依赖
4. 保留 Gmail API 等明确需要的第三方服务
5. 维护 Email / Contacts / CRM / Search 等核心功能
6. 尽量减少侵入式修改，保证未来可以持续合并 upstream
7. 对所有修改执行架构、稳定性、安全性和回归审查

---

## 一、触发条件

当用户提出以下任何与 `KevinLaucn/macro` 有关的任务时自动使用此 Skill：

- Macro 源码分析
- Macro 二开
- Macro 私有化 / self-host
- Docker / VPS 部署
- Gmail 邮件同步 / Email Service
- Contacts / CRM
- Search
- PostgreSQL 数据结构
- OAuth
- WebSocket / connection_gateway
- Storage / LocalStack / MinIO / S3
- Macro 官方云依赖清理 / macro.com 外连审计
- telemetry / analytics 清理
- upstream merge
- Macro bug 修复 / 新功能开发
- Macro Rust 微服务调试
- Macro 前端 SolidJS 修改
- Macro AI tools
- Macro 数据库 migration/schema
- GitHub Actions / Docker image 构建
- 代码审查 / 发布前 QC

默认认为目标仓库是：`KevinLaucn/macro`，除非用户明确指定其他仓库。

---

## 二、必须优先使用 GitHub Connector / 本地源码真凭实据

所有涉及仓库当前代码、目录、Issue、PR、workflow、commit、branch 的判断：

- **必须先读取当前真实代码与 Git 状态**。
- **禁止仅凭模型记忆猜测 Macro 当前实现**。

优先检查：
- 当前 `main`
- 用户指定 branch
- 用户自己的修改
- 最近 upstream 改动
- 官方 `macro-inc/macro` 的相关 issue / PR

需要确认外部行为或官方文档时，再查：
- `docs.macro.com`
- `macro-inc/macro`
- 官方 GitHub Issues / Discussions

---

## 三、优先利用 CodeGraph 极速掌握依赖与调用链

仓库已完整建立本地代码智能关系图（基于 tree-sitter AST 解析的 SQLite 知识图谱，位于 `/Volumes/开发/macro/.codegraph`，包含 11,000+ 文件、150,000+ 节点与 500,000+ 边）：

- **定位符号与调用链**：查找函数/组件的调用方时，优先执行 `codegraph callers <symbol>`；查找它调用的下游时，执行 `codegraph callees <symbol>`。
- **改动影响面评估**：重构或清理函数/接口前，使用 `codegraph impact <symbol>` 审查受影响范围。
- **按文件审查引用关系**：使用 `codegraph node <filepath>` 查看文件包含的符号及上游依赖文件。
- **快速符号检索**：使用 `codegraph query <symbol>` 精准定位符号定义与声明。
- **更新索引状态**：在做完大批量文件变更后，可通过 `codegraph sync /Volumes/开发/macro` 快速增量同步索引。
- **原则**：严禁无目的地使用 grep 盲搜上万个文件，优先通过 CodeGraph 获取结构化、确定性的语法级关联。

---

## 四、引用 Macro 官方 repo-local Skills

不要复制官方技能的全部规则到本 Skill。在执行任务前，根据任务类型读取并遵循仓库 `.agents/skills/` 中的相关 Skill：

### 1. `cloud-storage-hexagonal-architecture`
- **路径**：`.agents/skills/cloud-storage-hexagonal-architecture/SKILL.md`
- **适用场景**：修改或审查 `crates/**`、`services/**`、Rust backend、database adapter、external HTTP client、S3 / Storage、Redis、OpenSearch、authorization、inbound/outbound/domain、service client 时必须读取。
- **原则**：必须保持 Macro 的 hexagonal / ports-and-adapters 架构。`domain` 不直接依赖 SQLx、AWS SDK、Redis、HTTP client、reqwest、环境变量或传输 DTO。外部服务替换优先通过 outbound adapter 完成。

### 2. `debug-service`
- **路径**：`.agents/skills/debug-service/SKILL.md`
- **适用场景**：Rust service 无法启动、运行时错误、Docker container crash、API 500、connection refused、service startup failure 时优先使用。

### 3. `dump-schema`
- **路径**：`.agents/skills/dump-schema/SKILL.md`
- **适用场景**：分析 PostgreSQL、Email 表结构、Gmail message/thread 数据、attachments、contacts、users、workspace、migrations 时优先使用。

### 4. `qc`
- **路径**：`.agents/skills/qc/SKILL.md`
- **适用场景**：任何较大代码修改完成后，执行对应 QC 思路（Code Review, Simplification, Consistency, Robustness, Scope），特别检查是否为了小功能侵入过多 upstream 代码。

### 5. `dependabot`
- **路径**：`.agents/skills/dependabot/skill.md`
- **适用场景**：处理 Cargo, Bun, npm, pnpm 依赖升级与安全漏洞。

### 6. `create-ai-tool`
- **路径**：`.agents/skills/create-ai-tool/SKILL.md`
- **适用场景**：增加或修改 Macro AI Tool。优先设计 read-only AI tools（如 AI 查询历史邮件、搜索历史沟通）。未经用户明确要求，禁止赋予 AI 自动发信、删信或修改数据的权限。

### 7. `upgrade-model`
- **路径**：`.agents/skills/upgrade-model/SKILL.md`
- **适用场景**：用户明确要求修改 Macro AI Chat 模型时读取。

---

## 四、我们的私有化原则

- **核心目标**：`Zero Macro Cloud Dependency`（零 Macro 官方云依赖）。
- **边界定义**：不是 `Zero Internet`。允许必要的第三方 SaaS/API（如 Gmail 访问 Google）。

### 默认允许的外部服务
- **Google**：`accounts.google.com`、`oauth2.googleapis.com`、`gmail.googleapis.com`、`googleapis.com`
- **GitHub**：仅当用户启用 GitHub integration 时
- **用户自选指定**：自定义 LLM API、用户自建 OAuth Provider、CDN、外部 Webhook

### 默认禁止依赖 Macro 官方云
审查所有 `*.macro.com`，包括：
`macro.com`、`dev.macro.com`、`mcp-server.macro.com`、`connection-gateway.macro.com`、`cloud-storage`、`auth-service`、`document-cognition`、`notification`、`contacts`、`email service`、`websocket service`。

- **原则**：本地已有对应服务必须连接本地服务；本地暂时无对应服务，明确标记 `LOCALIZATION GAP`，禁止静默 fallback 到 Macro 官方云。

---

## 五、禁止隐式官方云 fallback

代码中若出现「本地服务失败 → 自动请求 macro.com」，视为高风险 P1 缺陷！
必须遵循：
`本地请求失败 → 返回明确错误 → UI graceful degradation（优雅降级）`。

特别审查：
`safeFetch`、`fetchWithToken`、`platformFetch`、各 service clients、websocket 初始化、OAuth 回调。

---

## 六、Gmail 架构原则

我们的 Email 模式为：
```text
Gmail / Google Workspace
         │ (Gmail REST API)
         ▼
Local Macro Email Service
         │
         ▼
Local PostgreSQL / Search / Storage
         │
         ▼
     Macro Web
```
不部署 Postfix、Dovecot 或自建 SMTP server（除非用户未来明确要求）。

### Gmail OAuth
必须使用用户自建凭证：`GOOGLE_CLIENT_ID`、`GOOGLE_CLIENT_SECRET`、`GOOGLE_REDIRECT_URI`。严禁依赖 Macro 官方 Google OAuth App。

### Gmail 数据落地
通过 migration、schema、`crates/email`、`email_service` 明确 thread metadata、message body、labels、attachments 等的落地逻辑，不凭猜测判断。

---

## 七、附件策略

不要默认强制所有 Gmail attachment 二进制永久保存本地。先确认 Macro 现有架构：
1. Gmail attachment reference
2. Macro local file ingestion
3. Search indexing
4. PDF text parsing
5. Object storage

- 若仅用于前端查看：优先保留 Gmail API lazy fetch。
- 若 AI / 全文搜索 / PDF 解析依赖本地附件：保留 Macro 官方 ingestion pipeline。
- 不得未经分析直接剔除 storage pipeline。

---

## 八、WebSocket / Realtime

审查 `connection_gateway`、`websocket_service`、`collaboration` 及其重试行为。
- VPS self-host 下优先使用本地 WebSocket。
- 严禁客户端向 `wss://*.macro.com` 无限重试。
- 若某功能未启用 realtime，静默关闭对应连接，杜绝 Console error storm、无限 retry 与页面崩溃。

---

## 九、Telemetry / Analytics

所有遥测默认必须可配置关闭。重点核查：
`packages/observability`、Sentry、PostHog、Datadog、OpenTelemetry exporter 等。
- 允许：本地日志、本地 Prometheus、本地 OpenTelemetry collector。
- 严禁向第三方外漏用户邮箱、服务器 IP、工作区数据、文档及邮件敏感元数据。

---

## 十、代码修改准则

修改优先级：
1. 配置解决
2. 环境变量（Environment variable）
3. Adapter 替换
4. 依赖注入（Dependency injection）
5. Wrapper / Proxy 反代
6. 小范围 patch
7. 最后才考虑修改核心 domain

**目标**：`Minimize Fork Divergence`（最小化分叉差异）。每次修改必须确保未来 `upstream merge` 轻松平滑。

---

## 十一、禁止无意义删除功能

不要因为当前不用某模块就大面积删除 upstream 代码（如 Docs、Canvas、Calls、Agents 等）。
- 优先采用：`disable` / `feature flag` / `no-route` / `no-start`。
- 避免直接删除源码，将合并冲突降至最低。

---

## 十二、代码修改后的验收标准

- **Rust**：`cargo fmt`、`cargo check`、`cargo clippy`、`cargo test -p <affected crate>`
- **Frontend**：`bun format`、`bun check`，必要时 `bun gen-api` / `bun gen-tools`
- **Docker**：`docker compose config`、容器健康检查、日志审查
- **Database**：`migration validation`、`schema diff`

---

## 十三、私有化网络验收标准

在完成自托管任务后，必须给出严密的网络验证方式：
1. **浏览器 DevTools Network**：过滤 `macro.com`，验证请求数为 0。
2. **服务器侧验证**：DNS / firewall / proxy logs 验证 `*.macro.com = 0 runtime requests`。
3. **白名单核验**：确保仅用户明确批准的 Google Gmail API / OAuth / 自选 LLM Provider 通过。

---

## 十四、风险分级

- **P0 / Critical**：用户邮件、密钥、Token 直接外发至 Macro 官方服务器。
- **P1 / High**：本地请求失败隐式自动 fallback 到官方云服务。
- **P2 / Medium**：无敏感数据的第三方 telemetry / 上报。
- **P3 / Low**：构建阶段（build-time）访问官方文档或通用源。

---

## 十五、标准响应输出格式

### 代码审计类任务
1. **结论**：一句话说明当前是否安全 / 可行。
2. **发现**：Markdown 表格汇总（Severity, Module, File, Behavior, External Endpoint, Fix）。
3. **改造方案**：最小侵入路径。
4. **需要修改的文件**：精确路径列表。
5. **验证**：具体测试命令或网络验证流程。

### Bug 修复类任务
1. **Root Cause**
2. **Affected Files**
3. **Fix**
4. **Regression Risk**
5. **Test**

### 部署类任务
1. **Required Containers**
2. **Optional Containers**
3. **Environment Variables**
4. **Reverse Proxy**
5. **Persistent Volumes**
6. **Health Checks**
7. **External Allowlist**

---

## 十五.一、生产环境基础设施连接规范 (Production VPS SSH & Deploy Spec)

在执行远程部署、运维排障、日志审查或容器重启时，使用以下标准生产环境凭据与配置：

- **主机别名**: `tencent-us2h8g`
- **服务器 IP**: `43.135.149.165`
- **SSH 端口**: `18222`
- **登录用户**: `root`
- **密钥路径**: `~/.ssh/us2h8g.pem`
- **SSH 配置模板**:
  ```ssh-config
  Host tencent-us2h8g
      HostName 43.135.149.165
      User root
      Port 18222
      IdentityFile ~/.ssh/us2h8g.pem
      IdentitiesOnly yes
      PubkeyAuthentication yes
      PreferredAuthentications publickey
      PasswordAuthentication no
      ServerAliveInterval 30
      ServerAliveCountMax 6
      TCPKeepAlive yes
  ```
- **生产部署绝对路径**:
  - Compose 与环境文件：`/app/macro/docker-compose.yml`、`/app/macro/.env`
  - 数据持久化目录：`/app/macro/data/` (`postgres/`, `redis/`)
  - 1Panel / OpenResty 反向代理站点配置：`/app/1panel/www/conf.d/macro.chnprints.com.conf`
  - 1Panel / OpenResty 站点专属反代扩展目录：`/app/1panel/www/sites/macro.chnprints.com/proxy/root.conf`
- **生产运维高频检查命令**:
  - 容器状态：`ssh -p 18222 -i ~/.ssh/us2h8g.pem root@43.135.149.165 "docker ps -a"`
  - 服务健康状态：`ssh -p 18222 -i ~/.ssh/us2h8g.pem root@43.135.149.165 "docker logs --tail 100 macro-web"`
  - 生产镜像快速更新拉取：`ssh -p 18222 -i ~/.ssh/us2h8g.pem root@43.135.149.165 "cd /app/macro && docker compose pull macro-web && docker compose up -d --force-recreate macro-web"`


---

## 十六、核心重点保护业务

重点保护四大核心支柱：
- **Email**（Gmail 客服、同步、写信）
- **Contacts**（联系人）
- **CRM**（客户沟通全流程、历史时间线）
- **Search**（历史邮件与文档检索）

---

## 十七、禁止事项清单

- 严禁在未读取当前代码的情况下猜测实现
- 严禁将 Macro Hosted 架构当成本地架构
- 严禁为解决单一小问题大面积重构 upstream
- 严禁引入新的 `macro.com` 外部依赖
- 严禁硬编码 VPS IP、OAuth Secret、API Key，严禁提交 Secret 至 Git
- 严禁未经测试随意修改数据库 migration
- 严禁为简单需求增加冗余沉重微服务

---

## 十八、Upstream 合并与版本维系

- 任何功能开发需保持 upstream 兼容，单独放置 adapter 或独立配置。
- 保持 `git fetch upstream` 与 `git merge upstream/main` 的低成本合并能力。
- 若 upstream 官方已实现相同能力，优先采用官方标准方案，主动废除自身过时的临时 patch。

---

## 十九、开发规范与辅助目录归类

本仓库包含若干面向 AI / IDE / 构建系统的辅助目录。执行产品瘦身、私有化清理或删除目录前，必须先区分它们是否属于产品运行时。

### `.claude/`：保留

`.claude/` 是 Claude Code 使用的开发规范、命令和 repo-local skills，不是 Macro 产品运行时功能，也不是 VPS 部署必需项，但对长期二开维护有价值。

默认策略：
- 保留 `.claude/`，作为开发规范与 AI 辅助维护资料。
- 不把 `.claude/` 视为可随 Phase 1 产品精简一起删除的业务模块。
- 若未来不再使用 Claude Code，可先把仍有价值的规范迁移到 `.agents/skills/` 或 `AGENTS.md`，再考虑删除。
- 修改 `.claude/skills/*` 时，保持与 `.agents/skills/*` 的职责一致，避免两套规范互相冲突。

### `.cursor/`：可清理候选

`.cursor/` 主要是 Cursor / Cursor Cloud 专用开发环境配置和 MCP 调试配置，不属于 Macro 产品功能。

默认策略：
- 本地二开与 VPS 私有化部署不依赖 `.cursor/`。
- 如果不再使用 Cursor Cloud Agent，可以纳入清理范围。
- 删除前检查是否仍有人依赖 `.cursor/environment.json`、`.cursor/stack.sh`、`.cursor/infra.sh`、`.cursor/rebuild.sh` 或 `.cursor/mcp.json` 做远程开发。

### `.sqlx/`：必须保留

`.sqlx/` 是 SQLx 离线查询元数据缓存，Rust build、clippy、CI 中的 `SQLX_OFFLINE=true` 会依赖它。

默认策略：
- 禁止删除 `.sqlx/`。
- 禁止手动编辑 `.sqlx/query-*.json`。
- 修改 Rust SQL 查询或 migration 后，从仓库根目录运行：
  ```bash
  nix develop --command just prepare_db
  ```
