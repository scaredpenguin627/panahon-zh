package org.panahon.zh.ui

import android.graphics.BitmapFactory
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Cloud
import androidx.compose.material.icons.filled.DarkMode
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.LocationOn
import androidx.compose.material.icons.filled.Map
import androidx.compose.material.icons.filled.Notifications
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.WbSunny
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Divider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import org.panahon.zh.api.ApiEnvelope
import org.panahon.zh.api.LoadState

@Composable
fun PanahonApp(viewModel: PanahonViewModel = viewModel()) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val snapshot = (state.weather as? LoadState.Ready)?.response?.data?.toString()

    PanahonTheme {
        Box(Modifier.fillMaxSize()) {
            WeatherBackground(snapshot)
            Scaffold(
                containerColor = Color.Transparent,
                topBar = { AppTopBar(state, viewModel) },
                bottomBar = { AppNavigation(state.tab, viewModel::selectTab) },
            ) { padding ->
                Column(Modifier.fillMaxSize().padding(padding)) {
                    state.notice?.let { NoticeBanner(it) }
                    if (state.showLocationSearch) {
                        LocationSearchScreen(state, viewModel)
                    } else {
                        when (state.tab) {
                            AppTab.HOME -> HomeScreen(state, viewModel)
                            AppTab.AQI -> AqiScreen(state)
                            AppTab.MAP -> MapScreen(state, viewModel)
                            AppTab.ALERTS -> AlertsScreen(state, viewModel)
                            AppTab.SETTINGS -> SettingsScreen(state, viewModel)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun AppTopBar(state: PanahonUiState, viewModel: PanahonViewModel) {
    TopAppBar(
        title = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Icon(Icons.Default.LocationOn, null, tint = Color.White.copy(alpha = 0.84f))
                Spacer(Modifier.width(5.dp))
                Column(Modifier.clickable { viewModel.openSearch() }) {
                    Text(state.cityName, fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
                    Text("菲律宾天气", fontSize = 11.sp, color = Color.White.copy(alpha = 0.68f))
                }
            }
        },
        actions = {
            IconButton(onClick = { viewModel.openSearch() }) { Icon(Icons.Default.Search, "搜索城市") }
            IconButton(onClick = { viewModel.refresh() }) { Icon(Icons.Default.Refresh, "刷新") }
            IconButton(onClick = { viewModel.selectTab(AppTab.SETTINGS) }) { Icon(Icons.Default.Settings, "设置") }
        },
        colors = TopAppBarDefaults.topAppBarColors(containerColor = Color.Transparent),
    )
}

@Composable
private fun AppNavigation(selected: AppTab, onSelected: (AppTab) -> Unit) {
    NavigationBar(containerColor = Color(0xB5101722)) {
        AppTab.entries.forEach { tab ->
            NavigationBarItem(
                selected = tab == selected,
                onClick = { onSelected(tab) },
                icon = { Icon(tab.icon(), contentDescription = tab.label) },
                label = { Text(tab.label, fontSize = 11.sp) },
            )
        }
    }
}

private fun AppTab.icon() = when (this) {
    AppTab.HOME -> Icons.Default.Home
    AppTab.AQI -> Icons.Default.WbSunny
    AppTab.MAP -> Icons.Default.Map
    AppTab.ALERTS -> Icons.Default.Notifications
    AppTab.SETTINGS -> Icons.Default.Settings
}

@Composable
private fun NoticeBanner(message: String) {
    Surface(color = Color(0xCC7B4B2A), modifier = Modifier.fillMaxWidth()) {
        Text(message, modifier = Modifier.padding(horizontal = 16.dp, vertical = 9.dp), fontSize = 12.sp)
    }
}

@Composable
private fun HomeScreen(state: PanahonUiState, viewModel: PanahonViewModel) {
    val weather = (state.weather as? LoadState.Ready)?.response
    val current = weather?.data.asObject()?.object("current")
    val daily = weather?.data.asObject()?.array("daily") ?: emptyList()
    val hourly = weather?.data.asObject()?.array("hourly") ?: emptyList()
    val supplementary = weather?.data.asObject()?.object("supplementary")
    ScrollColumn {
        HeroWeather(state, current, state.weather)
        weather?.let { response ->
            response.data.asObject()?.let { data ->
                val alertCount = (state.alerts as? LoadState.Ready)?.response?.data.asObject()?.array("items")?.size ?: 0
                if (alertCount > 0) SmallStatusCard("有 $alertCount 条预警", "查看预警详情", Icons.Default.Notifications, onClick = { viewModel.selectTab(AppTab.ALERTS) })
                HourlyCard(hourly)
                DailyCard(daily)
                SummaryCard(
                    title = "当前指标",
                    rows = listOf(
                        "体感" to current?.string("feelsLikeC")?.let { "$it°C" },
                        "湿度" to current?.number("humidityPercent")?.let { "${it.toInt()}%" },
                        "风力" to current?.number("windKph")?.let { "${it.toInt()} km/h" },
                        "能见度" to supplementary?.number("visibilityKm")?.let { "%.1f km".format(it) },
                    ),
                )
                SunCard(supplementary)
                AirSummaryCard(state.airQuality, viewModel::selectTab)
                MapSummaryCard(state.map, viewModel::selectTab)
                SourceRow(response)
            }
        }
        if (weather == null && state.weather is LoadState.Failed) EmptyDataCard("天气服务暂时不可用", "重试") { viewModel.refresh() }
    }
}

@Composable
private fun HeroWeather(state: PanahonUiState, current: JsonObject?, loadState: LoadState) {
    GlassCard(Modifier.fillMaxWidth()) {
        Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.fillMaxWidth()) {
            Text(state.cityName, fontSize = 26.sp, fontWeight = FontWeight.Medium)
            val temperature = current?.number("temperatureC")
            Text(temperature?.let { "${it.toInt()}°" } ?: "暂无数据", fontSize = 76.sp, fontWeight = FontWeight.Light)
            Text(current?.string("condition") ?: qualityLabel(loadState), fontSize = 16.sp, color = Color.White.copy(alpha = 0.82f))
            val high = (loadState as? LoadState.Ready)?.response?.data.asObject()?.array("daily")?.firstOrNull()?.asObject()?.number("highC")
            val low = (loadState as? LoadState.Ready)?.response?.data.asObject()?.array("daily")?.firstOrNull()?.asObject()?.number("lowC")
            if (high != null || low != null) Text("最高 ${high?.toInt() ?: "--"}°  ·  最低 ${low?.toInt() ?: "--"}°", color = Color.White.copy(alpha = 0.72f))
            if (loadState is LoadState.Ready && loadState.response.meta.stale) Text("上次成功刷新：${loadState.response.meta.fetched_at ?: "未知"}", fontSize = 11.sp, color = Color.White.copy(alpha = 0.6f))
        }
    }
}

@Composable
private fun HourlyCard(items: List<JsonElement>) {
    SectionCard("逐小时") {
        Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            if (items.isEmpty()) Text("暂无逐小时数据", color = Color.White.copy(alpha = 0.7f))
            items.take(24).forEachIndexed { index, item ->
                val obj = item.asObject()
                Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.width(64.dp)) {
                    Text(obj?.string("time")?.takeLast(5) ?: "--", fontSize = 12.sp)
                    Icon(if (index % 3 == 0) Icons.Default.Cloud else Icons.Default.WbSunny, null, tint = Color(0xFFFFD56A), modifier = Modifier.size(25.dp).padding(top = 6.dp))
                    Text(obj?.number("temperatureC")?.let { "${it.toInt()}°" } ?: "--", fontSize = 15.sp, modifier = Modifier.padding(top = 8.dp))
                    Text(obj?.number("precipitationMm")?.let { "%.1f mm".format(it) } ?: "暂无", fontSize = 10.sp, color = Color.White.copy(alpha = 0.62f))
                }
            }
        }
    }
}

@Composable
private fun DailyCard(items: List<JsonElement>) {
    SectionCard("五日预报") {
        if (items.isEmpty()) Text("暂无逐日数据", color = Color.White.copy(alpha = 0.7f))
        items.take(5).forEachIndexed { index, item ->
            val obj = item.asObject()
            Row(Modifier.fillMaxWidth().padding(vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                Text(obj?.string("date") ?: if (index == 0) "今天" else "--", modifier = Modifier.weight(1f), maxLines = 1, overflow = TextOverflow.Ellipsis)
                Icon(if (index % 2 == 0) Icons.Default.WbSunny else Icons.Default.Cloud, null, tint = Color(0xFFFFD56A), modifier = Modifier.size(24.dp))
                Text("${obj?.number("lowC")?.toInt() ?: "--"}° / ${obj?.number("highC")?.toInt() ?: "--"}°", modifier = Modifier.width(96.dp), textAlign = androidx.compose.ui.text.style.TextAlign.End)
            }
            if (index < minOf(items.size, 5) - 1) Divider(color = Color.White.copy(alpha = 0.12f))
        }
    }
}

@Composable
private fun SummaryCard(title: String, rows: List<Pair<String, String?>>) {
    SectionCard(title) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            rows.forEach { (label, value) ->
                Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.weight(1f)) {
                    Text(value ?: "暂无数据", fontSize = 14.sp)
                    Text(label, fontSize = 11.sp, color = Color.White.copy(alpha = 0.62f), modifier = Modifier.padding(top = 4.dp))
                }
            }
        }
    }
}

