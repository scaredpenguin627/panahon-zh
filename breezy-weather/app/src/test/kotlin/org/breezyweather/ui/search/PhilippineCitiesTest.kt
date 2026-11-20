package org.breezyweather.ui.search

import io.kotest.matchers.shouldBe
import org.junit.jupiter.api.Test
import java.util.Locale

class PhilippineCitiesTest {
    @Test
    fun searchesEnglishAndChineseWithoutANetworkProvider() {
        val pairs = listOf("Manila" to "马尼拉", "Cebu" to "宿务", "Davao" to "达沃")
        for ((english, chinese) in pairs) {
            val location = findOfflinePhilippineCities("  ${english.uppercase()}  ", Locale.CHINA).single()
            location.city shouldBe chinese
            location.countryCode shouldBe "PH"
            location.timeZone.id shouldBe "Asia/Manila"
            val englishLocation = findOfflinePhilippineCities(chinese, Locale.ENGLISH).single()
            englishLocation.formattedId shouldBe location.formattedId
            englishLocation.city.startsWith(english) shouldBe true
        }
        findOfflinePhilippineCities("", Locale.CHINA).size shouldBe 3
        findOfflinePhilippineCities("Tokyo", Locale.ENGLISH).size shouldBe 0
    }
}
