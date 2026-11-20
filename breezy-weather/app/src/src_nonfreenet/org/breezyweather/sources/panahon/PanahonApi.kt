package org.breezyweather.sources.panahon

import io.reactivex.rxjava3.core.Observable
import kotlinx.serialization.Serializable
import retrofit2.http.GET
import retrofit2.http.Query

interface PanahonApi {
    @GET("nowcast/dashboard")
    fun dashboard(@Query("lat") lat: Double, @Query("lon") lon: Double): Observable<PanahonDashboard>

    @GET("alerts")
    fun alerts(@Query("lat") lat: Double, @Query("lon") lon: Double): Observable<PanahonAlerts>
}

@Serializable
data class PanahonDashboard(val success: Boolean, val meta: PanahonArea? = null, val data: PanahonForecast)

@Serializable
data class PanahonArea(
    val lat: String? = null,
    val lon: String? = null,
    val barangay: String? = null,
    val municipality: String? = null,
    val province: String? = null,
    val region: String? = null,
)

@Serializable
data class PanahonForecast(
    val nc: PanahonPeriod? = null,
    val hc: List<PanahonPeriod> = emptyList(),
    val dc: List<PanahonPeriod> = emptyList(),
)

@Serializable
data class PanahonPeriod(
    val time: String? = null,
    val dt: String? = null,
    val desc: String? = null,
    val tmp: String? = null,
    val tmax: String? = null,
    val tmin: String? = null,
    val tprecip: String? = null,
    val hindex: String? = null,
)

@Serializable
data class PanahonAlerts(val success: Boolean, val data: List<PanahonBulletin>)

@Serializable
data class PanahonBulletin(
    val alert_id: String,
    val title: String,
    val message: String? = null,
    val issued_at_timestamp: Long? = null,
    val valid_until_at: String? = null,
    val locations: List<String>? = null,
)