@Composable
private fun SunCard(supplementary: JsonObject?) {
    SectionCard("日出日落") {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceAround) {
            Metric("日出", supplementary?.string("sunrise") ?: "暂无数据")
            Metric("日落", supplementary?.string("sunset") ?: "暂无数据")
            Metric("紫外线", supplementary?.number("uvIndex")?.toString() ?: "暂无数据")
        }
    }
}

@Composable
private fun Metric(label: String, value: String) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) { Text(value, fontSize = 14.sp); Text(label, fontSize = 11.sp, color = Color.White.copy(alpha = 0.62f)) }
}

@Composable
private fun AirSummaryCard(state: LoadState, onOpen: (AppTab) -> Unit) {
    val data = (state as? LoadState.Ready)?.response?.data.asObject()
    val summary = data?.object("summary")
    SmallStatusCard("空气质量", summary?.string("category") ?: qualityLabel(state), Icons.Default.WbSunny, onClick = { onOpen(AppTab.AQI) }, trailing = summary?.number("maxAqi")?.let { "AQI ${it.toInt()}" })
}

@Composable
private fun MapSummaryCard(state: LoadState, onOpen: (AppTab) -> Unit) {
    val data = (state as? LoadState.Ready)?.response?.data.asObject()
    val frameCount = data?.array("frames")?.size ?: 0
    SmallStatusCard("降水雷达", if (frameCount > 0) "$frameCount 个时间帧" else qualityLabel(state), Icons.Default.Map, onClick = { onOpen(AppTab.MAP) })
}

