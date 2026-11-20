package org.breezyweather.ui.main

import android.app.Activity
import android.content.ActivityNotFoundException
import android.net.Uri
import breezyweather.domain.location.model.Location
import io.kotest.matchers.shouldBe
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import org.breezyweather.common.utils.helpers.IntentHelper
import org.junit.jupiter.api.Test

class LocationIntentTest {
    @Test
    fun missingGeoHandlerTriggersTheUnavailableMessage() {
        val activity = mockk<Activity>()
        val location = Location(latitude = 11.98013, longitude = 121.91932)
        mockkStatic(Uri::class)
        try {
            every { Uri.parse("geo:11.98013,121.91932") } returns mockk()
            every { activity.startActivity(any()) } throws ActivityNotFoundException()
            IntentHelper.openLocationInOtherApp(activity, location) shouldBe false
        } finally {
            unmockkStatic(Uri::class)
        }
    }
}
