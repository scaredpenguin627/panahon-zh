# 在 ChatGPT 云端跑这个项目

云端可以跑测试、检查代码、验证两个官方数据源是否还通。它跑不了界面，也装不到手机上。

APK 构建和真机验证留在本机做，那边已经有 SDK、NDK 和连着的数据线。

## 环境设置逐项

打开环境设置页，按下面填。

**代码仓库**

选 `scaredpenguin627/panahon-zh`，分支 `main`。当前 main 是 `0e1cdce`，包含重做后的界面和直连 EMB 的空气质量实现。

**安装脚本**

只填这一行，具体逻辑放在仓库里，方便改也方便看：

```sh
#!/bin/sh
set -eu
export CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup
export PATH="$CARGO_HOME/bin:$PATH"
for d in /workspace/panahon-zh /workspace/*/panahon-zh; do
  if [ -f "$d/scripts/cloud-install.sh" ]; then exec bash "$d/scripts/cloud-install.sh"; fi
done
echo "找不到 panahon-zh，请检查仓库挂载路径" >&2
exit 1
```

脚本会装 build-essential、rustup（1.88.0）、加 `aarch64-linux-android` 目标、预取依赖。可重复执行。

**启动技能**

`npx skills add leter/zh-tech-writing -g` 是给助手装一个中文写作规范，和构建无关。留着不影响，嫌慢可以删掉。

**公网访问**

「启用对所有公网地址的访问」保持关闭。「启用包管理器域名」必须打开，cargo 要靠它访问 crates.io。

「额外允许的域名」填这五个：

| 域名 | 用途 |
| --- | --- |
| `app.powerbi.com` | 解析 EMB 报表所在集群 |
| `*.analysis.windows.net` | 读取报表模型和查询数据（`wabi-*-api`） |
| `api.mobile.panahon.gov.ph` | PAGASA 天气、预报、预警、地点搜索 |
| `origin.panahon.gov.ph` | PAGASA 雷达、卫星、闪电图层 |
| `air-quality-api.open-meteo.com` | EMB 不可用时的 CAMS 位置预测 |

一个域名可以带通配符，所以 `*.analysis.windows.net` 一条就够，不用逐个列 `wabi-east-asia-a-primary-api` 之类的子域。

**HTTP 密钥和环境变量**

跑测试和探活都不需要，留空。

云端要出 APK 时才需要签名文件，见下面「在云端出 APK」。

**隐私**

「仅限我自己」即可。

**最后一步：保存并发布草稿**

配置文件有两个状态：草稿和已发布。只点「保存草稿」不会生效，运行中的实例仍按旧配置跑。

这就是之前 `/api/aqi` 报 `EAI_AGAIN app.powerbi.com` 的原因：域名没进运行实例的解析器。

点发布后，重新开一次会话再重试。

## 验证

进工作区后跑：

```bash
bash scripts/cloud-verify.sh
```

它会依次跑单元测试、工作区检查、Android 目标检查，最后用真实网络拉一次数据。

期望输出：

- 单元测试 10 个全过（`aqi_categories_follow_the_six_bands`、`dsr_rows_are_decoded_and_aliased` 等）。
- `cargo check` 无 error。
- 探活打印一行天气和一行空气质量，形如：

```
天气 OK：马尼拉 32°C 晴 · 体感 37°C · 56% · 1 km/h 北 · 降水 0.00 mm 无降雨
空气 OK：DENR-EMB（菲律宾环境管理局） · 16 站 · 平均 27 · 最差 50 纳沃塔斯
```

如果空气那行走的是 `Open-Meteo / CAMS`，说明 EMB 没读到。先确认域名和发布状态。

## 云端做不到的事

- **跑不了界面。** eframe 需要 OpenGL 和窗口系统，容器里没有。`cargo check` 能过，`cargo run` 会失败。
- **装不到手机。** 没有 adb，也没有设备。截图和交互验证只能在本机做。
- **没有 Android SDK/NDK。** 所以默认出不了 APK。

## 在云端出 APK

只有想要 CI 产物时才值得做，装完约 3 GB。

1. 额外允许 `dl.google.com` 和 `static.rust-lang.org`。
2. 安装脚本里加一个环境变量：

```sh
export PANAHON_INSTALL_ANDROID=1
```

脚本会装 JDK 17、cmdline-tools、platform 35、build-tools 35.0.1、NDK 27.2.12479018 和 `cargo-apk`。

3. 构建需要签名。把 keystore 用 base64 存进「HTTP 密钥」，构建时还原：

```bash
echo "$PANAHON_KEYSTORE_B64" | base64 -d > /tmp/release.jks
export CARGO_APK_RELEASE_KEYSTORE=/tmp/release.jks
export CARGO_APK_RELEASE_KEYSTORE_PASSWORD="$PANAHON_KEYSTORE_PASSWORD"
cargo apk build --release --features network --lib --target aarch64-linux-android
```

4. 产物在 `target/release/apk/panahon-zh.apk`。

签名必须和手机上的版本一致，否则装不上。换签名要先卸载旧包，会丢应用数据。

## 常见问题

**`EAI_AGAIN` 或域名解析失败**

草稿没发布，或者域名写错。发布后重开会话。

**`failed to find tool "clang.exe"`**

只在 Android 目标碰到。`cargo check --target aarch64-linux-android` 不需要 NDK，`cargo apk build` 需要。云端没装 SDK 时只能做前者。

**`cargo test --workspace` 卡在编译 eframe**

正常，第一次要编译几百个依赖。只验证数据层可以用 `-p panahon-core`，快很多。

**界面上出现方框**

改了中文文案但没重新生成字体子集：

```bash
python3 tools/subset-font.py --source <NotoSansCJKsc-Regular.otf>
```

**`/api/aqi` 连不上**

那是 `services/emb-aqi` 里的旧 Node 服务。Rust 版已经直连 EMB，不需要它。留着只作对照。

## 两边怎么分工

| 事项 | 云端 | 本机 |
| --- | --- | --- |
| 单元测试 | 可以 | 可以 |
| 数据源探活 | 可以 | 可以 |
| 界面调试 | 不行 | 可以 |
| 出 APK | 装 SDK 后可以 | 可以 |
| 装机和截图 | 不行 | 可以 |

日常流程：云端跑 `cloud-verify.sh` 确认数据源没坏，本机出包装机看效果。
