> 投敌了喵，mihomo真好用喵。
> 转移到我的[mihomo-tui](https://github.com/yaellona/mihomo-tui)喵。

## 介绍

这个是基于sing-box内核的tui。

> 注意，目前仅测试了vless协议，别的没测试过
>
> 我没有别的梯子了喵

## 用法

### 安装sing-box

只要能在终端直接输入sing-box即可。

安装方式：

| 系统      | 命令                        | 说明                                    |
| --------- | --------------------------- | --------------------------------------- |
| windows   | `winget install sing-box`   | 没有`winget`可以去微软商店下载          |
| archLinux | `paru -S sing-box`          | 我想无需多言                            |
| nixos     | `sudo nixos-rebuild switch` | `sing-box`添加到`configuration.nix`里面 |

> 其他的没用过或者不常用的就不说了

### 编译

拉取项目前先安装`rustup`,执行命令：

```bash
rustup install stable
```

安装完成后，进入项目文件夹，执行命令：

```bash
cargo build --release
```

编译完成后，可以在`./target/release`中找到编译的可执行文件。

## TODO

1. 添加linux的系统代理功能

2. 添加tun功能

3. tui自定义修改策略

4. 添加后台运行模式
