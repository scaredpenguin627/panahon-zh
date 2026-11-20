package org.breezyweather.sources.panahon

import breezyweather.domain.location.model.Location
import breezyweather.domain.weather.model.Hourly
import breezyweather.domain.weather.model.Wind
import breezyweather.domain.weather.wrappers.CurrentWrapper
import breezyweather.domain.weather.wrappers.HourlyWrapper
import breezyweather.domain.weather.wrappers.TemperatureWrapper
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.doubleOrNull
import org.breezyweather.sources.completeCurrentFromHourlyData
import org.breezyweather.sources.pagasa.json.PagasaCurrentResult
import org.breezyweather.unit.computing.computeDewPoint
import org.breezyweather.unit.ratio.Ratio.Companion.percent
import org.breezyweather.unit.speed.Speed.Companion.kilometersPerHour
import org.breezyweather.unit.speed.Speed.Companion.metersPerSecond
import org.breezyweather.unit.temperature.Temperature.Companion.celsius
import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.Test
import java.time.Instant
import java.util.Date

class PanahonAtmosphereTest {
    private val now = Instant.parse("2026-11-20T10:45:00Z").toEpochMilli()
    private val manila = Location(latitude = 14.5995, longitude = 120.9842)
    private val boracay = Location(latitude = 11.98013, longitude = 121.91932)
    private val aws = Json { ignoreUnknownKeys = true }.decodeFromString<PagasaCurrentResult>(
        """{"datetime":"November 20, 2026, 6:30 pm","site_name":"Science Garden, Quezon City",
            "latitude":"14.645","longitude":"121.044","temperature":"26 °C","humidity":"88 %",
            "wind_speed":"3.6 km/hr","wind_direction":"ENE","pressure":1012.9}"""
    )

    @Test
    fun freshLocalObservationUsesStationTemperatureForDewPoint() {
        val closerButStale = aws.copy(latitude = "14.5995", longitude = "120.9842", datetime = "November 20, 2026, 4:44 pm")
        val future = closerButStale.copy(datetime = "November 20, 2026, 7:00 pm")
        val bad = aws.copy(windSpeed = "-- km/hr")
        val older = aws.copy(datetime = "November 20, 2026, 6:20 pm", humidity = "77 %")
        val reports = mapOf("key" to listOf(closerButStale, future, aws, bad, older))
        val selected = requireNotNull(nearbyObservation(manila, reports, now))
        assertEquals(1012.9, aws.pressure?.doubleOrNull)
        assertEquals(3.6.kilometersPerHour, selected.wind?.speed)
        assertEquals(88.percent, selected.humidity)
        assertEquals(computeDewPoint(26.celsius, 88.percent), selected.dewPoint)
        assertEquals("PAGASA AWS · Science Garden, Quezon City · 2026-11-20 18:30 PHT", selected.attribution())
        val romblon = aws.copy(latitude = "12.593", longitude = "122.219", siteName = "Romblon")
        assertNull(nearbyObservation(boracay, mapOf("key" to listOf(aws, romblon)), now))
    }

    @Test
    fun forecastWindAndHumidityJoinByTimestampAndNeverReplacePagasaTemperature() {
        val hour = Instant.parse("2026-11-20T10:00:00Z").epochSecond
        val grid = PanahonModelGrid(
            latitude = 11.915642, longitude = 121.96413,
            hourly = PanahonModelHours(
                time = listOf(hour), humidity = listOf(79.0), dewPoint = listOf(23.0),
                speed = listOf(5.5), direction = listOf(42.0), gusts = listOf(7.0)
            )
        )
        val forecast = HourlyWrapper(date = Date(hour * 1000), temperature = TemperatureWrapper(29.celsius))
        val next = forecast.copy(date = Date((hour + 3600) * 1000))
        val reading = grid.hourlyReadings(boracay)
        val merged = listOf(forecast, next).withAtmosphere(reading)
        assertEquals(29.celsius, merged.first().temperature?.temperature)
        assertEquals(79.percent, merged.first().relativeHumidity)
        assertEquals(23.celsius, merged.first().dewPoint)
        assertEquals(5.5.metersPerSecond, merged.first().wind?.speed)
        assertEquals("Open-Meteo · 2026-11-20 18:00 PHT", reading.getValue(forecast.date).attribution())
        assertEquals(next, merged.last())
        assertTrue(grid.hourlyReadings(manila).isEmpty())
        assertEquals(listOf(forecast), listOf(forecast).withAtmosphere(emptyMap()))
    }

    @Test
    fun hourlyDerivedCurrentKeepsModelDewPointAndCreditsOnlyTheFieldsActuallyUsed() {
        val ph = boracay.copy(forecastSource = "panahon")
        val hour = Hourly(
            date = Date(now - now % 3_600_000), relativeHumidity = 77.percent, dewPoint = 23.2.celsius,
            wind = Wind(speed = 5.32.metersPerSecond)
        )
        fun merged(current: CurrentWrapper?, location: Location = ph) =
            completeCurrentFromHourlyData(current, hour, null, null, location)

        val derived = merged(null)
        assertEquals("Open-Meteo · 2026-11-20 18:00 PHT", derived.windHumiditySource)
        assertEquals(23.2.celsius, derived.dewPoint)
        assertEquals(23.2.celsius, merged(CurrentWrapper(temperature = TemperatureWrapper(30.celsius))).dewPoint)

        val station = CurrentWrapper(
            relativeHumidity = 88.percent, dewPoint = 24.celsius,
            wind = Wind(speed = 3.6.kilometersPerHour), windHumiditySource = "PAGASA AWS · Science Garden"
        )
        assertEquals(station.windHumiditySource, merged(station).windHumiditySource)
        assertEquals(station.dewPoint, merged(station).dewPoint)
        val mixed = merged(CurrentWrapper(wind = Wind(speed = 8.metersPerSecond)))
        assertEquals(8.metersPerSecond, mixed.wind?.speed)
        assertEquals(23.2.celsius, mixed.dewPoint)
        assertNull(mixed.windHumiditySource) // The wind and humidity do not come from one source.
        assertNull(merged(null, ph.copy(forecastSource = "other")).windHumiditySource)
    }

}
