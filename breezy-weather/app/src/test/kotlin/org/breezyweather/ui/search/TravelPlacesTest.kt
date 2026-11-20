package org.breezyweather.ui.search

import java.io.File
import java.util.Locale
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

class TravelPlacesTest {
    private val directory = listOf(File("src/main/assets/tourism"), File("app/src/main/assets/tourism"))
        .first { it.isDirectory }
    private val regions = listOf("luzon", "vismin").map { File(directory, "$it.tsv") }
    private val catalog = originalPhilippinePlaces + regions.flatMap { readTourismPlaces(it.reader(Charsets.UTF_8)) }

    @Test
    fun everySpellingSelectsTheCataloguedCoordinatesOffline() {
        val ids = originalPhilippinePlaces.map { it.location(Locale.CHINA, "菲律宾").cityId }.toMutableSet()
        for (region in regions) {
            val lines = region.readLines(Charsets.UTF_8).filter { it.isNotBlank() }
            assertEquals("id\tzh\ten\tlat\tlon\taliases\tsource", lines.first().removePrefix("\uFEFF"))
            assertTrue(lines.size > 1)
            for (line in lines.drop(1)) {
                val cells = line.split('\t')
                val id = "panahon-${cells[0]}"
                assertTrue(ids.add(id), "Duplicate ID: $id")
                assertTrue(cells[6].startsWith("https://"), id)
                for (name in (cells.subList(1, 3) + cells[5].split('|')).filter { it.isNotBlank() }) {
                    val hits = findOfflinePhilippineCities(name, Locale.CHINA, catalog)
                    val selected = hits.single { it.cityId == id }
                    assertEquals(cells[3].toDouble(), selected.latitude, id)
                    assertEquals(cells[4].toDouble(), selected.longitude, id)
                    assertEquals("PH", selected.countryCode)
                    assertEquals("Asia/Manila", selected.timeZone.id)
                }
            }
        }
        assertEquals(ids.size, findOfflinePhilippineCities("", Locale.ENGLISH, catalog).size)
    }

    @Test
    fun punctuationAndAccentsDoNotHideDestinationsOrShadowExactMatches() {
        val places = listOf(
            PhilippinePlace("area", "测试群岛", "Test Islands", 10.1, 122.0, listOf("Lagoon area")),
            PhilippinePlace("lagoon", "测试湖", "Lagóon", 10.0, 122.1, listOf("Lagoon-A"))
        )
        assertEquals("panahon-lagoon", findOfflinePhilippineCities("LAGOON", Locale.CHINA, places).first().cityId)
        assertEquals("panahon-lagoon", findOfflinePhilippineCities("lagoon a", Locale.ENGLISH, places).first().cityId)
        assertTrue(findOfflinePhilippineCities(",,,", Locale.CHINA, places).isEmpty())
    }
}
