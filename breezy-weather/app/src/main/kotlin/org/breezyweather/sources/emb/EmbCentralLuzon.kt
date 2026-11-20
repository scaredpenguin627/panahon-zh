package org.breezyweather.sources.emb

import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.doubleOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.TimeZone

/** Region III's official dashboard labels concentrations in µg/Ncm; these measurements
 * stay in the station panel with the published unit and are not treated as ambient µg/m³. */
internal fun centralLuzonReading(result: JsonObject, station: String, pollutant: String): EmbStation? {
    if (result["province"]?.jsonPrimitive?.content != station || result["pollutant"]?.jsonPrimitive?.content != pollutant) return null
    val row = result["row"] as? JsonObject ?: return null
    val stamp = result["time"]?.jsonPrimitive?.longOrNull?.takeIf { it > 0 } ?: return null
    val clock = SimpleDateFormat("yyyy-MM-dd HH:mm", Locale.US).apply { timeZone = TimeZone.getTimeZone("Asia/Manila") }
    val label = when (pollutant) {
        "pm10" -> "PM10"
        "pm25" -> "PM2.5"
        else -> return null
    }
    val readings = buildList {
        if (result["latestAqiVerified"]?.jsonPrimitive?.booleanOrNull == true) {
            row["AQI"]?.jsonPrimitive?.doubleOrNull?.takeIf { it.isFinite() && it >= 0 }?.let {
                add(EmbMeasurement(label, it, "AQI", row["Status"]?.jsonPrimitive?.contentOrNull))
            }
        }
        row["Concentration (µg/Ncm)"]?.jsonPrimitive?.doubleOrNull?.takeIf { it.isFinite() && it >= 0 }?.let {
            add(EmbMeasurement(label, it, "µg/Ncm"))
        }
    }
    if (readings.isEmpty()) return null
    return EmbStation(station, observed = clock.format(Date(stamp)), readings = readings)
}
