package org.panahon.zh.ui

import android.app.Application
import android.net.Uri
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import org.panahon.zh.BuildConfig
import org.panahon.zh.api.ApiEnvelope
import org.panahon.zh.api.LoadState
import org.panahon.zh.data.WeatherRepository

enum class AppTab(val label: String) {
    HOME("天气"), AQI("空气"), MAP("地图"), ALERTS("预警"), SETTINGS("设置")
}

data class PanahonUiState(
    val tab: AppTab = AppTab.HOME,
    val apiBaseUrl: String = BuildConfig.DEFAULT_API_BASE_URL,
    val cityName: String = "马尼拉",
    val latitude: Double = 14.5995,
    val longitude: Double = 120.9842,
    val locationQuery: String = "",
    val locationResults: List<kotlinx.serialization.json.JsonObject> = emptyList(),
    val locationSearchLoading: Boolean = false,
    val weather: LoadState = LoadState.Idle,
    val airQuality: LoadState = LoadState.Idle,
    val alerts: LoadState = LoadState.Idle,
    val map: LoadState = LoadState.Idle,
    val mapFrame: ByteArray? = null,
    val mapFrameIndex: Int = 0,
    val mapFrameLoading: Boolean = false,
    val mapFrameError: String? = null,
    val showLocationSearch: Boolean = false,
    val searchQuery: String = "",
    val searchResults: List<kotlinx.serialization.json.JsonElement> = emptyList(),
    val searchLoading: Boolean = false,
    val notice: String? = null,
)

class PanahonViewModel(application: Application) : AndroidViewModel(application) {
    private val repository = WeatherRepository(application)
    private val preferences = application.getSharedPreferences("settings", 0)
    private val json = Json { ignoreUnknownKeys = true; explicitNulls = false }
    private val state = MutableStateFlow(
        PanahonUiState(
            apiBaseUrl = preferences.getString("api_base_url", BuildConfig.DEFAULT_API_BASE_URL)
                ?: BuildConfig.DEFAULT_API_BASE_URL,
        ),
    )
    val uiState: StateFlow<PanahonUiState> = state.asStateFlow()

    init { refresh() }

    fun selectTab(tab: AppTab) { state.value = state.value.copy(tab = tab) }

    fun openSearch() { state.value = state.value.copy(showLocationSearch = true, searchQuery = "", searchResults = emptyList()) }

    fun closeSearch() { state.value = state.value.copy(showLocationSearch = false) }

    fun searchLocations(query: String) {
        state.value = state.value.copy(searchQuery = query)
        if (query.trim().length < 2) {
            state.value = state.value.copy(searchResults = emptyList())
            return
        }
        viewModelScope.launch {
            state.value = state.value.copy(searchLoading = true)
            repository.fetch("/api/v1/locations/search?q=${java.net.URLEncoder.encode(query.trim(), "UTF-8")}", "locations_${query.trim().lowercase()}", state.value.apiBaseUrl)
                .onSuccess { response -> state.value = state.value.copy(searchResults = response.data.asObject()?.array("items") ?: emptyList(), searchLoading = false) }
                .onFailure { state.value = state.value.copy(searchResults = emptyList(), searchLoading = false) }
        }
    }

    fun selectLocation(element: kotlinx.serialization.json.JsonElement) {
        val location = element.asObject() ?: return
        val name = location.string("name") ?: location.string("nameEn") ?: return
        val latitude = location.number("latitude") ?: return
        val longitude = location.number("longitude") ?: return
        state.value = state.value.copy(showLocationSearch = false, cityName = name, latitude = latitude, longitude = longitude)
        refresh()
    }

    fun setApiBaseUrl(value: String) {
        val normalized = value.trim().removeSuffix("/")
        if (!BuildConfig.ALLOW_CLEARTEXT_API && normalized.startsWith("http://")) {
            state.value = state.value.copy(notice = "正式版只允许 HTTPS 地址")
            return
        }
        state.value = state.value.copy(apiBaseUrl = normalized)
        preferences.edit().putString("api_base_url", normalized).apply()
    }

    fun setCity(name: String, latitude: Double, longitude: Double) {
        state.value = state.value.copy(cityName = name, latitude = latitude, longitude = longitude, locationQuery = name, locationResults = emptyList())
        refresh()
    }

    fun setLocationQuery(value: String) { state.value = state.value.copy(locationQuery = value) }

    fun searchLocations() {
        val query = state.value.locationQuery.trim()
        if (query.length < 2) return
        state.value = state.value.copy(locationSearchLoading = true)
        viewModelScope.launch {
            repository.fetch("/api/v1/locations/search?q=${Uri.encode(query)}", "locations_${query.lowercase()}", state.value.apiBaseUrl)
                .onSuccess { response ->
                    val items = response.data?.let { it as? kotlinx.serialization.json.JsonObject }
                        ?.get("items") as? kotlinx.serialization.json.JsonArray
                    state.value = state.value.copy(locationResults = items?.mapNotNull { it as? kotlinx.serialization.json.JsonObject } ?: emptyList(), locationSearchLoading = false)
                }
                .onFailure { state.value = state.value.copy(locationResults = emptyList(), locationSearchLoading = false, notice = "地点搜索失败") }
        }
    }

    fun refresh() {
        val current = state.value
        val query = "lat=${current.latitude}&lon=${current.longitude}"
        state.value = current.copy(
            weather = LoadState.Loading,
            airQuality = LoadState.Loading,
            alerts = LoadState.Loading,
            map = LoadState.Loading,
            mapFrameError = null,
            notice = null,
        )
        viewModelScope.launch { load("/api/v1/weather?$query", "weather", { response -> state.value = state.value.copy(weather = response) }) }
        viewModelScope.launch { load("/api/v1/air-quality?$query", "air", { response -> state.value = state.value.copy(airQuality = response) }) }
        viewModelScope.launch { load("/api/v1/alerts?$query", "alerts", { response -> state.value = state.value.copy(alerts = response) }) }
        viewModelScope.launch {
            load("/api/v1/maps/radar", "map") { response -> state.value = state.value.copy(map = response) }
            loadMapFrame(0)
        }
    }

    fun loadMapFrame(index: Int) {
        viewModelScope.launch {
            state.value = state.value.copy(mapFrameLoading = true, mapFrameError = null, mapFrameIndex = index)
            repository.fetchBytes("/api/v1/maps/radar/frames/$index", state.value.apiBaseUrl)
                .onSuccess { bytes -> state.value = state.value.copy(mapFrame = bytes, mapFrameLoading = false) }
                .onFailure { error -> state.value = state.value.copy(mapFrameLoading = false, mapFrameError = error.message ?: "地图帧不可用") }
        }
    }

    private suspend fun load(path: String, cacheKey: String, update: (LoadState) -> Unit) {
        val result = repository.fetch(path, cacheKey, state.value.apiBaseUrl)
        update(result.fold(
            onSuccess = { LoadState.Ready(it) },
            onFailure = { LoadState.Failed(message = it.message ?: "刷新失败") },
        ))
        result.getOrNull()?.takeIf { it.meta.stale }?.let {
            state.value = state.value.copy(notice = "刷新失败，当前显示最近一次成功缓存")
        }
        if (result.isFailure && state.value.notice == null) {
            state.value = state.value.copy(notice = "刷新失败，已显示可用的历史数据或空状态")
        }
    }
}


