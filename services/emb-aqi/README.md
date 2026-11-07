# EMB 空气质量数据接入说明

这两个页面都没有可供调用的数据接口。数据全部在 Power BI 公开报表里，页面只是外壳。

本目录给出一套已经验证可用的取数方案：服务端把 Power BI 报表读出来，转成干净的 JSON，手机 App 直接调。

## 现状

- `https://ncr.emb.gov.ph/realtimeairmonitoring/` 是 WordPress 页面，正文里只有一个 `<iframe>`，指向 `app.powerbi.com`。
- `https://air.emb.gov.ph/ambient-air-quality-monitoring/` 同样是 WordPress 页面，嵌了 6 个 Power BI 报表。
- 两个站点都不返回任何空气质量数字。抓 HTML 拿不到数据。
- 这两个站挂了 Azure WAF。请求头里没有浏览器 UA 时，直接返回 403。本目录的代码已经带上 UA。

报表清单（`r=` 参数里的 resourceKey）：

| resourceKey | 报表名 | 状态 |
| --- | --- | --- |
| `156a380e-4084-4645-8a01-6a1e845d7416` | AQI_new | 可用，16 个监测站，PM10 + PM2.5 |
| `92487219-adb0-465a-b67f-3e4977860565` | AQI_new_GAS | 可用，模型不同（SO2 / O3 / NO2 + 气象） |
| `e1287f74-472f-4def-802b-19ce33ff0982` | AQI | 可用，模型不同 |
| `f1043bab-727d-4db7-b8db-60995f84b4e3` | 无 | 已失效，返回 401 |
| `a2048358-8160-47b6-9b73-bbb61c53091a` | Batangas | 已失效，返回 401 |
| `aa67617c-2a22-4930-b72b-12d19fb1ba2b` | Antipolo | 已失效，返回 401 |

后三个报表在 EMB 自己的页面上也打不开。这不是抓取姿势的问题。

本目录默认只处理第一张表 `AQI_new`。它同时被两个页面引用，是 NCR 空气质量模块的主数据源。

## 数据是怎么取出来的

Power BI 的匿名嵌入（Publish to Web）走三段式协议。浏览器打开报表时做的也是这三步。

1. **解析参数。** iframe 的 `r=` 是 base64，解开是 `{"k":"<resourceKey>","t":"<tenantId>","m":10}`。
2. **找集群。** 请求 `app.powerbi.com/view?r=...`，从返回的 HTML 里读 `resolvedClusterUri`。把域名里的 `-redirect` 换成 `-api`，才是数据接口。
3. **取模型。** `GET {cluster}/public/reports/{resourceKey}/modelsAndExploration?preferReadOnlySession=true`，响应里有 `modelId`，还有报表的分页和视觉对象结构。
4. **换数据。** `POST {cluster}/public/reports/querydata?synchronous=true`，请求体是一段语义查询（本目录叫"配方"），返回 DSR 格式的数据。
5. **解码 DSR。** 值是按字典下标压缩存的，时间和整数还有各自的编码方式。解码逻辑在 `src/powerbi.mjs` 的 `decodeDsr`。

第 4 步有三个请求头不能少：`X-PowerBI-ResourceKey` 决定查哪张报表，`RequestId` 和 `ActivityId` 缺了会被服务端直接断开连接。

配方是从浏览器里抓下来的固定 JSON，存在 `recipes/` 下。三份配方分别对应 PM10 表、PM2.5 表和"数据时间"卡片。

## 跑起来

需要 Node 18 以上。没有第三方依赖。

```bash
cd emb-aqi
node src/collect.mjs      # 打印一份完整 JSON
node server.mjs           # 起 HTTP 服务，默认 8787
```

`collect.mjs` 的输出节选：

```json
{
  "updatedAt": "2026-11-07T18:00:00.000Z",
  "dataClock": "2026-11-07 18:00",
  "summary": { "stationCount": 16, "avgAqi": 39, "maxAqi": 63, "maxStation": "纳沃塔斯", "category": "良" },
  "stations": [
    {
      "id": "navotas",
      "name": "Navotas",
      "nameZh": "纳沃塔斯",
      "address": "Navotas City Hall",
      "pm10": 63,
      "pm25": 54,
      "aqi": 63,
      "pollutant": "PM10",
      "category": "良",
      "color": "#E8C400",
      "remark": "Fair",
      "remarkZh": "良"
    }
  ]
}
```

## 接口

`GET /api/aqi` 返回全部站点，缓存 5 分钟。`GET /api/aqi?refresh=1` 强制刷新。`GET /health` 用来探活。

