package org.breezyweather.sources.emb

import android.content.Context
import breezyweather.domain.location.model.Location
import breezyweather.domain.source.SourceContinent
import breezyweather.domain.source.SourceFeature
import breezyweather.domain.weather.wrappers.WeatherWrapper
import dagger.hilt.android.qualifiers.ApplicationContext
import io.reactivex.rxjava3.core.Observable
import kotlinx.coroutines.rx3.rxSingle
import org.breezyweather.BuildConfig
import org.breezyweather.R
import org.breezyweather.common.source.HttpSource
import org.breezyweather.common.source.NonFreeNetSource
import org.breezyweather.common.source.WeatherSource
import org.breezyweather.sources.openmeteo.OpenMeteoService
import javax.inject.Inject

class EmbAirQualityService @Inject constructor(
    @ApplicationContext private val context: Context,
    private val stations: EmbRepository,
    private val model: OpenMeteoService,
) : HttpSource(), WeatherSource, NonFreeNetSource {
    override val id = ID
    override val name get() = context.getString(R.string.emb_source_name)
    override val continent = SourceContinent.ASIA
    override val privacyPolicyUrl = BuildConfig.PRIVACY_POLICY_LINK
    override val supportedFeatures = mapOf(
        SourceFeature.AIR_QUALITY to "DENR-EMB • ${model.supportedFeatures.getValue(SourceFeature.AIR_QUALITY)}"
    )
    override val attributionLinks = model.attributionLinks +
        ("DENR-EMB" to "https://air.emb.gov.ph/ambient-air-quality-monitoring/")

    override fun isFeatureSupportedForLocation(location: Location, feature: SourceFeature) =
        feature == SourceFeature.AIR_QUALITY && location.countryCode.equals("PH", ignoreCase = true)

    override fun getFeaturePriorityForLocation(location: Location, feature: SourceFeature) =
        if (isFeatureSupportedForLocation(location, feature)) WeatherSource.PRIORITY_HIGHEST else WeatherSource.PRIORITY_NONE

    override fun requestWeather(
        context: Context,
        location: Location,
        requestedFeatures: List<SourceFeature>,
    ): Observable<WeatherWrapper> {
        if (SourceFeature.AIR_QUALITY !in requestedFeatures) return Observable.just(WeatherWrapper())
        // Station sub-indices have their own display and cache. The shared weather model only accepts concentrations.
        val stationReports = rxSingle { stations.forLocation(location) }
            .onErrorReturn { emptyList() }
            .toObservable()
        val modelWeather = Observable.defer {
            model.requestWeather(context, location, listOf(SourceFeature.AIR_QUALITY))
        }.onErrorReturn { WeatherWrapper(failedFeatures = mapOf(SourceFeature.AIR_QUALITY to it)) }
        return Observable.zip(stationReports, modelWeather) { _, concentrations -> concentrations }
    }

    companion object {
        const val ID = "denr_emb"
    }
}
