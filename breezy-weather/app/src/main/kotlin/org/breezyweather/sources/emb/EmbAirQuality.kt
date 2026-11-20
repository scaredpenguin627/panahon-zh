package org.breezyweather.sources.emb

import breezyweather.domain.weather.model.AirQuality
import org.breezyweather.domain.weather.index.PollutantIndex
import org.breezyweather.unit.pollutant.PollutantConcentration
import org.breezyweather.unit.pollutant.PollutantConcentrationUnit
import org.breezyweather.unit.pollutant.toPollutantConcentration
import java.util.Locale

private val EMB_POLLUTANTS = listOf(
    "PM2.5" to PollutantIndex.PM25,
    "PM25" to PollutantIndex.PM25,
    "PM10" to PollutantIndex.PM10,
    "NO2" to PollutantIndex.NO2,
    "SO2" to PollutantIndex.SO2,
    "O3" to PollutantIndex.O3,
    "CO" to PollutantIndex.CO
)

/**
 * Picks the newest station reading that still carries a pollutant value and hands it to the
 * shared [AirQuality] model, so the main card, the details screen and the widgets all use the
 * same index formula as the rest of the app.
 */
internal fun List<EmbLocalReport>.toAirQuality(): AirQuality? {
    val station = asSequence()
        .flatMap { report -> report.snapshot?.stations.orEmpty().asSequence() }
        .filter { site -> site.readings.any { it.concentration() != null } }
        .maxByOrNull { it.observed.orEmpty() }
        ?: return null

    fun reading(pollutant: PollutantIndex, vararg names: String): PollutantConcentration? =
        names.firstNotNullOfOrNull { name ->
            station.readings
                .firstOrNull { it.name.equals(name, ignoreCase = true) }
                ?.concentration(pollutant)
        }

    return AirQuality(
        pM25 = reading(PollutantIndex.PM25, "PM2.5", "PM25"),
        pM10 = reading(PollutantIndex.PM10, "PM10"),
        nO2 = reading(PollutantIndex.NO2, "NO2"),
        sO2 = reading(PollutantIndex.SO2, "SO2"),
        o3 = reading(PollutantIndex.O3, "O3"),
        cO = reading(PollutantIndex.CO, "CO")
    ).takeIf { it.isValid }
}

private fun EmbMeasurement.concentration(): PollutantConcentration? = EMB_POLLUTANTS
    .firstOrNull { (name, _) -> name.equals(this.name, ignoreCase = true) }
    ?.let { (_, pollutant) -> concentration(pollutant) }

private fun EmbMeasurement.concentration(pollutant: PollutantIndex): PollutantConcentration? {
    val normalized = unit.lowercase(Locale.ROOT)
        .replace("μ", "u")
        .replace("µ", "u")
        .replace("³", "3")
        .replace(" ", "")
    if (normalized == "aqi") return indexToConcentration(pollutant, value)
    if (!normalized.endsWith("g/m3") && !normalized.endsWith("g/ncm")) return null
    return value.toPollutantConcentration(
        if (normalized.startsWith("mg")) {
            PollutantConcentrationUnit.MILLIGRAM_PER_CUBIC_METER
        } else {
            PollutantConcentrationUnit.MICROGRAM_PER_CUBIC_METER
        }
    )
}

/**
 * DENR-EMB publishes a sub-index, not a concentration. Invert the index formula of the shared
 * model, so the card shows the same number the station reports instead of a second opinion.
 */
private fun indexToConcentration(pollutant: PollutantIndex, index: Double): PollutantConcentration {
    val levels = PollutantIndex.aqiThresholds
    val breakpoints = pollutant.thresholds
    val last = levels.lastIndex
    val level = levels.indexOfLast { index >= it }.coerceIn(0, last)
    val value = if (level == last) {
        index * breakpoints[last] / levels[last]
    } else {
        breakpoints[level] + (index - levels[level]) *
            (breakpoints[level + 1] - breakpoints[level]) / (levels[level + 1] - levels[level])
    }
    return value.toPollutantConcentration(
        if (pollutant == PollutantIndex.CO) {
            PollutantConcentrationUnit.MILLIGRAM_PER_CUBIC_METER
        } else {
            PollutantConcentrationUnit.MICROGRAM_PER_CUBIC_METER
        }
    )
}
