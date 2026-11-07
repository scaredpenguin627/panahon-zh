# IDA Pro 第一轮分析

本项目先用 IDA Pro 9.5 分析 arm64 Flutter 原生库，再整理 Craft Anything 证据。

## 输入和命令

输入文件是 `D:\IDA PRO 9.5\_panahon_xapk_inspect\libapp.so`。脚本位于 `tools/ida_first_pass.py`，可用下面的命令运行：

```powershell
$env:IDA_FIRST_PASS_OUT = "D:\IDA PRO 9.5\panahon-zh\docs\ida-first-pass.json"
& "C:\Program Files\IDA Professional 9.5\idat.exe" -A `
  -S"D:\IDA PRO 9.5\panahon-zh\tools\ida_first_pass.py" `
  -L"D:\IDA PRO 9.5\panahon-zh\docs\ida-first-pass.log" `
  "D:\IDA PRO 9.5\_panahon_xapk_inspect\libapp.so"
```

## 结果

- IDA Pro 版本：`9.5`。
- 处理器：`ARM`。
- 已识别区段：`.rodata`、`.text`、`.bss` 和 `.eh_frame`。
- 原生库没有导出可直接使用的函数名，函数数量显示为 `0`。Flutter AOT 库经过裁剪，不能按普通应用源码恢复。
- 字符串中出现 `WeatherTile`、`AllAlertsTab`、`ManualObservation`、`Weather Station` 和公开 API 地址。这些字符串用于确认功能范围。

完整结果保存在 `ida-first-pass.json`。日志保存在 `ida-first-pass.log`。后续工作使用 Craft Anything 读取包清单和 Flutter 分包信息，再编写独立 Rust 实现。
