package org.panahon.zh.data

import android.content.Context
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import org.panahon.zh.api.ApiEnvelope
import org.panahon.zh.api.WeatherApi

class WeatherRepository(context: Context) {
    private val preferences = context.getSharedPreferences("weather_cache", Context.MODE_PRIVATE)
    private val json = Json { ignoreUnknownKeys = true; explicitNulls = false }
    private val api = WeatherApi(json)

    suspend fun fetch(path: String, cacheKey: String, baseUrl: String): Result<ApiEnvelope> {
        return runCatching {
            val response = api.get(baseUrl, path)
            preferences.edit()
                .putString(cacheKey, json.encodeToString(response))
                .putLong("${cacheKey}_saved_at", System.currentTimeMillis())
                .apply()
            response
        }.recoverCatching { error ->
            val cached = preferences.getString(cacheKey, null)
                ?: throw error
            val response = json.decodeFromString<ApiEnvelope>(cached)
            response.copy(meta = response.meta.copy(cached = true, stale = true))
        }
    }

    suspend fun fetchBytes(path: String, baseUrl: String): Result<ByteArray> =
        runCatching { api.getBytes(baseUrl, path) }

    fun clearCache() {
        preferences.edit().clear().apply()
    }

    fun cachedAt(cacheKey: String): Long? {
        val value = preferences.getLong("${cacheKey}_saved_at", 0L)
        return value.takeIf { it > 0L }
    }
}

