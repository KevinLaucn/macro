# 生产环境部署与运维架构规范 (Production Deployment & Operations Spec)

## 1. 生产环境拓扑

- **容器化引擎**：Docker Compose，统一编排 Web 前端、核心 Rust 服务集群、PostgreSQL、Redis、OpenSearch 等。
- **反向代理与 SSL**：基于 1Panel / OpenResty 或 Nginx 托管，负责公网 SSL 终结、请求限流与反向代理路由。
- **配置分离**：业务配置和敏感环境变量落盘于 `/app/macro/.env`，模板见 `self-host/.env.example`。

## 2. 生产镜像发布与 Profile 体系

- **Full Profile**：
  - 服务镜像：`ghcr.io/kevinlaucn/macro-services:$VERSION`
  - 数据库迁移镜像：`ghcr.io/kevinlaucn/macro-init:$VERSION`
- **Email Profile（生产推荐精简闭包）**：
  - 服务镜像：`ghcr.io/kevinlaucn/macro-services-email:$VERSION`
  - 数据库迁移镜像：`ghcr.io/kevinlaucn/macro-init-email:$VERSION`
- **构建原则**：生产 VPS 主机禁止承担任何高负载编译任务，镜像统一由独立专用编译机或 GitHub Actions 预构建发布后 pull 拉取。

## 3. 运维标准指令范式 (命令模板)

- **容器状态核查**：`ssh <host_alias> "docker ps -a"`
- **服务健康与实时日志**：`ssh <host_alias> "docker logs --tail 100 <service_name>"`
- **单服务优雅更新**：`ssh <host_alias> "cd /app/macro && docker compose pull <service_name> && docker compose up -d --force-recreate <service_name>"`
- **真实连接凭据**：详细主机 IP、端口与专用 SSH 密钥路径请直接查阅被本地 `.gitignore` 保护的 `.agents/skills/macro-private-maintainer/.local-production.md`。
