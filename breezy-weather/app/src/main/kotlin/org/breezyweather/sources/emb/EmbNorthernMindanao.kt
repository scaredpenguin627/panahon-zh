package org.breezyweather.sources.emb

import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.doubleOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

/** The Region X public dashboard returns an array of key/value rows per station.
 * Only the explicitly identified PM AQI fields are copied into the local station panel. */
internal fun northernMindanaoStations(data: JsonArray): List<EmbStation> = data.mapNotNull { properties ->
    val values = properties.jsonArray.associate { item ->
        item.jsonObject.getValue("key").jsonPrimitive.content to item.jsonObject["value"]?.jsonPrimitive
    }
    val name = values["stationID"]?.contentOrNull ?: return@mapNotNull null
    val date = values["date"]?.contentOrNull?.takeIf { it.matches(Regex("\\d{4}-\\d{2}-\\d{2} \\d{2}:\\d{2}(:\\d{2})?")) }
        ?: return@mapNotNull null
    val readings = listOf("pm10AQI" to "PM10", "pm25AQI" to "PM2.5").mapNotNull { (field, label) ->
        values[field]?.doubleOrNull?.takeIf { it.isFinite() && it >= 0 }?.let { EmbMeasurement(label, it, "AQI") }
    }
    if (readings.isEmpty()) null else EmbStation(name, observed = date.take(16), readings = readings)
}
