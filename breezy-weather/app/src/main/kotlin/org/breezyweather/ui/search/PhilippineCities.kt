package org.breezyweather.ui.search

import breezyweather.domain.location.model.Location
import java.io.Reader
import java.text.Normalizer
import java.util.Locale
import java.util.TimeZone

private val accentsAndSeparators = Regex("[\\p{M}\\p{P}\\p{Z}\\s]+")
private fun searchKey(value: String): String =
    Normalizer.normalize(value, Normalizer.Form.NFD).lowercase(Locale.ROOT).replace(accentsAndSeparators, "")

internal class PhilippinePlace(
    private val id: String,
    private val chinese: String,
    private val english: String,
    private val latitude: Double,
    private val longitude: Double,
    aliases: List<String> = emptyList(),
) {
    private val names = (listOf(chinese, english) + aliases).map(::searchKey).filter(String::isNotEmpty).distinct()
    val cityId = "panahon-$id"

    fun otherName(locale: Locale): String = if (locale.language == "zh") english else chinese

    fun matchRank(term: String): Int? = when {
        term.isEmpty() -> 0
        term in names -> 0
        names.any { it.startsWith(term) } -> 1
        term.length >= 2 && names.any { term in it } -> 2
        else -> null
    }

    fun location(locale: Locale, country: String) = Location(
        latitude = latitude,
        longitude = longitude,
        timeZone = TimeZone.getTimeZone("Asia/Manila"),
        countryCode = "PH",
        country = country,
        city = if (locale.language == "zh") chinese else english,
        cityId = cityId
    )
}

// These IDs and coordinates identify saved locations from 1.0.0.
internal val originalPhilippinePlaces = listOf(
    PhilippinePlace("manila", "马尼拉", "Manila", 14.5995, 120.9842, listOf("馬尼拉")),
    PhilippinePlace("cebu", "宿务", "Cebu City", 10.3157, 123.8854, listOf("宿霧", "宿雾", "宿務")),
    PhilippinePlace("davao", "达沃", "Davao City", 7.1907, 125.4553, listOf("達沃")),
    PhilippinePlace("boracay", "长滩岛", "Boracay Island", 11.98013, 121.91932, listOf("長灘島"))
)

internal fun readTourismPlaces(reader: Reader): List<PhilippinePlace> = reader.buffered().useLines { lines ->
    lines.drop(1).filter { it.isNotBlank() }.map { line ->
        val fields = line.split('\t')
        require(fields.size == 7) { "Travel place requires seven columns" }
        PhilippinePlace(fields[0], fields[1], fields[2], fields[3].toDouble(), fields[4].toDouble(), fields[5].split('|'))
    }.toList()
}

internal fun findOfflinePhilippineCities(
    query: String,
    locale: Locale,
    places: List<PhilippinePlace> = originalPhilippinePlaces,
): List<Location> {
    val term = searchKey(query)
    if (query.isNotBlank() && term.isEmpty()) return emptyList()
    val country = Locale.Builder().setRegion("PH").build().getDisplayCountry(locale)
    return places.mapNotNull { place -> place.matchRank(term)?.let { rank -> rank to place } }
        .sortedBy { it.first }
        .map { it.second.location(locale, country) }
}

internal fun mergeSearchLocations(offline: List<Location>, online: List<Location>): List<Location> =
    (offline + online).distinctBy { it.latitude to it.longitude }
