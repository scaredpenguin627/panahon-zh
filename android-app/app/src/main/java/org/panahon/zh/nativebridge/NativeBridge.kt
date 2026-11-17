package org.panahon.zh.nativebridge

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

@Serializable
data class BackgroundProfile(
    val mode: String = "unknown",
    val isDay: Boolean = true,
    val primary: String = "#234E70",
    val secondary: String = "#101722",
    val intensity: Float = 0.25f,
    val seed: Long = 1L,
)

object NativeBridge {
    private val json = Json { ignoreUnknownKeys = true; explicitNulls = false }

    init {
        runCatching { System.loadLibrary("panahon_mobile_core") }
    }

    private external fun nativeBackgroundProfile(snapshot: String): String

    fun backgroundProfile(snapshot: String?): BackgroundProfile {
        if (snapshot.isNullOrBlank()) return BackgroundProfile()
        return runCatching {
            json.decodeFromString<BackgroundProfile>(nativeBackgroundProfile(snapshot))
        }.getOrElse { fallback(snapshot) }
    }

    private fun fallback(snapshot: String): BackgroundProfile {
        val lower = snapshot.lowercase()
        val night = lower.contains("night") || lower.contains("moon")
        val rain = lower.contains("rain") || lower.contains("storm")
        return when {
            rain -> BackgroundProfile("rain", !night, "#31566B", "#111A27", 0.65f, snapshot.hashCode().toLong())
            night -> BackgroundProfile("night", false, "#25305B", "#090F1C", 0.38f, snapshot.hashCode().toLong())
            else -> BackgroundProfile("clear", true, "#4B8DB5", "#17263B", 0.25f, snapshot.hashCode().toLong())
        }
    }
}

