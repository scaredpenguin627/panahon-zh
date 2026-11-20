/*
 * This file is part of Breezy Weather.
 *
 * Breezy Weather is free software: you can redistribute it and/or modify it
 * under the terms of the GNU Lesser General Public License as published by the
 * Free Software Foundation, version 3 of the License.
 *
 * Breezy Weather is distributed in the hope that it will be useful, but
 * WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
 * or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
 * License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with Breezy Weather. If not, see <https://www.gnu.org/licenses/>.
 */

package org.breezyweather.sources.climweb

import android.content.Context
import androidx.compose.ui.text.input.KeyboardType
import breezyweather.domain.location.model.Location
import breezyweather.domain.source.SourceContinent
import breezyweather.domain.source.SourceFeature
import breezyweather.domain.weather.model.Alert
import breezyweather.domain.weather.reference.AlertSeverity
import breezyweather.domain.weather.wrappers.WeatherWrapper
import com.google.maps.android.PolyUtil
import com.google.maps.android.data.geojson.GeoJsonFeature
import com.google.maps.android.data.geojson.GeoJsonMultiPolygon
import com.google.maps.android.data.geojson.GeoJsonParser
import com.google.maps.android.data.geojson.GeoJsonPolygon
import io.reactivex.rxjava3.core.Observable
import org.breezyweather.R
import org.breezyweather.common.preference.EditTextPreference
import org.breezyweather.common.preference.Preference
import org.breezyweather.common.source.ConfigurableSource
import org.breezyweather.common.source.HttpSource
import org.breezyweather.common.source.WeatherSource
import org.breezyweather.common.source.WeatherSource.Companion.PRIORITY_HIGHEST
import org.breezyweather.common.source.WeatherSource.Companion.PRIORITY_NONE
import org.breezyweather.domain.settings.SourceConfigStore
import org.breezyweather.sources.climweb.json.ClimWebAlertsResult
import org.json.JSONObject
import retrofit2.Retrofit
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

/**
 * Open-source system used by many African countries
 * https://github.com/wmo-raf/climweb
 *
 * Is an abstract class that must be implemented by each national source
 */
abstract class ClimWebService : HttpSource(), WeatherSource, ConfigurableSource {

    protected abstract val context: Context
    protected abstract val jsonClient: Retrofit.Builder

    /**
     * E.g. https://www.weatherzw.org.zw/
     */
    protected abstract val baseUrl: String

    /**
     * Populate this with the country's ISO 3166-1 alpha-2 code.
     * E.g. "ZW" for Zimbabwe
     */
    protected abstract val countryCode: String

    // TODO: Remove this if/when ClimWeb starts being used outside of Africa
    override val continent = SourceContinent.AFRICA

    /**
     * E.g. R.string.settings_weather_source_msd_zw_instance
     */
    protected abstract val instancePreference: Int

    protected abstract val weatherAttribution: String
    protected abstract val alertAttribution: String

    protected val mApi: ClimWebApi
        get() {
            return jsonClient
                .baseUrl(instance!!)
                .build()
                .create(ClimWebApi::class.java)
        }

    // SECONDARY WEATHER SOURCE
    override val supportedFeatures
        get() = mapOf(
            SourceFeature.ALERT to alertAttribution,
        )
    override val attributionLinks
        get() = mapOf(
            weatherAttribution to baseUrl
        )
    override fun isFeatureSupportedForLocation(
        location: Location,
        feature: SourceFeature,
    ): Boolean {
        return when (feature) {
            SourceFeature.ALERT ->
                location.countryCode != null &&
                    location.countryCode.equals(countryCode, ignoreCase = true)
            else -> false
        }
    }

    override fun getFeaturePriorityForLocation(
        location: Location,
        feature: SourceFeature,
    ): Int {
        return when {
            isFeatureSupportedForLocation(location, feature) -> PRIORITY_HIGHEST
            else -> PRIORITY_NONE
        }
    }