@Composable
private fun SmallStatusCard(title: String, subtitle: String, icon: androidx.compose.ui.graphics.vector.ImageVector, onClick: () -> Unit, trailing: String? = null) {
    GlassCard(Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.fillMaxWidth()) {
            Icon(icon, null, tint = Color(0xFFFFD56A), modifier = Modifier.size(25.dp))
            Column(Modifier.weight(1f).padding(start = 12.dp)) { Text(title, fontWeight = FontWeight.SemiBold); Text(subtitle, fontSize = 12.sp, color = Color.White.copy(alpha = 0.68f)) }
            trailing?.let { Text(it, fontWeight = FontWeight.SemiBold) }
        }
    }
}

@Composable
private fun SourceRow(response: ApiEnvelope) {
    Text("来源：${response.meta.source.joinToString("、").ifBlank { "暂无" }} · ${response.meta.quality}", fontSize = 11.sp, color = Color.White.copy(alpha = 0.56f), modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp))
}

@Composable
private fun AqiScreen(state: PanahonUiState) {
    val response = (state.airQuality as? LoadState.Ready)?.response
    val data = response?.data.asObject()
    ScrollColumn {
        PageTitle("空气质量", "显示真实来源和污染物可用性")
        GlassCard(Modifier.fillMaxWidth()) {
            val summary = data?.object("summary")
            Text(summary?.string("category") ?: qualityLabel(state.airQuality), fontSize = 31.sp, fontWeight = FontWeight.SemiBold)
            Text(summary?.number("maxAqi")?.let { "最高 AQI ${it.toInt()}" } ?: "暂无 AQI", color = Color.White.copy(alpha = 0.72f))
            Spacer(Modifier.height(12.dp))
            Text("来源：${response?.meta?.source?.joinToString("、") ?: "暂无"}", fontSize = 12.sp, color = Color.White.copy(alpha = 0.65f))
        }
        SectionCard("监测站") {
            val stations = data?.array("stations") ?: emptyList()
            if (stations.isEmpty()) Text("暂无监测站数据", color = Color.White.copy(alpha = 0.7f))
            stations.take(16).forEach { element ->
                val station = element.asObject()
                Row(Modifier.fillMaxWidth().padding(vertical = 8.dp), horizontalArrangement = Arrangement.SpaceBetween) {
                    Column(Modifier.weight(1f)) { Text(station?.string("nameZh") ?: station?.string("name") ?: "未知站点"); Text(station?.string("pollutant") ?: "污染物暂无数据", fontSize = 11.sp, color = Color.White.copy(alpha = 0.6f)) }
                    Text(station?.number("aqi")?.toInt()?.toString() ?: "暂无", fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
                }
            }
        }
    }
}

@Composable
private fun MapScreen(state: PanahonUiState, viewModel: PanahonViewModel) {
    val response = (state.map as? LoadState.Ready)?.response
    val data = response?.data.asObject()
    ScrollColumn {
        PageTitle("天气地图", "地图帧由 PAGASA 服务提供")
        GlassCard(Modifier.fillMaxWidth().height(260.dp)) {
            val bitmap = state.mapFrame?.let { BitmapFactory.decodeByteArray(it, 0, it.size)?.asImageBitmap() }
            if (bitmap != null) {
                Image(bitmap, "降水雷达", Modifier.fillMaxSize().clip(RoundedCornerShape(14.dp)), contentScale = ContentScale.Crop)
            } else {
                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Icon(Icons.Default.Map, null, tint = Color(0xFFB9DDF2), modifier = Modifier.size(72.dp))
                    Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.padding(top = 110.dp)) {
                        Text(if ((data?.array("frames")?.size ?: 0) > 0) "雷达帧加载中" else "暂无地图帧")
                        Text(state.mapFrameError ?: "暂无可显示的图像", fontSize = 11.sp, color = Color.White.copy(alpha = 0.62f))
                    }
                }
            }
        }
        val frameCount = data?.array("frames")?.size ?: 0
        if (frameCount > 1) {
            SectionCard("时间轴") {
                var selectedFrame by remember(state.mapFrameIndex) { mutableStateOf(state.mapFrameIndex.toFloat()) }
                Slider(value = selectedFrame, onValueChange = { selectedFrame = it }, onValueChangeFinished = { viewModel.loadMapFrame(selectedFrame.toInt()) }, valueRange = 0f..(frameCount - 1).toFloat(), steps = (frameCount - 2).coerceAtLeast(0))
                Text("第 ${state.mapFrameIndex + 1} / $frameCount 帧", fontSize = 11.sp, color = Color.White.copy(alpha = 0.64f))
            }
        }
        SectionCard("图层") { listOf("降水雷达", "卫星", "闪电", "台风", "灾害").forEach { layer -> Row(Modifier.fillMaxWidth().padding(vertical = 9.dp), horizontalArrangement = Arrangement.SpaceBetween) { Text(layer); Text(if (layer == "降水雷达" && data?.array("frames")?.isNotEmpty() == true) "可用" else "暂无数据", color = Color.White.copy(alpha = 0.65f), fontSize = 12.sp) } } }
        response?.let { SourceRow(it) }
    }
}

