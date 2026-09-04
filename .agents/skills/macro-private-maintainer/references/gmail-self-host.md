# Gmail 与自托管邮件系统架构规范 (Gmail Self-Host Architecture Spec)

## 1. 核心链路模式

私有化自托管下的 Email 架构拓扑如下：

```text
Gmail / Google Workspace
         │ (Gmail REST API)
         ▼
Local Macro Email Service (`services/email_service`)
         │
         ▼
Local PostgreSQL (`crates/email_db_client`, `macro_db`) / Search / Storage
         │
         ▼
      Macro Web (`apps/web`)
```

- **邮件通道原则**：核心邮件交互全部基于 Google Gmail 官方 REST API（`gmail.googleapis.com`）。
- **无自建邮件服务器**：生产环境不部署 Postfix、Dovecot 或自建 SMTP/IMAP 服务端（除非未来用户明确要求）。
- **Pub/Sub 同步**：后台工作进程消费 Gmail Push/PubSub 事件，增量同步邮件线程与状态变化。

## 2. Gmail OAuth 认证原则

- **自建应用凭据**：生产与本地必须使用用户自建 Google Cloud Console OAuth 凭据：
  - `GOOGLE_CLIENT_ID`
  - `GOOGLE_CLIENT_SECRET`
  - `GOOGLE_REDIRECT_URI`
- **禁止依赖官方 App**：严禁回退到或引用 Macro 官方的 Google OAuth 凭据或跳转链接。
- **Token 存储**：OAuth Refresh Token 与 Access Token 密文落库至本地 PostgreSQL，不得向任何第三方外发。

## 3. Gmail 数据模型与落地

- 严禁猜测数据结构，严格对照：
  - `crates/email_db_client/migrations/`
  - `crates/email/` 与 `crates/models_soup/`
  - `services/email_service/src/pubsub/inbox_sync/`
- 核心字段规范：
  - `thread_id`、`message_id`：映射 Gmail 原始 ID。
  - `history_id`：追踪 Gmail 增量变更历史点。
  - `labels` / `system_labels`：映射 Gmail 标签与已读/未读状态。
  - `open_tracking`：本地自托管已读追踪支持。
