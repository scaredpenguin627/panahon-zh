package org.breezyweather.sources.panahon

import breezyweather.domain.weather.reference.WeatherCode
import org.breezyweather.common.exceptions.OutdatedServerDataException
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertThrows
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import java.time.Instant

class PanahonConverterTest {
    private val now = Instant.parse("2026-12-31T15:20:00Z").toEpochMilli()
    private val evening = PanahonPeriod(time = "11:00 PM", desc = "Partly cloudy skies", tmp = "28 °C")
    private val forecast = PanahonForecast(
        nc = evening,
        hc = listOf(evening, evening.copy(time = "12:00 AM")),
        dc = listOf(PanahonPeriod(dt = "12/31", tmax = "31 °C"), PanahonPeriod(dt = "01/01", tmin = "22 °C"))
    )

    @Test
    fun localTimeCrossesMidnightAndNewYear() {
        val (hourly, daily) = forecast.toForecast(now)
        assertEquals("2026-12-31T16:00:00Z", hourly.last().date.toInstant().toString())
        assertEquals("2026-12-31T16:00:00Z", daily.last().date.toInstant().toString())
        assertEquals(WeatherCode.PARTLY_CLOUDY, forecast.toCurrent(now).weatherCode)
        assertThrows(OutdatedServerDataException::class.java) { forecast.toCurrent(now + 7200000) }
    }

    @Test
    fun dashboardWithYesterdaysLastHourAtMidnight() {
        val afterMidnight = Instant.parse("2026-12-31T16:10:00Z").toEpochMilli()
        val rolled = forecast.copy(dc = listOf(PanahonPeriod(dt = "01/01")))
        assertEquals(WeatherCode.PARTLY_CLOUDY, rolled.toCurrent(afterMidnight).weatherCode)
        assertEquals("2026-12-31T15:00:00Z", rolled.toForecast(afterMidnight).first.first().date.toInstant().toString())
    }

    @Test
    fun rejectsTheServersDefaultLocation() {
        val cebu = PanahonArea(lat = "10.3157", lon = "123.8854", municipality = "Cebu City")
        assertTrue(cebu.matches(10.3157, 123.8854))
        assertFalse(cebu.matches(14.5995, 120.9842))
        assertFalse(cebu.copy(lat = null).matches(10.3157, 123.8854))
    }

    @Test
    fun filtersUnrelatedAndExpiredNationalFeedItems() {
        val local = PanahonBulletin("local", "Official title", valid_until_at = "January 01, 2027 01:00 AM", locations = listOf("Metro Manila"))
        val alerts = PanahonAlerts(true, listOf(
            local, local.copy(alert_id = "national", locations = listOf("Nationwide")),
            local.copy(alert_id = "cebu", locations = listOf("Cebu")),
            local.copy(alert_id = "expired", valid_until_at = "December 31, 2026 10:00 PM")
        ))
        assertEquals(listOf("local", "national"), alerts.toAlerts(PanahonArea(region = "Metro Manila"), now).map { it.alertId })
    }
}
