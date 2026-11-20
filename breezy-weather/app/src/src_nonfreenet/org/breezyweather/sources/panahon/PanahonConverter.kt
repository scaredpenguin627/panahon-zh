package org.breezyweather.sources.panahon

import breezyweather.domain.weather.model.Alert
import breezyweather.domain.weather.model.Precipitation
import breezyweather.domain.weather.reference.AlertSeverity
import breezyweather.domain.weather.reference.WeatherCode
import breezyweather.domain.weather.wrappers.CurrentWrapper
import breezyweather.domain.weather.wrappers.DailyWrapper
import breezyweather.domain.weather.wrappers.HalfDayWrapper
import breezyweather.domain.weather.wrappers.HourlyWrapper
import breezyweather.domain.weather.wrappers.TemperatureWrapper
import org.breezyweather.common.exceptions.InvalidOrIncompleteDataException
import org.breezyweather.common.exceptions.OutdatedServerDataException
import org.breezyweather.unit.precipitation.Precipitation.Companion.millimeters
import org.breezyweather.unit.temperature.Temperature.Companion.celsius
import java.text.ParsePosition
import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Date
import java.util.Locale
import java.util.TimeZone
import kotlin.math.abs
import kotlin.time.Duration.Companion.days
import kotlin.time.Duration.Companion.hours

private val manila = TimeZone.getTimeZone("Asia/Manila")
// The server silently returns Manila when it does not recognize the coordinate parameters.
internal fun PanahonArea.matches(latitude: Double, longitude: Double): Boolean {
    val returnedLat = lat?.toDoubleOrNull() ?: return false
    val returnedLon = lon?.toDoubleOrNull() ?: return false
    return abs(latitude - returnedLat) < 0.01 && abs(longitude - returnedLon) < 0.01
}

private val numericPrefix = Regex("""^[+-]?\d+(?:\.\d+)?""")
private fun String?.number() = this?.trim()?.let { numericPrefix.find(it)?.value?.toDoubleOrNull() }
private fun formatter(pattern: String) = SimpleDateFormat(pattern, Locale.US).apply {
    timeZone = manila
    isLenient = false
}

private fun SimpleDateFormat.read(value: String?): Date? {
    if (value == null) return null
    val pos = ParsePosition(0)
    return parse(value, pos)?.takeIf { pos.index == value.length }
}

// Order matters for descriptions such as "cloudy skies with rainshowers and thunderstorms".
private fun weatherCode(description: String?): WeatherCode? {
    val text = description?.lowercase(Locale.ROOT) ?: return null
    return when {
        "thunder" in text -> WeatherCode.THUNDERSTORM
        "rain" in text || "shower" in text -> WeatherCode.RAIN
        "partly cloudy" in text -> WeatherCode.PARTLY_CLOUDY
        "cloudy" in text || "overcast" in text -> WeatherCode.CLOUDY
        "sunny" in text || "clear" in text || "fair" in text -> WeatherCode.CLEAR
        else -> null
    }
}

private fun nearestDay(value: String?, now: Long): Date? {
    val year = Calendar.getInstance(manila).apply { timeInMillis = now }.get(Calendar.YEAR)
    val stamp = formatter("yyyy/MM/dd")
    return (year - 1..year + 1).mapNotNull { stamp.read("$it/$value") }.minByOrNull { abs(it.time - now) }
}

private fun PanahonForecast.anchorDate(now: Long): Date {
    val today = nearestDay(dc.firstOrNull()?.dt, now) ?: throw InvalidOrIncompleteDataException()
    if (now - today.time !in 0 until 2.days.inWholeMilliseconds) throw OutdatedServerDataException()
    return today
}

// At midnight dc[0] can already be today while nc and hc[0] still start late yesterday.
private fun PanahonForecast.firstHour(time: String?, now: Long): Date {
    val date = formatter("yyyy-MM-dd").format(anchorDate(now))
    val stamp = formatter("yyyy-MM-dd h:mm a").read("$date $time") ?: throw InvalidOrIncompleteDataException()
    return if (stamp.time - now > 12.hours.inWholeMilliseconds) Date(stamp.time - 1.days.inWholeMilliseconds) else stamp
}

