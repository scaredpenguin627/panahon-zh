package org.breezyweather.sources.emb

import android.content.Context
import breezyweather.domain.location.model.Location
import breezyweather.domain.source.SourceFeature
import breezyweather.domain.weather.wrappers.WeatherWrapper
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.collections.immutable.persistentListOf
import org.breezyweather.common.source.FeatureSource
import org.breezyweather.sources.SourceManager
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test

class EmbAirQualityServiceTest {
    private val context = mockk<Context>()
    private val stations = mockk<EmbRepository>()
    private val service = EmbAirQualityService(context, stations)
    private val manila = Location(city = "Manila", countryCode = "PH", latitude = 14.5995, longitude = 120.9842)
    private val aq = SourceFeature.AIR_QUALITY

    @Test
    fun officialSubIndicesRemainStationReadings() {
        val report = mockk<EmbLocalReport>()
        coEvery { stations.forLocation(manila) } returns listOf(report)
        assertEquals(WeatherWrapper(), service.requestWeather(context, manila, listOf(aq)).blockingFirst())
        assertEquals(mapOf(aq to "DENR-EMB"), service.supportedFeatures)
        coVerify(exactly = 1) { stations.forLocation(manila) }
    }

    @Test
    fun noStationOrDisabledSourceNeverProvidesModelFallback() {
        coEvery { stations.forLocation(manila) } returns emptyList()
        assertNull(service.requestWeather(context, manila, listOf(aq)).blockingFirst().airQuality)
        assertEquals(WeatherWrapper(), service.requestWeather(context, manila, listOf(SourceFeature.CURRENT)).blockingFirst())
        assertEquals(WeatherWrapper(), service.requestWeather(context, manila.copy(countryCode = "JP"), listOf(aq)).blockingFirst())
        coVerify(exactly = 1) { stations.forLocation(any()) }
        assertNull(manila.copy(airQualitySource = "").embAirQualitySource)
        assertNull(manila.copy(airQualitySource = "openmeteo").embAirQualitySource)
        assertNull(manila.copy(countryCode = "JP", airQualitySource = service.id).embAirQualitySource)
        assertEquals(service.id, manila.copy(airQualitySource = service.id).embAirQualitySource)
    }

    @Test
    fun selectorDoesNotOfferModelEvenWhenAnOldLocationSelectedIt() {
        val model = mockk<FeatureSource> { every { id } returns "openmeteo" }
        val manager = mockk<SourceManager>()
        every { manager.getFeatureSources() } returns persistentListOf(service, model)
        every { manager.getSupportedFeatureSources(aq, any(), any()) } answers { callOriginal() }
        assertEquals(listOf(service), manager.getSupportedFeatureSources(aq, manila, "openmeteo"))
        assertEquals(emptyList<FeatureSource>(), manager.getSupportedFeatureSources(aq, manila.copy(countryCode = "JP"), service.id))
    }
}
