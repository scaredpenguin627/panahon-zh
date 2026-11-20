// Ported from the PANaHON 中文版 EMB collector. See assets/licenses/PANaHON-EMB-MIT.txt.
package org.breezyweather.sources.emb

import kotlinx.serialization.json.*
import java.text.ParsePosition
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.TimeZone

/** Read the flat table/card DSR shapes emitted by EMB's published visuals. R and Ø are
 * bitmaps for repeated and null fields; neither consumes a cell from the compact C array. */
internal fun decodeEmbTable(result: JsonObject): List<Map<String, String>> {
    val data = result.getValue("results").jsonArray.first().jsonObject.getValue("result").jsonObject
        .getValue("data").jsonObject
    val names = data.getValue("descriptor").jsonObject.getValue("Select").jsonArray.associate {
        it.jsonObject.getValue("Value").jsonPrimitive.content to it.jsonObject.getValue("Name").jsonPrimitive.content
    }
    val rows = mutableListOf<Map<String, String>>()
    val clock = SimpleDateFormat("yyyy-MM-dd HH:mm", Locale.US).apply { timeZone = TimeZone.getTimeZone("UTC") }
    for (ds in data.getValue("dsr").jsonObject.getValue("DS").jsonArray.map { it.jsonObject }) {
        val dict = ds["ValueDicts"]?.jsonObject
        for (phase in ds.getValue("PH").jsonArray) {
            for ((_, group) in phase.jsonObject.filterKeys { it.startsWith("DM") }) {
                var schema = emptyList<JsonObject>()
                var previous = emptyMap<String, String>()
                for (row in group.jsonArray.map { it.jsonObject }) {
                    schema = row["S"]?.jsonArray?.map { it.jsonObject } ?: schema
                    val cells = row["C"]?.jsonArray
                    val repeatMask = row["R"]?.jsonPrimitive?.longOrNull ?: 0L
                    val nullMask = row["Ø"]?.jsonPrimitive?.longOrNull ?: 0L
                    var cursor = 0
                    val record = mutableMapOf<String, String>()
                    for ((i, field) in schema.withIndex()) {
                        val id = field.getValue("N").jsonPrimitive.content
                        val name = names[id] ?: id
                        val bit = 1L shl i
                        if ((repeatMask and bit) != 0L) {
                            previous[name]?.let { record[name] = it }
                            continue
                        }
                        if ((nullMask and bit) != 0L) continue
                        val cell = if (cells != null) cells.getOrNull(cursor++) else row[id]
                        if (cell == null || cell is JsonNull) continue
                        val raw = cell.jsonPrimitive
                        val dictionary = field["DN"]?.jsonPrimitive?.content
                        val text = when {
                            dictionary != null -> raw.intOrNull?.let { dict?.get(dictionary)?.jsonArray?.getOrNull(it)?.jsonPrimitive?.contentOrNull }
                            field["T"]?.jsonPrimitive?.intOrNull == 7 && raw.longOrNull != null -> clock.format(Date(raw.long))
                            else -> raw.content.removeSuffix("L").takeIf { raw.content.matches(Regex("-?\\d+L")) } ?: raw.content
                        }
                        if (text != null) record[name] = text
                    }
                    rows.add(record)
                    previous = record
                }
            }
        }
    }
    return rows
}

private val microgramsPerNormalCubicMetre = Regex(
    """([+-]?(?:\d+(?:\.\d+)?|\.\d+))\s*(?:ug|µg|μg)/ncm""",
    RegexOption.IGNORE_CASE
)

internal fun embNumber(text: String?, unit: String): Double? {
    val raw = text?.trim() ?: return null
    val numeric = raw.toDoubleOrNull() ?: when (unit) {
        "µg/Ncm" -> microgramsPerNormalCubicMetre.matchEntire(raw)?.groupValues?.get(1)?.toDoubleOrNull()
        else -> null
    }
    return numeric?.takeIf { it.isFinite() && it >= 0 }
}

internal fun embClock(raw: String?, dateOnly: Boolean, annual: Boolean): String? {
    val text = raw?.trim() ?: return null
    if (annual) return text.takeIf { it.matches(Regex("(19|20)\\d{2}")) }
    val patterns = listOf("yyyy-MM-dd HH:mm", "yyyy-MM-dd HH:mm:ss", "yyyy-MM-dd", "M/d/yyyy")
    val zone = TimeZone.getTimeZone("UTC") // Parse only the report's displayed wall clock.
    for (pattern in patterns) {
        val format = SimpleDateFormat(pattern, Locale.US).apply { isLenient = false; timeZone = zone }
        val position = ParsePosition(0)
        val date = format.parse(text, position) ?: continue
        if (position.index != text.length) continue
        val output = if (dateOnly || ':' !in pattern) "yyyy-MM-dd" else "yyyy-MM-dd HH:mm"
        return SimpleDateFormat(output, Locale.US).apply { timeZone = zone }.format(date)
    }
    return null
}

internal fun assembleEmbSnapshot(feed: EmbFeed, data: Map<String, List<Map<String, String>>>, fetchedAt: Long): EmbSnapshot {
    val reported = embClock(feed.timeQuery?.let { data[it]?.firstOrNull()?.get(feed.timeField) }, false, false)
    // Historical reports may contain multiple rows per station. Pick one observation time
    // per station, preserving co-temporal measurements from separate pollutant visuals.
    val observations = mutableMapOf<Pair<String, String?>, EmbStation>()
    for (table in feed.tables) {
        for (row in data.getValue(table.recipe)) {
            val station = table.fixedStation ?: row[table.stationField]?.trim()?.takeIf { it.isNotEmpty() } ?: continue
            val rawTime = table.timeField?.let(row::get)
            val time = embClock(rawTime, table.dateOnly, feed.kind == "annual")
            if (time.isNullOrBlank() && reported.isNullOrBlank()) continue
            val readings = table.columns.mapNotNull { col ->
                embNumber(row[col.field], col.unit)?.let {
                    EmbMeasurement(col.label, it, col.unit, col.ratingField?.let(row::get))
                }
            }
            if (readings.isEmpty()) continue
            val key = station to time
            val prior = observations[key]
            observations[key] = EmbStation(
                name = station,
                address = table.addressField?.let(row::get) ?: prior?.address,
                observed = time,
                readings = (prior?.readings.orEmpty() + readings).distinctBy { it.name to it.unit }
            )
        }
    }
    val latest = observations.values.groupBy { it.name }.values.map { group -> group.maxBy { it.observed.orEmpty() } }
    return EmbSnapshot(fetchedAt, reported, latest.sortedBy { it.name })
}
