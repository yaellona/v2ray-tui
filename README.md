## 介绍

这个是基于sing-box内核的tui。

> 注意，目前仅测试了vless协议，别的没测试过
>
> 我没有别的梯子了喵

## 安装

### 1. 安装 sing-box

本项目依赖 sing-box 内核，请确保 `sing-box` 命令可用。

| 系统      | 命令                        | 说明                                    |
| --------- | --------------------------- | --------------------------------------- |
| windows   | `winget install sing-box`   | 没有`winget`可以去微软商店下载          |
| archLinux | `paru -S sing-box`          | 我想无需多言                            |
| nixos     | `sudo nixos-rebuild switch` | `sing-box`添加到`configuration.nix`里面 |

### 2. 下载 ladderust

从 [GitHub Releases](https://github.com/yaellona/v2ray-tui/releases) 下载对应平台的压缩包：

| 系统      | 文件                                  |
| --------- | ------------------------------------- |
| Windows   | `ladderust-x86_64-pc-windows-msvc.zip` |
| Linux     | `ladderust-x86_64-unknown-linux-gnu.tar.gz` |
| macOS     | `ladderust-x86_64-apple-darwin.tar.gz` |

解压后将 `ladderust`（或 `ladderust.exe`）放到 PATH 目录下即可。

### 从源码编译

```bash
rustup install stable
git clone https://github.com/yaellona/v2ray-tui.git
cd v2ray-tui
cargo build --release
```

编译产物在 `./target/release/ladderust`。

## 发布

推送 `v*` 标签自动触发 GitHub Actions 构建并发布：

```bash
git tag v0.1.0
git push origin v0.1.0
```

## TODO

1. 添加linux的系统代理功能

2. 添加tun功能

3. tui自定义修改策略

4. 添加后台运行模式
