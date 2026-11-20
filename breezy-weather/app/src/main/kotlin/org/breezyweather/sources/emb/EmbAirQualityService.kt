package org.breezyweather.sources.emb

import android.content.Context
import breezyweather.domain.location.model.Location
import breezyweather.domain.source.SourceContinent
import breezyweather.domain.source.SourceFeature
import breezyweather.domain.weather.wrappers.AirQualityWrapper
import breezyweather.domain.weather.wrappers.WeatherWrapper
import dagger.hilt.android.qualifiers.ApplicationContext
import io.reactivex.rxjava3.core.Observable
import kotlinx.coroutines.rx3.rxSingle
import org.breezyweather.BuildConfig
import org.breezyweather.R
import org.breezyweather.common.source.HttpSource
import org.breezyweather.common.source.NonFreeNetSource
import org.breezyweather.common.source.WeatherSource
import javax.inject.Inject

class EmbAirQualityService @Inject constructor(
    @ApplicationContext private val context: Context,
    private val stations: EmbRepository,
) : HttpSource(), WeatherSource, NonFreeNetSource {
    override val id = ID
    override val name get() = context.getString(R.string.emb_source_name)
    override val continent = SourceContinent.ASIA
    override val privacyPolicyUrl = BuildConfig.PRIVACY_POLICY_LINK
    override val supportedFeatures = mapOf(SourceFeature.AIR_QUALITY to "DENR-EMB")
    override val attributionLinks = mapOf("DENR-EMB" to "https://air.emb.gov.ph/ambient-air-quality-monitoring/")

    override fun isFeatureSupportedForLocation(location: Location, feature: SourceFeature) =
        feature == SourceFeature.AIR_QUALITY && location.countryCode.equals("PH", ignoreCase = true)

    override fun getFeaturePriorityForLocation(location: Location, feature: SourceFeature) =
        if (isFeatureSupportedForLocation(location, feature)) WeatherSource.PRIORITY_HIGHEST else WeatherSource.PRIORITY_NONE

    override fun requestWeather(
        context: Context,
        location: Location,
        requestedFeatures: List<SourceFeature>,
    ): Observable<WeatherWrapper> {
        if (SourceFeature.AIR_QUALITY !in requestedFeatures || !location.countryCode.equals("PH", ignoreCase = true)) {
            return Observable.just(WeatherWrapper())
        }
        return rxSingle { stations.forLocation(location) }
            .map { reports ->
                WeatherWrapper(
                    airQuality = reports.toAirQuality()?.let { AirQualityWrapper(current = it) }
                )
            }
            .toObservable()
    }

    companion {
        const val ID = "denr_emb"
    }
}

internal val Location.embAirQualitySource: String?
    get() = airQualitySource?.takeIf { it == EmbAirQualityService.ID && countryCode.equals("PH", ignoreCase = true) }
