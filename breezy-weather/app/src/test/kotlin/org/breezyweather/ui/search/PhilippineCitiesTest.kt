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
        findOfflinePhilippineCities("", Locale.CHINA).map { it.cityId } shouldBe
            listOf("panahon-manila", "panahon-cebu", "panahon-davao", "panahon-boracay")
        findOfflinePhilippineCities("Tokyo", Locale.ENGLISH).size shouldBe 0
    }

    @Test
    fun boracayAliasesSelectTheSameIslandWithoutNetworkAccess() {
        for (query in listOf("长滩岛", "長灘島", "Boracay", "boracay island")) {
            val result = findOfflinePhilippineCities(query, Locale.CHINA).single()
            result.cityId shouldBe "panahon-boracay"
            result.city shouldBe "长滩岛"
            result.latitude shouldBe 11.98013
            result.longitude shouldBe 121.91932
            result.countryCode shouldBe "PH"
            result.timeZone.id shouldBe "Asia/Manila"
        }
    }

    @Test
    fun onlineSearchKeepsOneBoracayResultAndOtherPlaces() {
        val offline = findOfflinePhilippineCities("长滩岛", Locale.CHINA).single()
        val sameIsland = offline.copy(cityId = "remote-island", city = "Boracay Island")
        val anotherPlace = sameIsland.copy(cityId = "another", longitude = 122.0)
        mergeSearchLocations(listOf(offline), listOf(sameIsland, anotherPlace)) shouldBe listOf(offline, anotherPlace)
    }
}
