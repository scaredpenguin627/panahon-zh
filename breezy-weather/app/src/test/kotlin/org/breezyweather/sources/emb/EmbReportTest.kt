package org.breezyweather.sources.emb

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.Test
import java.io.File
import java.time.Instant

class EmbReportTest {
    private val assets = listOf(File("src/main/assets/emb"), File("app/src/main/assets/emb")).first { it.isDirectory }
    private val ncr = Json.decodeFromString<EmbFeed>(File(assets, "ncr.json").readText())

    private fun snapshot(feed: EmbFeed, sample: String): EmbSnapshot {
        val file = checkNotNull(javaClass.getResource("/emb/$sample-sample.json")) { "Missing $sample fixture" }
        val replies = Json.parseToJsonElement(file.readText()).jsonObject
        return assembleEmbSnapshot(feed, replies.mapValues { decodeEmbTable(it.value.jsonObject) }, 10L)
    }

    @Test
    fun allPublishedReportsHaveAnObservedSiteAndARealQueryFixture() {
        for (file in assets.listFiles()!!.filter { it.name == "ncr.json" || it.name.endsWith(".report.json") }) {
            val feed = Json.decodeFromString<EmbFeed>(file.readText())
            if (feed.key == "emb-r10") {
                val sample = checkNotNull(javaClass.getResource("/emb/r10-sample.json")).readText()
                val readings = northernMindanaoStations(Json.parseToJsonElement(sample).jsonArray)
                val station = readings.single { it.name == "R10 PENRO" }
                assertTrue(feed.sites.any { it.station == station.name })
                assertEquals("2026-11-20 16:00", station.observed)
                assertEquals(listOf(53.0, 136.0), station.readings.map { it.value })
                continue
            }
            if (feed.key == "emb-r3") {
                val payload = Json.parseToJsonElement(checkNotNull(javaClass.getResource("/emb/r3-sample.json")).readText()).jsonObject
                val reading = checkNotNull(centralLuzonReading(payload, "meycauayan", "pm25"))
                assertEquals(setOf("µg/Ncm", "AQI"), reading.readings.map { it.unit }.toSet())
                assertNull(centralLuzonReading(payload, "clark", "pm25"))
                continue
            }
            for (recipe in feed.tables.map { it.recipe } + listOfNotNull(feed.timeQuery)) {
                val query = Json.parseToJsonElement(File(assets, "$recipe.json").readText()).jsonObject
                val semanticQuery = query.getValue("queries").jsonArray.single().jsonObject.getValue("Query").jsonObject
                    .getValue("Commands").jsonArray.single().jsonObject.getValue("SemanticQueryDataShapeCommand").jsonObject
                assertTrue(semanticQuery.getValue("Query").jsonObject.getValue("Select").jsonArray.isNotEmpty(), recipe)
                assertTrue("Binding" in semanticQuery, recipe)
            }
            val report = snapshot(feed, file.name.substringBefore("."))
            val matched = report.forStations(feed.sites.map { it.station }.toSet())
            assertTrue(matched.stations.any { station ->
                station.readings.isNotEmpty() && (station.observed ?: matched.reported) != null
            }, file.name)
        }
    }

    @Test
    fun extractsManilaIndicesAndOfficialRatingsWithoutConvertingToConcentration() {
        val report = snapshot(ncr, "ncr")
        val manila = report.stations.single { it.name == "Manila" }
        assertEquals("Mehan Garden, Manila City", manila.address)
        assertEquals(listOf(22.0, 18.0), manila.readings.map { it.value })
        assertTrue(manila.readings.all { it.unit == "AQI" && it.rating == "Good" })
        assertEquals("2026-11-20 12:00", report.reported)
        assertNull(manila.observed)
    }

    @Test
    fun widgetAcceptsOnlyFreshLocalOfficialSubIndices() {
        val report = snapshot(ncr, "ncr")
        val now = Instant.parse("2026-11-20T06:00:00Z").toEpochMilli()
        val ids = setOf("Manila")
        val (station, reading) = requireNotNull(report.widgetSubIndex(ids, now))
        assertEquals("Manila", station.name)
        assertEquals("AQI", reading.unit)
        assertEquals("PM2.5", reading.name)
        assertNull(report.widgetSubIndex(setOf("Boracay"), now))
        assertNull(report.widgetSubIndex(ids, now + 86_400_000L))
        assertNull(report.copy(reported = null).widgetSubIndex(ids, now))
        val raw = station.copy(readings = listOf(EmbMeasurement("PM2.5", 18.0, "μg/m³")))
        assertNull(report.copy(stations = listOf(raw)).widgetSubIndex(ids, now))
    }

    @Test
    fun stationSelectionRequiresNameAndGeography() {
        val site = ncr.sites.single { it.station == "Manila" }
        assertTrue(site.matches(setOf(normalizeCity("马尼拉")), 14.5995, 120.9842))
        assertTrue(site.matches(setOf(normalizeCity("City of Manila")), 14.5995, 120.9842))
        assertFalse(site.matches(setOf(normalizeCity("Metro Manila")), 14.6, 120.98))
        assertFalse(site.matches(setOf(normalizeCity("Manila")), 10.3157, 123.8854))
    }

    @Test
    fun emptyAndOfflineResponsesPreserveOnlySameStationHistory() {
        val report = snapshot(ncr, "ncr")
        val ids = setOf("Manila")
        val last = report.forStations(ids)
        for (failed in listOf(null, report.copy(stations = emptyList()))) {
            val result = selectEmbReport(ncr, failed, last, ids)
            assertEquals(last, result.snapshot)
            assertTrue(result.cachedOnError)
            assertFalse(selectEmbReport(ncr, failed, last, setOf("Makati")).cachedOnError)
        }
        assertTrue(selectEmbReport(ncr, null, null, ids).unavailable)
        assertFalse(selectEmbReport(ncr, report, last, ids).cachedOnError)
    }

    @Test
    fun preservesManualDateAndAnnualPrecision() {
        assertEquals("2026-10-28", embClock("10/28/2026", true, false))
        assertEquals("2026-10-28", embClock("2026-10-28 00:00", true, false))
        assertEquals("2026", embClock("2026", false, true))
        assertNull(embClock("02/30/2026", true, false))
        val report = EmbLocalReport(ncr.copy(kind = "manual"), snapshot(ncr, "ncr"))
        assertFalse(report.hasRecentMonitoring(Instant.parse("2026-11-20T06:00:00Z").toEpochMilli()))
    }

    @Test
    fun respectsDeclaredUnits() {
        assertEquals(6.2, embNumber(" 6.2 ug/NCM", "µg/Ncm"))
        assertNull(embNumber("6.2 ug/NCM", "AQI"))
        assertNull(embNumber("6.2 ppm", "µg/Ncm"))
        assertNull(embNumber("-13", "AQI"))
    }

    @Test
    fun staleUsesReportClockRatherThanDownloadTime() {
        val now = Instant.parse("2026-11-20T05:00:00Z").toEpochMilli()
        assertNull(reportAgeMillis("2026-11-19 56:00", now))
        assertNull(reportAgeMillis("2026-11-18 12:00extra", now))
        assertNull(reportAgeMillis("2026-11-22 12:00", now))
        assertEquals(23L, reportAgeMillis("2026-10-28", now)?.div(86_400_000L))
        assertTrue(isOldReport("2026-11-18 12:00", now))
        assertFalse(isOldReport("2026-11-20 12:00", now))
    }
}
