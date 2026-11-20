本项目是 Breezy Weather 的社区衍生版本。选择 PAGASA PANaHON 后，应用将所选地点的经纬度发送至 https://api.mobile.panahon.gov.ph/，用于查询该位置的预报和预警。选用 PANaHON 当前天气或预报时，应用还会向 Open-Meteo（api.open-meteo.com）发送地点坐标，以获取风、湿度和露点的补充模型数据；PAGASA 官网的自动站观测表在手机上按距离和观测时间匹配。字体随安装包提供，可离线使用。

首页显示 EMB 监测数据时，应用在手机上匹配地点与站点。它从菲律宾环境管理局的区域门户或微软 Power BI（app.powerbi.com 及报表使用的 analysis.windows.net 集群）读取公开报表。吕宋中部（Region III）的监测接口使用 embr3-onlinesystems.cloud，会收到所选站点的标识；北棉兰老（Region X）的监测接口使用 api.bpit-inc.com，应用读取报表后在本地筛选站点。最近一次成功取得的读数保存在本机。启用应用内更新检查后，应用向 GitHub 查询本仓库的正式版本。

Breezy Weather doesn’t collect any personal data.

Optionally, it can access your approximate or precise location to find weather data for your position. This data is shared with weather sources. If you don’t want to share your location, you can deny permissions and choose a city manually.

If you enable the “check for app updates” feature, the app will connect to GitHub every 24 hours at most to get the latest version of the Panahon ZH fork. The version you currently have installed will not be shared with GitHub (it will be compared on device). [GitHub Privacy Policy](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)

The icon packs feature (which allows you to customize weather icons) requires to list packages installed on your device. It will ignore all packages that don’t match one of these packs: Breezy Weather icon pack, Geometric Weather icon pack, Chronus icon pack. No data is collected, it is only processed at the moment it is needed.

Breezy Weather relies on third-party APIs to get various data such as your current location (if you decide to not use native GPS), location search or weather, please review their privacy policy (from Settings > Info icon > Privacy policy) and only use the ones you agree with.

Breezy Weather can optionally share your location and weather data with other apps. Please review their privacy policy before you decide to grant them permission.

This privacy policy may be updated any time, for example to clarify a use case or in case a feature gets added. In that case, it will be mentioned in the changelog. You can always access the latest version of the privacy policy from Settings > Info icon > Privacy policy.