| 字段 | 说明 |
| --- | --- |
| `updatedAt` | 监测数据时间，ISO 8601。取自报表的 `Min(CAMS.Date_Time)` |
| `dataClock` | 报表上的钟点读数，给 UI 直接显示用 |
| `fetchedAt` | 本次抓取时间 |
| `summary` | 站点数、平均 AQI、最差站点、总体分级 |
| `stations[].aqi` | 综合 AQI，取 PM10 与 PM2.5 中较大的一个 |
| `stations[].pollutant` | 首要污染物，`PM10` 或 `PM2.5` |
| `stations[].category` | 中文分级：优、良、轻度污染、中度污染、重度污染、严重污染 |
| `stations[].color` | 对应分级的色值，UI 直接用 |
| `stations[].remark` | EMB 原始评级，多数站点为空，空的时候用 `category` |

站点已按 AQI 从差到好排序，App 里直接渲染即可。

## 接到 App

`clients/` 下有两个可直接用的客户端：`aqi_client.dart`（Flutter）和 `aqiClient.ts`（React Native / Expo）。它们都带 5 分钟内存缓存。

三点注意：

- Android 模拟器访问电脑上的服务，地址写 `10.0.2.2`，不是 `127.0.0.1`。
- 真机要写局域网 IP，或者把服务部署到公网。
- iOS 和 Android 9 以上默认禁止明文 HTTP。上线前给服务配上 HTTPS，本地调试则在各自平台配置里放行。

## 直连还是走中间服务

两种都能用，差别在这。

| | App 直连 Power BI | 走中间服务 |
| --- | --- | --- |
| 实现 | 把配方 JSON 打进 App，用 `http` 直接 POST | App 只调 `/api/aqi` |
| 报表改版 | 必须发新版 App | 改服务端就行 |
| WAF、UA、时区 | 每个端都要处理一遍 | 服务端处理一次 |
| 成本 | 无 | 一台小机器或一个 Serverless 函数 |

建议走中间服务。配方是 Power BI 的内部格式，EMB 随时可能改报表，改一次就得发版很难受。

## 已知限制

- **三张报表已失效**，见上面的清单。EMB 页面上的 `Batangas`、`Antipolo` 两个 iframe 现在是空的。
- **另两张报表的模型不同。** `AQI_new_GAS` 是 86 个卡片视觉对象，没有站点表格。要接它得用 `tools/capture-recipe.mjs` 单独抓配方。
- **时间字段有时区歧义。** 报表里 `Date_Time` 存的是当地钟点，但按 UTC 传输。所以 `updatedAt` 会比马尼拉时间早 8 小时，`dataClock` 才是报表上显示的那个数。UI 上建议显示"更新于 X 分钟前"，用 `fetchedAt` 算，别用 `updatedAt`。
- **`remark` 大多为空。** EMB 只在部分站点填了评级，所以代码用数值自己算 `category`，两套并存。
- **报表改版会坏。** 配方里写死了表名和列名（`CAMS.Station`、`CAMS.PM10AQI` 等）。改了就得重抓配方。
- **别高频抓。** 监测站每小时更新一次，5 到 10 分钟拉一次足够。本服务默认 5 分钟缓存，并发请求只打一次上游。

## 换数据源的建议

如果你的 App 主要面向普通用户，只想要一个能显示的 AQI 数字，接第三方数据源更省事，也更稳。

- [AQICN / WAQI](https://aqicn.org/api/)：有马尼拉等城市的实时 AQI，提供免费 API，需要申请 token。
- [OpenWeatherMap Air Pollution](https://openweathermap.org/api/air-pollution)：按经纬度取 PM2.5、PM10、O3 等，免费额度够个人项目用。
- [IQAir](https://www.iqair.com/air-quality-monitor-airvisual)：数据质量好，商用要付费。

这些源可以和你自己抓的 EMB 数据并存：默认显示第三方数据，用户点进"官方数据"再看 EMB 的站点表格。

数据版权归 DENR-EMB。Publish to Web 是公开链接，不等于授权转载。App 里注明数据来源，不要把这些数据当自有数据卖。

## 扩展：接新的报表

```bash
node tools/capture-recipe.mjs <resourceKey>
```

脚本会用无头 Chrome 打开报表，把首屏所有 `querydata` 请求体存到 `captured/` 下，每个文件都打印了它查的字段。挑出你要的，改个名字放进 `recipes/`，在 `src/collect.mjs` 的 `RECIPES` 里注册即可。

只支持 Chrome 系浏览器，需要本机装好 Chrome。Linux 或 macOS 上如果路径不同，改 `tools/capture-recipe.mjs` 顶部的 `CHROME_CANDIDATES`。

另外，EMB NCR 自己有官方 App（Android 包在 `https://coc.embncr.online/AQI/aqi.apk`，iOS 是 App Store 的 `id6476482860`）。写这份文档时 `coc.embncr.online` 从境内连不上，TLS 直接断连，所以没能分析它的接口。如果你能访问，抓一次它的请求也许能找到一个更正规的接口。
