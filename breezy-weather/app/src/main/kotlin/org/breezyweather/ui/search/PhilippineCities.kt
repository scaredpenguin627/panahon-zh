package org.breezyweather.ui.search

import breezyweather.domain.location.model.Location
import java.util.Locale
import java.util.TimeZone

private enum class PhilippineCity(
    val english: String,
    val chinese: String,
    val latitude: Double,
    val longitude: Double,
    vararg val aliases: String,
) {
    MANILA("Manila", "马尼拉", 14.5995, 120.9842),
    CEBU("Cebu City", "宿务", 10.3157, 123.8854),
    DAVAO("Davao City", "达沃", 7.1907, 125.4553),
    BORACAY("Boracay Island", "长滩岛", 11.98013, 121.91932, "長灘島"),
}

internal fun findOfflinePhilippineCities(query: String, locale: Locale): List<Location> {
    val term = query.trim()
    val philippines = Locale.Builder().setRegion("PH").build().getDisplayCountry(locale)
    return PhilippineCity.entries
        .filter { place ->
            if (term.isEmpty()) {
                place != PhilippineCity.BORACAY
            } else {
                place.english.startsWith(term, ignoreCase = true) || place.chinese.startsWith(term) ||
                    place.aliases.any { it.startsWith(term, ignoreCase = true) }
            }
        }
        .map {
            Location(
                latitude = it.latitude,
                longitude = it.longitude,
                timeZone = TimeZone.getTimeZone("Asia/Manila"),
                countryCode = "PH",
                country = philippines,
                city = if (locale.language == "zh") it.chinese else it.english,
                cityId = "panahon-" + it.name.lowercase(Locale.ROOT)
            )
        }
}

internal fun mergeSearchLocations(offline: List<Location>, online: List<Location>): List<Location> =
    (offline + online).distinctBy { it.latitude to it.longitude }