    override fun requestWeather(
        context: Context,
        location: Location,
        requestedFeatures: List<SourceFeature>,
    ): Observable<WeatherWrapper> {
        val failedFeatures = mutableMapOf<SourceFeature, Throwable>()
        val alerts = if (SourceFeature.ALERT in requestedFeatures) {
            mApi.getAlerts().onErrorResumeNext {
                failedFeatures[SourceFeature.ALERT] = it
                Observable.just(ClimWebAlertsResult())
            }
        } else {
            Observable.just(ClimWebAlertsResult())
        }


        return alerts.map { alertsResult ->
            WeatherWrapper(
                alertList = if (SourceFeature.ALERT in requestedFeatures) {
                    getAlertList(location, alertAttribution, alertsResult)
                } else {
                    null
                },
                failedFeatures = failedFeatures
            )
        }
    }

    private fun getAlertList(
        location: Location,
        source: String?,
        alertsResult: ClimWebAlertsResult,
    ): List<Alert> {
        val matchingAlerts = getMatchingAlerts(location, alertsResult.features)
        if (matchingAlerts.isEmpty()) return emptyList()

        val alertList = mutableListOf<Alert>()
        val formatter = SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ssZ", Locale.ENGLISH)
        var alertId: String
        var startDate: Date?
        var endDate: Date?
        var severity: AlertSeverity

        matchingAlerts.forEach {
            alertId =
                it.getProperty("id")
                    ?: "${it.getProperty("event")} ${it.getProperty("areaDesc")} ${it.getProperty("onset")}"
            startDate = if (it.getProperty("onset") != null) {
                formatter.parse(it.getProperty("onset")!!)
            } else {
                null
            }
            endDate = if (it.getProperty("expires") != null) {
                formatter.parse(it.getProperty("expires")!!)
            } else {
                null
            }
            severity = with(it.getProperty("severity")) {
                when {
                    equals("Extreme", ignoreCase = true) -> AlertSeverity.EXTREME
                    equals("Severe", ignoreCase = true) -> AlertSeverity.SEVERE
                    equals("Moderate", ignoreCase = true) -> AlertSeverity.MODERATE
                    equals("Minor", ignoreCase = true) -> AlertSeverity.MINOR
                    else -> AlertSeverity.UNKNOWN
                }
            }

            alertList.add(
                Alert(
                    alertId = alertId,
                    startDate = startDate,
                    endDate = endDate,
                    headline = it.getProperty("headline")?.trim(),
                    description = it.getProperty("description")?.trim(),
                    instruction = it.getProperty("instruction")?.trim(),
                    source = source,
                    severity = severity,
                    color = Alert.colorFromSeverity(severity)
                )
            )
        }
        return alertList
    }

    private fun getMatchingAlerts(
        location: Location,
        alertFeatures: Any?,
    ): List<GeoJsonFeature> {
        val json = """{"type":"FeatureCollection","features":$alertFeatures}"""
        val geoJsonParser = GeoJsonParser(JSONObject(json))
        return geoJsonParser.features.filter { feature ->
            when (feature.geometry) {
                is GeoJsonPolygon -> (feature.geometry as GeoJsonPolygon).coordinates.any { polygon ->
                    PolyUtil.containsLocation(location.latitude, location.longitude, polygon, true)
                }
                is GeoJsonMultiPolygon -> (feature.geometry as GeoJsonMultiPolygon).polygons.any {
                    it.coordinates.any { polygon ->
                        PolyUtil.containsLocation(location.latitude, location.longitude, polygon, true)
                    }
                }
                else -> false
            }
        }
    }


    // CONFIG
    private val config
        get() = SourceConfigStore(context, id)
    override val isConfigured = true
    override val isRestricted = false
    private var instance: String?
        set(value) {
            value?.let {
                config.edit().putString("instance", value).apply()
            } ?: config.edit().remove("instance").apply()
        }
        get() = config.getString("instance", null) ?: baseUrl

    override fun getPreferences(context: Context): List<Preference> {
        return listOf(
            EditTextPreference(
                titleId = instancePreference,
                summary = { _, content ->
                    content.ifEmpty {
                        baseUrl
                    }
                },
                content = if (instance != baseUrl) instance else null,
                placeholder = baseUrl,
                regex = EditTextPreference.URL_REGEX,
                regexError = context.getString(R.string.settings_source_instance_invalid),
                keyboardType = KeyboardType.Uri,
                onValueChanged = {
                    instance = if (it == baseUrl) null else it.ifEmpty { null }
                }
            )
        )
    }
}