@Composable
private fun AlertsScreen(state: PanahonUiState, viewModel: PanahonViewModel) {
    val response = (state.alerts as? LoadState.Ready)?.response
    val items = response?.data.asObject()?.array("items") ?: emptyList()
    ScrollColumn {
        PageTitle("预警", "保留官方原文和有效期")
        if (items.isEmpty()) EmptyDataCard(if (state.alerts is LoadState.Failed) "预警服务暂时不可用" else "当前没有可显示的预警", "刷新") { viewModel.refresh() }
        items.forEach { item ->
            val alert = item.asObject()
            SectionCard(alert?.string("title") ?: alert?.string("kind") ?: "官方预警") {
                Text(alert?.string("message") ?: "暂无正文", fontSize = 14.sp)
                Spacer(Modifier.height(8.dp))
                Text("${alert?.string("issuedAt") ?: "发布时间暂无"} · ${alert?.string("validUntil") ?: "有效期暂无"}", fontSize = 11.sp, color = Color.White.copy(alpha = 0.62f))
            }
        }
    }
}

@Composable
private fun SettingsScreen(state: PanahonUiState, viewModel: PanahonViewModel) {
    var endpoint by remember(state.apiBaseUrl) { mutableStateOf(state.apiBaseUrl) }
    ScrollColumn {
        PageTitle("设置", "服务地址和缓存状态")
        SectionCard("后端地址") {
            TextField(value = endpoint, onValueChange = { endpoint = it }, modifier = Modifier.fillMaxWidth(), singleLine = true, label = { Text("API 地址") })
            Spacer(Modifier.height(10.dp))
            Button(onClick = { viewModel.setApiBaseUrl(endpoint); viewModel.refresh() }) { Text("保存并刷新") }
        }
        SectionCard("地点") {
            TextField(value = state.locationQuery, onValueChange = viewModel::setLocationQuery, modifier = Modifier.fillMaxWidth(), singleLine = true, label = { Text("搜索城市") })
            Spacer(Modifier.height(8.dp))
            Button(onClick = viewModel::searchLocations, enabled = state.locationQuery.trim().length >= 2 && !state.locationSearchLoading) {
                if (state.locationSearchLoading) CircularProgressIndicator(Modifier.size(18.dp), strokeWidth = 2.dp) else Icon(Icons.Default.Search, "搜索")
                Spacer(Modifier.width(6.dp))
                Text("搜索")
            }
            state.locationResults.forEach { item ->
                val name = item.string("name") ?: item.string("nameEn") ?: "未知地点"
                val lat = item.number("latitude")
                val lon = item.number("longitude")
                if (lat != null && lon != null) {
                    TextButton(onClick = { viewModel.setCity(name, lat, lon) }, modifier = Modifier.fillMaxWidth()) { Text("$name · ${item.string("region") ?: "菲律宾"}", maxLines = 1, overflow = TextOverflow.Ellipsis) }
                }
            }
        }
        SectionCard("数据行为") {
            Text("网络失败时只显示最近一次成功缓存。首次没有缓存时显示暂无数据，不会填入示例天气。", fontSize = 13.sp, color = Color.White.copy(alpha = 0.76f))
        }
        SectionCard("关于") {
            Text("菲律宾天气", fontWeight = FontWeight.SemiBold)
            Text("Compose + Rust JNI · 原创动态背景", fontSize = 12.sp, color = Color.White.copy(alpha = 0.66f), modifier = Modifier.padding(top = 5.dp))
        }
    }
}

