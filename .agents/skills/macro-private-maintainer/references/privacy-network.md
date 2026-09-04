# 隐私隔离、外连审计与遥测规约 (Privacy & Network Allowlist Spec)

## 1. 外部网络连接白名单 (Network Allowlist)

### 严格允许的外部服务
- **Google API**：`accounts.google.com`、`oauth2.googleapis.com`、`gmail.googleapis.com`、`googleapis.com`（仅用于用户自持凭据下的 Gmail 同步与认证）。
- **GitHub API**：仅当用户明确开启 GitHub 集成或更新检查时。
- **用户自选指定服务**：用户显式配置的第三方 LLM API、自定义 OAuth Provider、自建 CDN、Webhook 外部接收端。

### 严格禁止的外部请求 (Zero Macro Cloud)
- **全部官方域名**：`macro.com`、`dev.macro.com`、`*.macro.com`（包括 `auth-service`、`document-cognition`、`mcp-server`、`notification`、`cloud-storage`、`websocket` 等）。
- **零隐式 Fallback**：严禁出现「本地服务未响应或返回错误 → 自动 fallback 转发给官方云」的隐蔽请求逻辑。必须返回确定性错误并在 UI 层进行 graceful degradation。

## 2. 遥测与埋点阻断 (Telemetry Blocking)

- **默认全部禁用**：关闭任何第三方埋点上报 SDK（如 Sentry、PostHog、Datadog 等向公网云端上报）。
- **允许的监控形式**：允许向部署在内网环境的本地 Prometheus、本地 OpenTelemetry Collector 输出度量，允许本地日志落盘。
- **隐私红线**：严禁向任何外部第三方传输用户邮箱地址、服务器 IP、工作区敏感数据、文档与邮件正文元数据。
