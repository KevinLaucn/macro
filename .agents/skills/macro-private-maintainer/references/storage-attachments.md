# 存储与邮件附件策略规范 (Storage & Attachments Architecture Spec)

## 1. 附件生命周期与获取策略

邮件附件处理不应粗暴一刀切，必须按使用场景区分策略：

1. **前端查看 / 交互**：
   - 优先采用 Gmail API Lazy Fetch 机制，前端需要预览或下载附件时，由 `email_service` 通过用户授权 token 按需直连 Gmail 拉取并透传给前端，避免本地存储无节制膨胀。
2. **全文检索与 AI 分析**：
   - 若系统启用了 OpenSearch 历史全文检索、PDF 文本解析或本地附件索引，保留 Macro 官方 local file ingestion 与 pipeline 处理逻辑。
   - 不得未经调用链分析直接物理移除 storage pipeline。

## 2. 对象存储适配原则

- **接口抽象**：底层统一走 Macro Hexagonal Storage Port（`document_storage_service` / `static_file_service`）。
- **后端适配**：
  - 本地开发：可使用 LocalStack S3 或 MinIO 模拟 S3。
  - 生产环境：支持本地 MinIO、标准 AWS S3 或兼容 S3 协议的自建对象存储。
- **直传网关**：大附件通过 presigned URL 或 `static_file_service` 直接传输，避免主 API 进程承受过载流量。
