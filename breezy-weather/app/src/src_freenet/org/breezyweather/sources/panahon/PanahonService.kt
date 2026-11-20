package org.breezyweather.sources.panahon

import android.content.Context
import breezyweather.domain.location.model.Location
import breezyweather.domain.source.SourceFeature
import breezyweather.domain.weather.wrappers.WeatherWrapper
import io.reactivex.rxjava3.core.Observable
import org.breezyweather.common.exceptions.NonFreeNetSourceException
import javax.inject.Inject

class PanahonService @Inject constructor() : PanahonSource() {
    override fun requestWeather(context: Context, location: Location, requestedFeatures: List<SourceFeature>):
        Observable<WeatherWrapper> = Observable.error(NonFreeNetSourceException())
}
