# Craft Anything 分析记录

本目录保存 Craft Anything 的原始分析结果。分析对象是从 XAPK 解出的基础 APK 和 arm64 分包。

## 使用的命令

```powershell
craft-anything-cli --deep android-package-show --path "<base-apk>" --output-format json
craft-anything-cli --deep android-flutter-identify --path "<arm64-split-apk>" --output-format json
```

## 关键结果

- 包名为 `com.dostpagasa.panahon`，版本为 `1.1.4`，版本号为 `48`。
- 最低 SDK 为 `29`，目标 SDK 为 `36`。
- 基础 APK 有启动 Activity。
- arm64 分包包含 `libapp.so` 和 `libflutter.so`，识别结果为 Flutter，首选 ABI 为 `arm64-v8a`。
- 原清单包含 Firebase 和 Google Play 组件。Rust 重建没有加入 FCM、GSF、Play License 和设备注册依赖。

这些结果只用于确定功能范围和兼容性要求。Rust 工程没有复制原应用的 Flutter 二进制或打包资源。


