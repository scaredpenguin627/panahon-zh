package org.breezyweather.sources.emb

import kotlinx.serialization.Serializable
import java.text.Normalizer
import java.text.SimpleDateFormat
import java.util.Locale
import java.util.TimeZone
import kotlin.math.cos
import kotlin.math.hypot

/** Public EMB reports differ in both their Power BI schema and the units they publish. */
@Serializable
internal data class EmbFeed(
    val key: String,
    val page: String,
    val kind: String = "station",
    val tenant: String = "f6f4a692-43b3-433b-92b2-65c4e6ccd920",
    val timeQuery: String? = null,
    val timeField: String? = null,
    val tables: List<EmbTable>,
    val sites: List<EmbSite>,
)

@Serializable
internal data class EmbTable(
    val recipe: String,
    val stationField: String,
    val fixedStation: String? = null,
    val addressField: String? = null,
    val timeField: String? = null,
    val dateOnly: Boolean = false,
    val columns: List<EmbColumn>,
)

@Serializable
internal data class EmbColumn(
    val field: String,
    val label: String,
    val unit: String,
    val ratingField: String? = null,
)

@Serializable
internal data class EmbSite(
    val station: String,
    val city: String,
    val displayName: String? = null,
    val latitude: Double,
    val longitude: Double,
    val aliases: List<String> = emptyList(),
    val maxDistanceKm: Double = 30.0,
) {
    // Match the municipality as well as its geographic vicinity. Metro Manila's regional
    // name alone cannot identify a station in the City of Manila.
    fun matches(names: Set<String>, lat: Double, lon: Double): Boolean {
        val distanceKm = 111.2 * hypot(lat - latitude, (lon - longitude) * cos(Math.toRadians(latitude)))
        return distanceKm <= maxDistanceKm && (aliases + city).any { normalizeCity(it) in names }
    }
}

internal fun normalizeCity(value: String): String =
    Normalizer.normalize(value.trim().lowercase(Locale.ROOT), Normalizer.Form.NFD)
        .replace(Regex("\\p{M}+"), "").removePrefix("city of ").removeSuffix(" city").trim()

@Serializable
internal data class EmbMeasurement(val name: String, val value: Double, val unit: String, val rating: String? = null)

@Serializable
internal data class EmbStation(
    val name: String,
    val address: String? = null,
    val observed: String? = null,
    val readings: List<EmbMeasurement>,
)

@Serializable
internal data class EmbSnapshot(
    val fetchedAt: Long,
    val reported: String? = null,
    val stations: List<EmbStation>,
)

internal data class EmbLocalReport(
    val feed: EmbFeed,
    val snapshot: EmbSnapshot?,
    val cachedOnError: Boolean = false,
    val unavailable: Boolean = false,
)

internal fun EmbSnapshot.forStations(stations: Set<String>): EmbSnapshot =
    copy(stations = this.stations.filter { it.name in stations })

internal fun reportAgeMillis(clock: String?, now: Long): Long? {
    if (clock == null) return null
    val format = SimpleDateFormat(if (clock.length == 10) "yyyy-MM-dd" else "yyyy-MM-dd HH:mm", Locale.US).apply {
        isLenient = false
        timeZone = TimeZone.getTimeZone("Asia/Manila")
    }
    val position = java.text.ParsePosition(0)
    val time = format.parse(clock, position) ?: return null
    return (now - time.time).takeIf { position.index == clock.length && it >= 0 }
}

internal fun isOldReport(clock: String?, now: Long): Boolean =
    reportAgeMillis(clock, now)?.let { it > 86_400_000L } == true

// The last successful readings are stored under the feed and matched station IDs, so switching
// locations cannot expose cached readings from another municipality.
internal fun selectEmbReport(
    feed: EmbFeed,
    fresh: EmbSnapshot?,
    previous: EmbSnapshot?,
    stations: Set<String>,
): EmbLocalReport {
    val matched = fresh?.forStations(stations)
    if (matched != null && matched.stations.isNotEmpty()) return EmbLocalReport(feed, matched)
    val fallback = previous?.forStations(stations)?.takeIf { it.stations.isNotEmpty() }
    return EmbLocalReport(feed, fallback ?: matched, cachedOnError = fallback != null, unavailable = fresh == null)
}


internal fun EmbLocalReport.hasRecentMonitoring(now: Long): Boolean {
    if (feed.kind != "station") return false
    return snapshot?.stations?.any {
        val clock = it.observed ?: snapshot.reported
        reportAgeMillis(clock, now)?.let { age -> age <= 86_400_000L } == true
    } == true
}
