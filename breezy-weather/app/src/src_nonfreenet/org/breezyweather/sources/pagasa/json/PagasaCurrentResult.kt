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

package org.breezyweather.sources.pagasa.json

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonPrimitive
import org.breezyweather.unit.speed.Speed.Companion.kilometersPerHour

@Serializable
data class PagasaCurrentResult(
    val datetime: String,
    val temperature: String?,
    val humidity: String?,
    val pressure: JsonPrimitive?,
    @SerialName("wind_speed") val windSpeed: String?,
    @SerialName("wind_direction") val windDirection: String?,
    val latitude: String,
    val longitude: String,
    @SerialName("site_name") val siteName: String? = null,
) {
    val observedWindSpeed get() = windSpeed?.trim()
        ?.let { windKph.matchEntire(it)?.groupValues?.get(1)?.toDoubleOrNull() }
        ?.takeIf { it.isFinite() }
        ?.kilometersPerHour
}

private val windKph = Regex("""(\d+(?:\.\d+)?)\s*km/(?:h|hr)""", RegexOption.IGNORE_CASE)
