$ErrorActionPreference = "Stop"
$source = Join-Path $PSScriptRoot "..\IDA9.5汉化\MCP"
if (-not (Test-Path -LiteralPath $source)) { throw "MCP source folder was not found: $source" }
$destination = Join-Path $env:APPDATA "Hex-Rays\IDA Pro\plugins\ida-mcp"
New-Item -ItemType Directory -Path $destination -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $source "ida_mcp_plugin.py"),(Join-Path $source "ida-plugin.json"),(Join-Path $source "README.md") -Destination $destination -Force
Write-Host "Installed per-user IDA MCP plugin at $destination"
Write-Host "Close and reopen IDA 9.5 before testing the plugin."
