package org.breezyweather.ui.search

import breezyweather.domain.location.model.Location
import java.util.Locale
import java.util.TimeZone

// City centers shared with the offline location list in services/weather-api.
private enum class PhilippineCity(
    val english: String,
    val chinese: String,
    val latitude: Double,
    val longitude: Double,
) {
    MANILA("Manila", "马尼拉", 14.5995, 120.9842),
    CEBU("Cebu City", "宿务", 10.3157, 123.8854),
    DAVAO("Davao City", "达沃", 7.1907, 125.4553),
}

internal fun findOfflinePhilippineCities(query: String, locale: Locale): List<Location> {
    val term = query.trim()
    val philippines = Locale.Builder().setRegion("PH").build().getDisplayCountry(locale)
    return PhilippineCity.entries
        .filter { it.english.startsWith(term, ignoreCase = true) || it.chinese.startsWith(term) }
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