internal fun PanahonForecast.toCurrent(now: Long, describe: (String) -> String = { it }): CurrentWrapper {
    val current = nc ?: throw InvalidOrIncompleteDataException()
    val issued = firstHour(current.time, now)
    if (now - issued.time !in 0..2.hours.inWholeMilliseconds) throw OutdatedServerDataException()
    val temp = current.tmp.number() ?: throw InvalidOrIncompleteDataException()
    val code = weatherCode(current.desc)
    return CurrentWrapper(
        weatherCode = code,
        weatherText = current.desc?.let(describe),
        temperature = TemperatureWrapper(temp.celsius, current.hindex.number()?.celsius)
    )
}

internal fun PanahonForecast.toForecast(now: Long, describe: (String) -> String = { it }): Pair<List<HourlyWrapper>, List<DailyWrapper>> {
    val daily = dc.mapNotNull { item ->
        val date = nearestDay(item.dt, now) ?: return@mapNotNull null
        val code = weatherCode(item.desc)
        // The bulletin contains one all-day outlook and a pair of extrema.
        DailyWrapper(
            date = date,
            day = HalfDayWrapper(
                weatherCode = code, weatherText = item.desc?.let(describe),
                temperature = TemperatureWrapper(item.tmax.number()?.celsius)
            ),
            night = HalfDayWrapper(temperature = TemperatureWrapper(item.tmin.number()?.celsius))
        )
    }

    val clock = formatter("h:mm a")
    val hourDay = Calendar.getInstance(manila).apply { time = firstHour(hc.firstOrNull()?.time, now) }
    var previousMinute = -1
    val hourly = hc.mapNotNull { item ->
        val parsed = clock.read(item.time) ?: return@mapNotNull null
        val hhmm = Calendar.getInstance(manila).apply { time = parsed }
        val hour = hhmm.get(Calendar.HOUR_OF_DAY)
        val minute = hhmm.get(Calendar.MINUTE)
        val minuteOfDay = 60 * hour + minute
        if (minuteOfDay < previousMinute) hourDay.add(Calendar.DATE, 1)
        if (minuteOfDay == previousMinute) return@mapNotNull null
        previousMinute = minuteOfDay
        hourDay.set(Calendar.HOUR_OF_DAY, hour)
        hourDay.set(Calendar.MINUTE, minute)
        val code = weatherCode(item.desc)
        HourlyWrapper(
            date = hourDay.time, weatherCode = code, weatherText = item.desc?.let(describe),
            temperature = TemperatureWrapper(item.tmp.number()?.celsius, item.hindex.number()?.celsius),
            precipitation = item.tprecip.number()?.let { Precipitation(total = it.millimeters) }
        )
    }
    if (daily.isEmpty() || hourly.none { it.date.time + 1.hours.inWholeMilliseconds > now }) {
        throw InvalidOrIncompleteDataException()
    }
    return hourly to daily
}

internal fun PanahonArea.alertAreas(): List<String> {
    // The coordinates endpoint returns national bulletins. Use official English area names from the dashboard.
    // An unqualified town (e.g. Rizal) can share a name with a distant province.
    val city = municipality?.takeIf { it.endsWith(" City", ignoreCase = true) || it.equals("Manila", ignoreCase = true) }
    return listOfNotNull(city, province, region).map { it.trim() }.filter { it.isNotEmpty() }.distinctBy { it.lowercase(Locale.ROOT) }
}

internal fun PanahonAlerts.toAlerts(area: PanahonArea, now: Long): List<Alert> {
    val areas = area.alertAreas().map { it.lowercase(Locale.ROOT) }
    // Some offshore coordinates have verified dashboard coordinates but no named
    // province or region. Only explicitly nationwide bulletins can be matched there.
    val nationwide = setOf("all", "nationwide", "philippines", "entire philippines")
    val relevantAreas = areas + nationwide
    val stamp = formatter("MMMM d, yyyy hh:mm a")
    return data.mapNotNull { item ->
        // PAGASA can leave the locations list empty on a final advisory. The issuing
        // office or a national feed alone does not establish that it covers this location.
        if (item.locations.orEmpty().none { it.isNotBlank() && it.trim().lowercase(Locale.ROOT) in relevantAreas }) return@mapNotNull null
        val end = stamp.read(item.valid_until_at)
        if (end == null || item.alert_id.isBlank()) throw InvalidOrIncompleteDataException()
        if (end.time <= now) return@mapNotNull null
        Alert(
            alertId = item.alert_id,
            startDate = item.issued_at_timestamp?.let { Date(it * 1000L) },
            endDate = end, headline = item.title, description = item.message, source = "PAGASA",
            color = Alert.colorFromSeverity(AlertSeverity.UNKNOWN)
        )
    }
}
