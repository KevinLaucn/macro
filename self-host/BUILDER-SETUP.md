# Macro 独立 Linux Builder 搭建指南 (BUILDER-SETUP.md)

本指南针对为 `KevinLaucn/macro` 搭建专用的 **GitHub Actions Self-Hosted Linux Runner (macro-builder)**。

---

## 1. 架构定位原则

- **Builder ≠ 生产 VPS**：
  - 生产 VPS（运行路径 `/app/macro`）**绝对禁止**运行构建、编译或承载 GitHub Runner。生产机器只负责拉取已经构建好的 GHCR 镜像并启动容器。
  - Builder 是一台**独立的 Linux 编译机器**（例如单独的 Linux 开发机、本地物理机、专用云编译机或独立 VM）。

---

## 2. 推荐硬件配置

- **CPU**：4 核或以上（推荐 8 核 x86_64）
- **内存**：8 GB ~ 16 GB（构建 Rust/Nix 闭包建议至少 8GB 物理内存 + 4GB Swap）
- **磁盘**：100 GB ~ 200 GB SSD（用于持久化保存 `/nix/store` 以及 Docker 缓存层）
- **操作系统**：Ubuntu 22.04 / 24.04 LTS 或 Debian 12 (x86_64)

---

## 3. 基础依赖安装

在 Builder 主机上安装 Docker 和 Git：

```bash
sudo apt-get update && sudo apt-get install -y curl git jq build-essential

# 安装 Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER
```

---

## 4. 安装并启用 Nix

使用官方推荐的 Determinate Systems Nix 安装器（自动配置 Flakes 与多用户支持）：

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install

# 验证安装
nix --version
```

保证当前用户或后续的 GitHub Runner 用户对 Nix daemon 拥有访问权限（加入 `trusted-users`）：
在 `/etc/nix/nix.conf` 中追加：
```text
trusted-users = root runner
```
然后重启 Nix daemon：
```bash
sudo systemctl restart nix-daemon
```

---

## 5. 配置并注册 GitHub Actions Runner

在 GitHub 仓库中：
前往 **Settings** -> **Actions** -> **Runners** -> **New self-hosted runner** -> 选择 **Linux** / **x64**。

在 Builder 上执行注册命令，并特别注意**设置自定义标签**：

```bash
mkdir -p ~/actions-runner && cd ~/actions-runner

# 下载 Runner 安装包（根据 GitHub 页面提示的版本）
curl -o actions-runner-linux-x64.tar.gz -L https://github.com/actions/runner/releases/download/v2.322.0/actions-runner-linux-x64-2.322.0.tar.gz
tar xzf ./actions-runner-linux-x64.tar.gz

# 注册 Runner（指定 labels：macro-builder）
./config.sh --url https://github.com/KevinLaucn/macro \
  --token <YOUR_REGISTRATION_TOKEN> \
  --labels self-hosted,linux,x64,macro-builder \
  --unattended

# 作为 systemd 服务运行（随系统自启动）
sudo ./svc.sh install
sudo ./svc.sh start
```

---

## 6. 缓存与持久化优势

- **持久化 `/nix/store`**：每次构建生成的 Rust 依赖闭包都会常驻本地 `/nix/store`。下一次构建时，未修改的 crate 无需重复拉取和重复编译。
- **持久化 Docker Cache**：宿主机 Docker daemon 长期保持各阶段 layer 缓存。
- **隔离与安全**：此 Runner 仅用于 `KevinLaucn/macro` 信任的分支与 workflow_dispatch 调用，严禁运行不受信任的外部公共 PR 代码。