@Composable
private fun LocationSearchScreen(state: PanahonUiState, viewModel: PanahonViewModel) {
    ScrollColumn {
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            TextButton(onClick = { viewModel.closeSearch() }) { Text("返回") }
            Text("搜索地点", fontSize = 24.sp, fontWeight = FontWeight.SemiBold)
        }
        SectionCard("搜索城市") {
            TextField(
                value = state.searchQuery,
                onValueChange = viewModel::searchLocations,
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
                label = { Text("输入至少两个字符") },
                leadingIcon = { Icon(Icons.Default.Search, null) },
            )
            if (state.searchLoading) CircularProgressIndicator(modifier = Modifier.padding(top = 12.dp).size(22.dp), strokeWidth = 2.dp)
        }
        if (state.searchResults.isEmpty() && state.searchQuery.length >= 2 && !state.searchLoading) {
            EmptyDataCard("没有找到可用地点", "重新输入") { viewModel.searchLocations("") }
        }
        state.searchResults.forEach { result ->
            val item = result.asObject()
            SectionCard(item?.string("name") ?: item?.string("nameEn") ?: "未知地点") {
                Text(item?.string("region") ?: "地区暂无", fontSize = 12.sp, color = Color.White.copy(alpha = 0.65f))
                TextButton(onClick = { viewModel.selectLocation(result) }) { Text("选择此地点") }
            }
        }
    }
}

@Composable
private fun PageTitle(title: String, subtitle: String) {
    Column(Modifier.padding(horizontal = 16.dp, vertical = 10.dp)) { Text(title, fontSize = 26.sp, fontWeight = FontWeight.SemiBold); Text(subtitle, fontSize = 12.sp, color = Color.White.copy(alpha = 0.65f), modifier = Modifier.padding(top = 3.dp)) }
}

@Composable
private fun ScrollColumn(content: @Composable androidx.compose.foundation.layout.ColumnScope.() -> Unit) {
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(horizontal = 12.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(10.dp), content = content)
}

@Composable
private fun SectionCard(title: String, content: @Composable androidx.compose.foundation.layout.ColumnScope.() -> Unit) {
    GlassCard(Modifier.fillMaxWidth()) { Text(title, fontSize = 13.sp, color = Color.White.copy(alpha = 0.72f), modifier = Modifier.padding(bottom = 10.dp)); content() }
}

@Composable
private fun GlassCard(modifier: Modifier = Modifier, content: @Composable androidx.compose.foundation.layout.ColumnScope.() -> Unit) {
    Card(modifier, shape = RoundedCornerShape(18.dp), colors = CardDefaults.cardColors(containerColor = Color.White.copy(alpha = 0.13f)), content = { Column(Modifier.padding(16.dp), content = content) })
}

@Composable
private fun EmptyDataCard(title: String, action: String, onClick: () -> Unit) {
    GlassCard(Modifier.fillMaxWidth()) { Text(title); TextButton(onClick = onClick) { Text(action) } }
}

private fun qualityLabel(state: LoadState): String = when (state) {
    LoadState.Idle -> "等待刷新"
    LoadState.Loading -> "正在加载"
    is LoadState.Ready -> if (state.response.meta.stale) "历史缓存" else "数据可用"
    is LoadState.Failed -> "暂无数据"
}


