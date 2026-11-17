package org.panahon.zh.api

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import java.io.IOException
import java.net.HttpURLConnection
import java.net.URL

class WeatherApi(
    private val json: Json = Json { ignoreUnknownKeys = true; explicitNulls = false },
) {
    suspend fun get(baseUrl: String, path: String): ApiEnvelope = withContext(Dispatchers.IO) {
        val normalizedBase = baseUrl.trim().removeSuffix("/")
        val connection = (URL("$normalizedBase$path").openConnection() as HttpURLConnection).apply {
            requestMethod = "GET"
            connectTimeout = 12_000
            readTimeout = 18_000
            setRequestProperty("Accept", "application/json")
            setRequestProperty("User-Agent", "panahon-zh-android/0.1")
        }
        try {
            val status = connection.responseCode
            val stream = if (status in 200..299) connection.inputStream else connection.errorStream
            val body = stream?.bufferedReader()?.use { it.readText() }.orEmpty()
            if (body.isBlank()) throw IOException("empty response ($status)")
            val envelope = json.decodeFromString<ApiEnvelope>(body)
            if (status !in 200..299) throw IOException(envelope.errors.firstOrNull()?.message ?: "HTTP $status")
            envelope
        } finally {
            connection.disconnect()
        }
    }

    suspend fun getBytes(baseUrl: String, path: String): ByteArray = withContext(Dispatchers.IO) {
        val normalizedBase = baseUrl.trim().removeSuffix("/")
        val connection = (URL("$normalizedBase$path").openConnection() as HttpURLConnection).apply {
            requestMethod = "GET"
            connectTimeout = 12_000
            readTimeout = 18_000
            setRequestProperty("Accept", "image/png,image/webp,image/*")
            setRequestProperty("User-Agent", "panahon-zh-android/0.1")
        }
        try {
            val status = connection.responseCode
            if (status !in 200..299) throw IOException("HTTP $status")
            connection.inputStream.use { it.readBytes() }
        } finally {
            connection.disconnect()
        }
    }
}

