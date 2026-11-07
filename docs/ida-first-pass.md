# IDA Pro 第一轮分析

本项目先用 IDA Pro 9.5 分析 arm64 Flutter 原生库，再整理 Craft Anything 证据。

## 输入和命令

输入文件是 `<workspace>/libapp.so`。脚本位于 `tools/ida_first_pass.py`，可用下面的命令运行：

```powershell
$env:IDA_FIRST_PASS_OUT = "<workspace>/docs/ida-first-pass.json"
& "<IDA_INSTALL>\idat.exe" -A `
  -S"<workspace>\tools\ida_first_pass.py" `
  -L"<workspace>\docs\ida-first-pass.log" `
  "<workspace>/libapp.so"
```

## 结果

- IDA Pro 版本：`9.5`。
- 处理器：`ARM`。
- 已识别区段：`.rodata`、`.text`、`.bss` 和 `.eh_frame`。
- 原生库没有导出可直接使用的函数名，函数数量显示为 `0`。Flutter AOT 库经过裁剪，不能按普通应用源码恢复。
- 字符串中出现 `WeatherTile`、`AllAlertsTab`、`ManualObservation`、`Weather Station` 和公开 API 地址。这些字符串用于确认功能范围。

完整结果保存在本地生成的 `ida-first-pass.json`，日志保存在 `ida-first-pass.log`。这两个文件包含本机路径，默认不提交到公开仓库。后续工作使用 Craft Anything 读取包清单和 Flutter 分包信息，再编写独立 Rust 实现。

