package org.panahon.zh.api

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

@Serializable
data class ApiMeta(
    val source: List<String> = emptyList(),
    val quality: String = "unavailable",
    val fetched_at: String? = null,
    val cached: Boolean = false,
    val stale: Boolean = false,
    val request_id: String? = null,
)

@Serializable
data class ApiError(
    val code: String,
    val message: String,
)

@Serializable
data class ApiEnvelope(
    val data: JsonElement? = null,
    val meta: ApiMeta = ApiMeta(),
    val errors: List<ApiError> = emptyList(),
)

data class CachedResponse(
    val raw: String,
    val response: ApiEnvelope,
    val fetchedAt: Long,
)

sealed interface LoadState {
    data object Idle : LoadState
    data object Loading : LoadState
    data class Ready(val response: ApiEnvelope) : LoadState
    data class Failed(val response: ApiEnvelope? = null, val message: String) : LoadState
}

