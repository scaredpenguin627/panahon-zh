package org.breezyweather.sources.panahon

import breezyweather.domain.location.model.Location
import breezyweather.domain.weather.model.Wind
import breezyweather.domain.weather.wrappers.HourlyWrapper
import com.google.maps.android.SphericalUtil
import com.google.maps.android.model.LatLng
import io.reactivex.rxjava3.core.Observable
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import org.breezyweather.sources.getWindDegree
import org.breezyweather.sources.pagasa.json.PagasaCurrentResult
import org.breezyweather.unit.computing.computeDewPoint
import org.breezyweather.unit.ratio.Ratio
import org.breezyweather.unit.ratio.Ratio.Companion.percent
import org.breezyweather.unit.speed.Speed.Companion.kilometersPerHour
import org.breezyweather.unit.speed.Speed.Companion.metersPerSecond
import org.breezyweather.unit.temperature.Temperature
import org.breezyweather.unit.temperature.Temperature.Companion.celsius
import retrofit2.http.GET
import retrofit2.http.Query
import java.text.ParsePosition
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.TimeZone
import kotlin.time.Duration.Companion.hours

internal interface PanahonModelApi {
    @GET("v1/forecast?hourly=relative_humidity_2m,dew_point_2m,wind_speed_10m,wind_direction_10m,wind_gusts_10m&wind_speed_unit=ms&timeformat=unixtime&forecast_days=3")
    fun weather(@Query("latitude") latitude: Double, @Query("longitude") longitude: Double): Observable<PanahonModelGrid>
}

@Serializable
internal data class PanahonModelGrid(
    val latitude: Double,
    val longitude: Double,
    val hourly: PanahonModelHours,
)

@Serializable
internal data class PanahonModelHours(
    val time: List<Long>,
    @SerialName("relative_humidity_2m") val humidity: List<Double?> = emptyList(),
    @SerialName("dew_point_2m") val dewPoint: List<Double?> = emptyList(),
    @SerialName("wind_speed_10m") val speed: List<Double?> = emptyList(),
    @SerialName("wind_direction_10m") val direction: List<Double?> = emptyList(),
    @SerialName("wind_gusts_10m") val gusts: List<Double?> = emptyList(),
)

internal data class AtmosphereReading(
    val time: Date,
    val wind: Wind?,
    val humidity: Ratio?,
    val dewPoint: Temperature?,
    val station: String? = null,
) {
    fun attribution(): String {
        val clock = SimpleDateFormat("yyyy-MM-dd HH:mm", Locale.US).apply {
            timeZone = TimeZone.getTimeZone("Asia/Manila")
        }
        val source = station?.let { "PAGASA AWS · $it" } ?: "Open-Meteo"
        return "$source · ${clock.format(time)} PHT"
    }
}

internal fun PanahonModelGrid.hourlyReadings(location: Location): Map<Date, AtmosphereReading> {
    // Open-Meteo returns the centre of a forecast grid cell, not the exact query coordinates.
    if (distanceMeters(location, latitude, longitude) > 50_000) return emptyMap()
    return hourly.time.mapIndexedNotNull { i, seconds ->
        val time = Date(seconds * 1000)
        val speed = hourly.speed.getOrNull(i)?.takeIf { it >= 0 }?.metersPerSecond
        val wind = speed?.let {
            Wind(
                speed = it,
                degree = hourly.direction.getOrNull(i)?.takeIf { d -> d in 0.0..360.0 },
                gusts = hourly.gusts.getOrNull(i)?.takeIf { g -> g >= 0 }?.metersPerSecond
            )
        }
        val humidity = hourly.humidity.getOrNull(i)?.takeIf { it in 0.0..100.0 }?.percent
        val dew = hourly.dewPoint.getOrNull(i)?.celsius
        if (wind == null && humidity == null && dew == null) return@mapIndexedNotNull null
        time to AtmosphereReading(time, wind, humidity, dew)
    }.toMap()
}

internal fun List<HourlyWrapper>.withAtmosphere(model: Map<Date, AtmosphereReading>) = map { hour ->
    model[hour.date]?.let { air ->
        hour.copy(wind = air.wind, relativeHumidity = air.humidity, dewPoint = air.dewPoint)
    } ?: hour
}

private fun distanceMeters(location: Location, latitude: Double, longitude: Double): Double =
    SphericalUtil.computeDistanceBetween(LatLng(location.latitude, location.longitude), LatLng(latitude, longitude))

internal fun nearbyObservation(
    location: Location,
    groups: Map<String, List<PagasaCurrentResult>>,
    now: Long,
): AtmosphereReading? {
    val stamp = SimpleDateFormat("MMMM d, yyyy, h:mm a", Locale.ENGLISH).apply {
        timeZone = TimeZone.getTimeZone("Asia/Manila")
        isLenient = false
    }
    var nearest = 25_000.0 // Do not substitute a remote island's observation.
    var selected: AtmosphereReading? = null
    groups.values.forEach { group ->
        for (row in group) {
            val lat = row.latitude.toDoubleOrNull()?.takeIf { it in -90.0..90.0 } ?: continue
            val lon = row.longitude.toDoubleOrNull()?.takeIf { it in -180.0..180.0 } ?: continue
            val distance = distanceMeters(location, lat, lon)
            if (distance > nearest) continue
            val pos = ParsePosition(0)
            val time = stamp.parse(row.datetime, pos) ?: continue
            if (pos.index != row.datetime.length || now - time.time !in 0..2.hours.inWholeMilliseconds) continue
            if (distance == nearest && selected?.time?.let { it >= time } == true) continue
            val name = row.siteName?.takeIf { it.isNotBlank() } ?: continue
            val humidity = row.humidity?.substringBefore(" ")?.toDoubleOrNull()?.takeIf { it in 1.0..100.0 }?.percent ?: continue
            val speed = row.windSpeed?.substringBefore(" ")?.toDoubleOrNull()?.takeIf { it >= 0 }?.kilometersPerHour ?: continue
            val temperature = row.temperature?.substringBefore(" ")?.toDoubleOrNull()?.takeIf { it in -70.0..65.0 }?.celsius ?: continue
            selected = AtmosphereReading(
                time, Wind(degree = getWindDegree(row.windDirection), speed = speed), humidity,
                computeDewPoint(temperature, humidity), station = name
            )
            nearest = distance
        }
    }
    return selected
}
