package org.breezyweather.option.utils

import org.breezyweather.common.extensions.initialAppLanguageTag
import org.breezyweather.common.extensions.supportedAppLocale
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import java.util.Locale

class AppLanguageTest {
    @Test
    fun preservesSystemModeAndResolvesTheFirstSupportedRestoredLanguage() {
        assertEquals("zh-CN", initialAppLanguageTag(emptyList(), firstLaunch = true))
        assertEquals("", initialAppLanguageTag(emptyList(), firstLaunch = false))
        assertEquals("", initialAppLanguageTag(listOf(Locale.FRENCH), firstLaunch = false))
        assertEquals("en", initialAppLanguageTag(listOf(Locale.FRENCH, Locale.UK), firstLaunch = false))
        assertEquals(Locale.UK, Locale.UK.supportedAppLocale)
        assertEquals(null, Locale.JAPANESE.supportedAppLocale)
    }

    @Test
    fun mapsChineseScriptBeforeRegionWhenRestoringTheLanguage() {
        val traditional = listOf("zh-Hant", "zh-HK", "zh-MO", "zh-TW", "zh-Hant-CN")
        val simplified = listOf("zh", "zh-CN", "zh-SG", "zh-Hans", "zh-Hans-HK")
        for (tag in traditional + simplified) {
            val locale = Locale.forLanguageTag(tag)
            val expected = if (tag in traditional) "zh-TW" else "zh-CN"
            assertEquals(expected, initialAppLanguageTag(listOf(locale), firstLaunch = false), tag)
            assertEquals(expected, locale.supportedAppLocale?.toLanguageTag(), tag)
        }
    }
}
