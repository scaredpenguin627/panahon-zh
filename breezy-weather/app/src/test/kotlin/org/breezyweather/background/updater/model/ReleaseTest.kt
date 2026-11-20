package org.breezyweather.background.updater.model

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

class ReleaseTest {
    private val page = "https://github.com/scaredpenguin627/panahon-zh/releases/tag/v1.0.0"
    private val files = page.replace("/tag/", "/download/")
    private val universal = "$files/panahon-zh-universal-v1.0.0.apk"
    private val x86 = "$files/panahon-zh-x86-v1.0.0.apk"
    private val arm64 = "$files/panahon-zh-arm64-v8a-v1.0.0.apk"
    private val mixedAssets = listOf("$files/SHA256SUMS", universal, x86, arm64)

    @Test
    fun respectsAbiOrderAndIgnoresOtherFiles() {
        val release = Release("v1.0.0", "", page, mixedAssets)
        assertEquals(arm64, release.getDownloadLink(listOf("arm64-v8a", "armeabi-v7a")))
        assertEquals(x86, release.getDownloadLink(listOf("x86_64", "x86")))
    }

    @Test
    fun prefersUniversalForUnknownAbi() {
        val release = Release("v1.0.0", "", page, mixedAssets)
        assertEquals(universal, release.getDownloadLink(listOf("riscv64")))
    }

    @Test
    fun linksToReleaseWhenNoSuitableAssetExists() {
        for (assets in listOf(emptyList(), listOf(x86, "$arm64.sha256", "$files/SHA256SUMS"))) {
            assertEquals(page, Release("v1.0.0", "", page, assets).getDownloadLink(listOf("arm64-v8a")))
        }
    }
}
