package org.panahon.zh.ui

import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.doubleOrNull
import kotlinx.serialization.json.intOrNull

fun JsonElement?.asObject(): JsonObject? = this as? JsonObject

fun JsonObject.string(name: String): String? = (this[name] as? JsonPrimitive)?.contentOrNull

fun JsonObject.number(name: String): Double? = (this[name] as? JsonPrimitive)?.doubleOrNull

fun JsonObject.integer(name: String): Int? = (this[name] as? JsonPrimitive)?.intOrNull

fun JsonObject.object(name: String): JsonObject? = this[name].asObject()

fun JsonObject.array(name: String): List<JsonElement> = this[name]
    ?.let { element -> (element as? kotlinx.serialization.json.JsonArray).orEmpty() }
    ?: emptyList()

