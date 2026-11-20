package org.breezyweather.sources.panahon

import android.content.Context
import breezyweather.domain.location.model.Location
import breezyweather.domain.source.SourceContinent
import breezyweather.domain.source.SourceFeature
import org.breezyweather.R
import org.breezyweather.common.source.HttpSource
import org.breezyweather.common.source.NonFreeNetSource
import org.breezyweather.common.source.WeatherSource
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.TimeZone

abstract class PanahonSource : HttpSource(), WeatherSource, NonFreeNetSource {
    override val id = "panahon"
    override val name = "PAGASA PANaHON"
    override val continent = SourceContinent.ASIA
    override val privacyPolicyUrl = ""
    override val attributionLinks = mapOf(
        name to "https://panahon.gov.ph/", "PAGASA AWS" to "https://www.pagasa.dost.gov.ph/",
        "Open-Meteo" to "https://open-meteo.com/"
    )
    override val supportedFeatures = mapOf(
        SourceFeature.FORECAST to "$name • Open-Meteo (CC BY 4.0)",
        SourceFeature.CURRENT to "$name • PAGASA AWS • Open-Meteo (CC BY 4.0)",
        SourceFeature.ALERT to name
    )

    override fun isFeatureSupportedForLocation(location: Location, feature: SourceFeature) =
        location.countryCode.equals("PH", ignoreCase = true) && feature in supportedFeatures

    override fun getFeaturePriorityForLocation(location: Location, feature: SourceFeature) =
        if (isFeatureSupportedForLocation(location, feature)) WeatherSource.PRIORITY_HIGHEST else WeatherSource.PRIORITY_NONE

    override val testingLocations = listOf(
        Location(
            latitude = 14.5995, longitude = 120.9842, timeZone = TimeZone.getTimeZone("Asia/Manila"),
            city = "Manila", countryCode = "PH", forecastSource = id, currentSource = id, alertSource = id
        )
    )
}

internal fun windHumidityAttribution(time: Date, station: String? = null): String {
    val clock = SimpleDateFormat("yyyy-MM-dd HH:mm", Locale.US).apply {
        timeZone = TimeZone.getTimeZone("Asia/Manila")
    }
    val source = station?.let { "PAGASA AWS · $it" } ?: "Open-Meteo"
    return "$source · ${clock.format(time)} PHT"
}

/** Compose display text at render time so a language change also updates cached readings. */
internal fun Context.localizedWindHumiditySource(source: String): String =
    if (source.startsWith("Open-Meteo · ")) {
        getString(R.string.current_wind_humidity_model) + source.removePrefix("Open-Meteo")
    } else {
        source
    }
