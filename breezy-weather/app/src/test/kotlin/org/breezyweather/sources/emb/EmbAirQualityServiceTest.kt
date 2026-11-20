package org.breezyweather.sources.emb

import android.content.Context
import breezyweather.domain.location.model.Location
import breezyweather.domain.source.SourceFeature
import breezyweather.domain.weather.model.AirQuality
import breezyweather.domain.weather.wrappers.AirQualityWrapper
import breezyweather.domain.weather.wrappers.WeatherWrapper
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.reactivex.rxjava3.core.Observable
import org.breezyweather.common.source.WeatherSource
import org.breezyweather.sources.openmeteo.OpenMeteoService
import org.breezyweather.unit.pollutant.PollutantConcentration.Companion.microgramsPerCubicMeter
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertSame
import org.junit.jupiter.api.Test
import io.reactivex.rxjava3.plugins.RxJavaPlugins
import io.reactivex.rxjava3.schedulers.TestScheduler
import kotlinx.coroutines.awaitCancellation
import java.io.IOException
import java.util.concurrent.TimeUnit
import java.util.concurrent.TimeoutException

class EmbAirQualityServiceTest {
    private val context = mockk<Context>()
    private val stations = mockk<EmbRepository>()
    private val model = mockk<OpenMeteoService> {
        every { supportedFeatures } returns mapOf(SourceFeature.AIR_QUALITY to "Open-Meteo")
        every { attributionLinks } returns emptyMap()
    }
    private val service = EmbAirQualityService(context, stations, model)
    private val manila = Location(city = "Manila", countryCode = "PH", latitude = 14.5995, longitude = 120.9842)

    @Test
    fun recommendsOnlyAirQualityForPhilippineLocations() {
        assertEquals(setOf(SourceFeature.AIR_QUALITY), service.supportedFeatures.keys)
        assertEquals(WeatherSource.PRIORITY_HIGHEST, service.getFeaturePriorityForLocation(manila, SourceFeature.AIR_QUALITY))
        assertFalse(service.isFeatureSupportedForLocation(manila, SourceFeature.FORECAST))
        assertFalse(service.isFeatureSupportedForLocation(manila.copy(countryCode = "JP"), SourceFeature.AIR_QUALITY))
    }

    @Test
    fun fetchesStationReportsAndKeepsTheModelIfThereIsNoStation() {
        val features = listOf(SourceFeature.AIR_QUALITY)
        val result = WeatherWrapper(airQuality = AirQualityWrapper(current = AirQuality(pM25 = 8.5.microgramsPerCubicMeter)))
        coEvery { stations.forLocation(manila) } returns emptyList()
        every { model.requestWeather(context, manila, features) } returns Observable.just(result)
        assertSame(result, service.requestWeather(context, manila, features).blockingFirst())
        coVerify(exactly = 1) { stations.forLocation(manila) }
    }

    @Test
    fun stationFailureDoesNotDiscardTheModelChart() {
        val features = listOf(SourceFeature.AIR_QUALITY)
        val result = WeatherWrapper(airQuality = AirQualityWrapper(current = AirQuality(pM10 = 17.0.microgramsPerCubicMeter)))
        coEvery { stations.forLocation(manila) } throws IOException("EMB unavailable")
        every { model.requestWeather(context, manila, features) } returns Observable.just(result)
        assertSame(result, service.requestWeather(context, manila, features).blockingFirst())
    }

    @Test
    fun modelFailureDoesNotCancelTheStationFetch() {
        val features = listOf(SourceFeature.AIR_QUALITY)
        val unavailable = IOException("model unavailable")
        coEvery { stations.forLocation(manila) } returns listOf(mockk())
        every { model.requestWeather(context, manila, features) } returns Observable.error(unavailable)
        val result = service.requestWeather(context, manila, features).blockingFirst()
        assertSame(unavailable, result.failedFeatures?.get(SourceFeature.AIR_QUALITY))
        coVerify(exactly = 1) { stations.forLocation(manila) }
    }

    @Test
    fun aStalledEndpointDoesNotBlockTheOtherAirQualityResult() {
        val timer = TestScheduler()
        val previous = RxJavaPlugins.getComputationSchedulerHandler()
        RxJavaPlugins.setComputationSchedulerHandler { timer }
        try {
            val features = listOf(SourceFeature.AIR_QUALITY)
            val result = WeatherWrapper(airQuality = AirQualityWrapper(current = AirQuality(pM10 = 17.0.microgramsPerCubicMeter)))
            coEvery { stations.forLocation(manila) } coAnswers { awaitCancellation() }
            every { model.requestWeather(context, manila, features) } returns Observable.just(result)
            val slowStations = service.requestWeather(context, manila, features).test()
            timer.advanceTimeBy(10, TimeUnit.SECONDS)
            slowStations.assertResult(result)

            coEvery { stations.forLocation(manila) } returns emptyList()
            every { model.requestWeather(context, manila, features) } returns Observable.never()
            val slowModel = service.requestWeather(context, manila, features).test()
            timer.advanceTimeBy(10, TimeUnit.SECONDS)
            slowModel.awaitDone(2, TimeUnit.SECONDS).assertComplete().assertValue {
                it.failedFeatures?.get(SourceFeature.AIR_QUALITY) is TimeoutException
            }
        } finally {
            RxJavaPlugins.setComputationSchedulerHandler(previous)
        }
    }
}
