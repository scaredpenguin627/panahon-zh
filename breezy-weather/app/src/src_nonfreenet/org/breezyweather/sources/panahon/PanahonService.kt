package org.breezyweather.sources.panahon

import android.content.Context
import breezyweather.domain.location.model.Location
import breezyweather.domain.source.SourceFeature
import breezyweather.domain.weather.wrappers.WeatherWrapper
import io.reactivex.rxjava3.core.Observable
import org.breezyweather.R
import org.breezyweather.common.exceptions.InvalidOrIncompleteDataException
import retrofit2.Retrofit
import java.util.Locale
import javax.inject.Inject
import javax.inject.Named

class PanahonService @Inject constructor(@Named("JsonClient") client: Retrofit.Builder) : PanahonSource() {
    private val api by lazy {
        client.baseUrl("https://api.mobile.panahon.gov.ph/api/v1/").build().create(PanahonApi::class.java)
    }

    override fun requestWeather(
        context: Context,
        location: Location,
        requestedFeatures: List<SourceFeature>,
    ): Observable<WeatherWrapper> {
        val forecast = SourceFeature.FORECAST in requestedFeatures
        val current = SourceFeature.CURRENT in requestedFeatures
        val alerts = SourceFeature.ALERT in requestedFeatures
        val lat = location.latitude
        val lon = location.longitude
        return Observable.zip(
            fetchIf(current || forecast || alerts) { api.dashboard(lat, lon).map { it.valid(it.success && it.meta?.matches(lat, lon) == true) } },
            fetchIf(alerts) { api.alerts(lat, lon).map { it.valid(it.success) } }
        ) { dashboard, bulletins ->
            val now = System.currentTimeMillis()
            val failures = mutableMapOf<SourceFeature, Throwable>()
            fun <T> convert(feature: SourceFeature, requested: Boolean, block: () -> T): T? {
                if (!requested) return null
                return try {
                    block()
                } catch (e: Exception) {
                    failures[feature] = e
                    null
                }
            }

            val outlook = convert(SourceFeature.FORECAST, forecast) { dashboard.getOrThrow()!!.data.toForecast(now) { describe(context, it) } }
            val observed = convert(SourceFeature.CURRENT, current) { dashboard.getOrThrow()!!.data.toCurrent(now) { describe(context, it) } }
            val warnings = convert(SourceFeature.ALERT, alerts) {
                val area = dashboard.getOrThrow()!!.meta ?: throw InvalidOrIncompleteDataException()
                bulletins.getOrThrow()!!.toAlerts(area, now)
            }
            WeatherWrapper(
                hourlyForecast = outlook?.first,
                dailyForecast = outlook?.second,
                current = observed,
                alertList = warnings,
                failedFeatures = failures
            )
        }
    }

    private fun describe(context: Context, text: String): String = when (text.trim().lowercase(Locale.ROOT)) {
        "clear skies", "clear sky" -> context.getString(R.string.common_weather_text_clear_sky)
        "partly cloudy skies" -> context.getString(R.string.common_weather_text_partly_cloudy)
        "cloudy skies" -> context.getString(R.string.common_weather_text_cloudy)
        "partly cloudy skies to at times cloudy with rainshowers and thunderstorms" ->
            context.getString(R.string.panahon_partly_cloudy_with_thunderstorms)
        else -> text
    }

    private fun <T> T.valid(success: Boolean): T =
        if (success) this else throw InvalidOrIncompleteDataException()

    private fun <T : Any> fetchIf(requested: Boolean, call: () -> Observable<T>): Observable<Result<T?>> =
        if (requested) {
            call().map { Result.success<T?>(it) }.onErrorReturn { Result.failure(it) }
        } else {
            Observable.just(Result.success(null))
        }
}
