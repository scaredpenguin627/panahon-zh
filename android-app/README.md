# 菲律宾天气 Android 前端

这是 Compose + Rust JNI 的原生 Android APK 前端。界面使用天气应用常见的信息层级和半透明卡片，动态背景由 Canvas 程序化绘制。参考视频没有作为运行时资源进入 APK。

## 开发运行

默认后端地址是 Android 模拟器可访问的 `http://10.0.2.2:8788`。真机或正式环境可以在设置页面修改地址。

```text
gradle assembleDebug
```

默认构建允许没有 Android NDK 的环境使用 Kotlin 背景回退。需要把 Rust JNI 编译进 APK 时使用：

```text
gradle assembleDebug -PwithRust=true
```

Rust Android 目标和 NDK 链接器需要预先安装。当前支持 `aarch64-linux-android`，也可以通过 `PANAHON_ANDROID_TARGET=x86_64-linux-android` 选择模拟器架构。

## 数据和缓存

前端调用 Rust HTTP 服务的天气、AQI、预警和地图接口。网络刷新成功后保存真实快照。网络失败时只显示最近一次成功快照，并标记为历史缓存。首次启动没有缓存时显示暂无数据和重试入口。

