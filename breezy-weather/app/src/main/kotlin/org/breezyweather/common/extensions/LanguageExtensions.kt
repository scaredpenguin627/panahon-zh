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

package org.breezyweather.common.extensions

import android.content.Context
import android.content.res.Configuration
import androidx.appcompat.app.AppCompatDelegate
import java.util.Locale

val Context.currentLocale: Locale
    get() {
        val selected = AppCompatDelegate.getApplicationLocales().get(0)?.supportedAppLocale
        if (selected != null) return selected
        val locales = resources.configuration.locales
        return (0 until locales.size()).firstNotNullOfOrNull { locales[it].supportedAppLocale } ?: Locale.ENGLISH
    }

internal val Locale.supportedAppLocale: Locale?
    get() = when (language) {
        "en" -> this
        "zh" -> if (isTraditionalChinese) Locale.TRADITIONAL_CHINESE else Locale.SIMPLIFIED_CHINESE
        else -> null
    }

internal val Locale.appLanguageTag: String?
    get() = if (language == "en") "en" else supportedAppLocale?.toLanguageTag()

internal fun initialAppLanguageTag(stored: List<Locale>, firstLaunch: Boolean): String {
    if (stored.isEmpty() && firstLaunch) return "zh-CN"
    // Use the first supported preference, or let Android follow the system languages.
    return stored.firstNotNullOfOrNull { it.appLanguageTag }.orEmpty()
}

// TODO: Review this use vs toLanguageTag()
val Locale.code: String
    get() {
        val language = language
        val country = country
        return if (isTraditionalChinese) {
            "zh-" + country.ifEmpty { "TW" }.lowercase()
        } else {
            language.lowercase()
        }
    }

// TODO: Review this use vs toLanguageTag()
val Locale.codeWithCountry: String
    get() {
        val language = language
        val country = country
        return if (!country.isNullOrEmpty()) {
            language.lowercase() + "-" + country.lowercase()
        } else {
            language.lowercase()
        }
    }

// Accepts "Hant" for traditional chinese but no country code otherwise
val Locale.codeForGeonames: String
    get() {
        return if (isTraditionalChinese) {
            language.lowercase() + "-Hant"
        } else {
            language.lowercase()
        }
    }

// Everything in uppercase + "ZHT" for traditional Chinese
val Locale.codeForNaturalEarth: String
    get() {
        return if (isTraditionalChinese) {
            language.uppercase() + "T"
        } else {
            language.uppercase()
        }
    }

val Locale.isChinese: Boolean
    get() = language.equals("zh", ignoreCase = true)

val Locale.isTraditionalChinese: Boolean
    get() = isChinese && when (script) {
        "Hant" -> true
        "Hans" -> false
        else -> country == "TW" || country == "HK" || country == "MO"
    }

fun Locale.getCountryName(countryCode: String): String {
    return Locale.Builder()
        .setLanguage(language)
        .setRegion(countryCode)
        .build()
        .getDisplayCountry(this)
}

/**
 * Replaces the given string to have at most [count] characters using [replacement] at its end.
 * If [replacement] is longer than [count] an exception will be thrown when `length > count`.
 */
fun String.chop(count: Int, replacement: String = "…"): String {
    return if (length > count) {
        take(count - replacement.length) + replacement
    } else {
        this
    }
}

fun String.capitalize(locale: Locale = Locale.Builder().setLanguage("en").setRegion("001").build()): String {
    return replaceFirstChar { firstChar ->
        if (firstChar.isLowerCase()) {
            firstChar.titlecase(locale)
        } else {
            firstChar.toString()
        }
    }
}

fun String.uncapitalize(locale: Locale = Locale.Builder().setLanguage("en").setRegion("001").build()): String {
    return replaceFirstChar { firstChar ->
        if (firstChar.isUpperCase()) {
            firstChar.lowercase(locale)
        } else {
            firstChar.toString()
        }
    }
}

fun Context.getStringByLocale(
    id: Int,
    locale: Locale = Locale.Builder().setLanguage("en").setRegion("001").build(),
): String {
    val configuration = Configuration(resources.configuration)
    configuration.setLocale(locale)
    return createConfigurationContext(configuration).resources.getString(id)
}
