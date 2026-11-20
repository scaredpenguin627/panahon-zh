package org.breezyweather.sources.emb

import android.content.Context
import breezyweather.domain.location.model.Location
import dagger.hilt.EntryPoint
import dagger.hilt.InstallIn
import dagger.hilt.android.EntryPointAccessors
import dagger.hilt.components.SingletonComponent

@EntryPoint
@InstallIn(SingletonComponent::class)
internal interface EmbWidgetEntryPoint {
    fun embRepository(): EmbRepository
}

internal fun Context.embWidgetReading(location: Location?): String? {
    if (location == null || location.airQualitySource != EmbAirQualityService.ID) return null
    val repository = EntryPointAccessors.fromApplication(applicationContext, EmbWidgetEntryPoint::class.java).embRepository()
    return repository.currentWidgetReading(location, System.currentTimeMillis())
}
