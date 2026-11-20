package org.breezyweather.sources.panahon

import breezyweather.domain.location.model.Location
import breezyweather.domain.source.SourceContinent
import breezyweather.domain.source.SourceFeature
import org.breezyweather.common.source.HttpSource
import org.breezyweather.common.source.NonFreeNetSource
import org.breezyweather.common.source.WeatherSource
import java.util.TimeZone

abstract class PanahonSource : HttpSource(), WeatherSource, NonFreeNetSource {
    override val id = "panahon"
    override val name = "PAGASA PANaHON"
    override val continent = SourceContinent.ASIA
    override val privacyPolicyUrl = ""
    override val attributionLinks = mapOf(name to "https://panahon.gov.ph/")
    override val supportedFeatures = listOf(SourceFeature.FORECAST, SourceFeature.CURRENT, SourceFeature.ALERT)
        .associateWith { name }

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
