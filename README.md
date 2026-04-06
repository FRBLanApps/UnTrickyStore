# UnTrickyStore

让 TrickyStore 的配置变得不再棘手。

> [!IMPORTANT]
> 本模块**专精**伪装引导加载程序状态与自动化 TrickyStore 配置，**而非**通过 Play Integrity。

## 前提条件

- 已安装 [TrickyStore](https://github.com/5ec1cff/TrickyStore) 或其分支（如 [TrickyStoreOSS](https://github.com/beakthoven/TrickyStoreOSS)、[TEESimulator](https://github.com/JingMatrix/TEESimulator)）
- 挂载系统**不是** OverlayFS（需使用 Magic Mount 或 Meta Module）

## 安装

1. 刷入模块并重启设备。
2. 按需在 WebUI 中调整配置（可选）。
3. 完成！

## 功能

### 核心

- **冲突模块处理** — 检测 30+ 冲突模块并自动标记移除；检测冲突应用并提示卸载；如有冲突则阻止启动并发送系统通知
- **目标列表管理** — 接管 TrickyStore 的 `target.txt`，自动根据已安装应用生成；inotify 实时监控变更
- **VBMeta Hash 修正** — 启动时自动修正异常的 VerifiedBootHash 属性
- **引导加载程序伪装** — 启动时伪装引导加载程序状态为锁定
- **安全补丁级别同步** — 将安全补丁级别同步到系统属性
- **Root 环境检测** — 自动检测 Magisk / KernelSU / APatch 及多 Root 共存场景
- **OnePlus 设备适配** — 自动将 OnePlus 特有系统应用加入目标列表

### WebUI

- 导出日志
- 目标列表管理
- 自定义安全补丁级别
- 联网拉取 Pixel 更新公告的最新安全补丁级别
- TrickyStore 后台服务管理（停止 / 启动 / 重启）
- UnTrickyStore 后台服务管理（停止 / 启动 / 重启）

> [!NOTE]
> **WebUI 支持**
> - **KernelSU / APatch** — 原生支持
> - **Magisk** — 通过 [MMRL](https://github.com/MMRLApp/MMRL) 或 [KSUWebUIStandalone](https://github.com/5ec1cff/KsuWebUIStandalone) 访问（未安装时自动安装 KSUWebUIStandalone）

### CLI

在终端以 Root 身份执行：

```sh
PATH="/data/adb/modules/untrickystore/bin:$PATH"
```

| 命令 | 说明 |
|------|------|
| `uts daemon` | 启动 inotify 监控守护进程 |
| `uts rootdetect` | 检测并输出 Root 环境 |
| `uts propstate` | 伪装引导加载程序状态 |
| `uts vbhash` | 修正 VBMeta Hash |
| `uts conflict-mod` | 检测冲突模块 |
| `uts conflict-app` | 检测冲突应用 |
| `uts target` | 重新生成 target.txt |
| `uts patch-sync` | 同步安全补丁级别 |
| `uts patch-fetch` | 联网拉取最新安全补丁级别 |
| `uts ts-ctl [stop\|start\|restart]` | TrickyStore 服务控制 |
| `uts uts-ctl [stop\|start\|restart]` | UnTrickyStore 服务控制 |
| `uts status` | 刷新模块描述 |
| `uts init-cfg` | 初始化配置目录 |

### 其他

- 在模块描述中显示运行环境和启动结果
- 根据系统语言显示 zh-Hans 或 en-US（运行状态 / 安装过程）
- OverlayFS 挂载系统自动拒绝安装

## 配置

- 配置目录：`/data/adb/untrickystore`
- 日志文件：`/data/adb/untrickystore/log/log.log`

遇到问题请创建 Issue 并附上日志。

## 构建

需要：JDK 21、Rust nightly（`aarch64-linux-android` target）、Android NDK 29、cargo-ndk、gettext

```sh
./gradlew :module:zipRelease
```

产物：`module/build/outputs/release/*.zip`（可刷入模块包）

## 许可证

[AGPL-3.0-or-later](LICENSE)

Copyright (C) 2025-2026 FRBLanApps
