package org.breezyweather.ui.main.adapters.main.holder

import android.content.ActivityNotFoundException
import android.content.Intent
import android.view.LayoutInflater
import android.view.ViewGroup
import android.widget.LinearLayout
import android.widget.TextView
import com.google.android.material.button.MaterialButton
import androidx.core.net.toUri
import androidx.core.view.isVisible
import androidx.lifecycle.lifecycleScope
import breezyweather.domain.location.model.Location
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import org.breezyweather.R
import org.breezyweather.common.activities.BreezyActivity
import org.breezyweather.domain.weather.model.validAirQuality
import org.breezyweather.sources.emb.EmbLocalReport
import org.breezyweather.sources.emb.isOldReport
import org.breezyweather.sources.emb.hasRecentMonitoring
import org.breezyweather.ui.main.MainActivity
import org.breezyweather.ui.theme.resource.providers.ResourceProvider
import java.text.DateFormat
import java.text.NumberFormat
import java.util.Date

class EmbViewHolder(parent: ViewGroup) : AbstractMainCardViewHolder(
    LayoutInflater.from(parent.context).inflate(R.layout.card_emb_air, parent, false)
) {
    private val readings: TextView = itemView.findViewById(R.id.emb_readings)
    private val model: TextView = itemView.findViewById(R.id.emb_model_source)
    private val links: LinearLayout = itemView.findViewById(R.id.emb_official_pages)
    private var update: Job? = null

    override fun onBindView(
        activity: BreezyActivity,
        location: Location,
        provider: ResourceProvider,
        listAnimationEnabled: Boolean,
        itemAnimationEnabled: Boolean,
    ) {
        super.onBindView(activity, location, provider, listAnimationEnabled, itemAnimationEnabled)
        val host = activity as MainActivity
        update?.cancel()
        val sourceId = location.airQualitySource ?: location.forecastSource
        val source = host.sourceManager.getWeatherSource(sourceId)
        model.isVisible = location.weather?.validAirQuality != null
        model.text = context.getString(if (sourceId == "openmeteo") R.string.emb_model else R.string.emb_other_source,
            source?.name ?: sourceId)
        readings.setText(R.string.emb_loading)
        links.removeAllViews()
        update = host.lifecycleScope.launch {
            val reports = host.embRepository.forLocation(location)
            val now = System.currentTimeMillis()
            val entries = reports.map { format(it) }.toMutableList()
            if (reports.isEmpty()) entries += context.getString(R.string.emb_uncovered)
            else if (reports.none { it.hasRecentMonitoring(now) } && reports.any { it.snapshot?.stations?.isNotEmpty() == true }) {
                entries.add(0, context.getString(R.string.emb_no_recent))
            }
            readings.text = entries.joinToString("\n\n")
            val pages = reports.distinctBy { it.feed.page }
            if (pages.isEmpty()) addLink("https://air.emb.gov.ph/national-air-quality-data/", "")
            else pages.forEach {
                val pollutants = it.feed.tables.flatMap { table -> table.columns.map { col -> col.label } }.distinct()
                addLink(it.feed.page, if (pages.size > 1) pollutants.joinToString(" / ") else "")
            }
        }
    }

    private fun addLink(uri: String, pollutants: String) {
        val button = MaterialButton(context, null, androidx.appcompat.R.attr.borderlessButtonStyle).apply {
            text = listOf(context.getString(R.string.emb_official_link), pollutants).filter { it.isNotEmpty() }.joinToString(" · ")
            setOnClickListener {
                try {
                    context.startActivity(Intent(Intent.ACTION_VIEW, uri.toUri()))
                } catch (_: ActivityNotFoundException) {
                    android.widget.Toast.makeText(context, uri, android.widget.Toast.LENGTH_LONG).show()
                }
            }
        }
        links.addView(button)
    }

    private fun format(report: EmbLocalReport): String {
        val snapshot = report.snapshot ?: return context.getString(R.string.emb_error)
        if (snapshot.stations.isEmpty()) return context.getString(R.string.emb_missing)
        val lines = mutableListOf<String>()
        when (report.feed.kind) {
            "manual" -> lines += context.getString(R.string.emb_manual)
            "annual" -> lines += context.getString(R.string.emb_annual)
            "historical" -> lines += context.getString(R.string.emb_historical)
        }
        if (report.cachedOnError) lines += context.getString(R.string.emb_cached)
        val now = System.currentTimeMillis()
        fun clock(label: Int, time: String): String {
            val text = context.getString(label, time)
            return if (isOldReport(time, now)) "$text · ${context.getString(R.string.emb_old)}" else text
        }
        snapshot.reported?.let { lines += clock(R.string.emb_report_time, it) }
        val number = NumberFormat.getNumberInstance()
        for (site in snapshot.stations) {
            val station = report.feed.sites.find { it.station == site.name }
            val place = listOfNotNull(station?.displayName ?: site.name, station?.city, site.address).distinct()
            lines += place.joinToString(" · ")
            site.observed?.let { lines += clock(if (report.feed.kind == "annual") R.string.emb_year else R.string.emb_station_time, it) }
            for (measurement in site.readings) {
                val value = number.format(measurement.value)
                val text = if (measurement.unit == "AQI") context.getString(R.string.emb_sub_index, measurement.name, value)
                    else "${measurement.name} $value ${measurement.unit}"
                lines += listOfNotNull(text, measurement.rating).joinToString(" · ")
            }
        }
        if ((snapshot.reported == null && snapshot.stations.any { it.observed == null }) || report.cachedOnError) {
            lines += context.getString(R.string.emb_fetched, DateFormat.getDateTimeInstance().format(Date(snapshot.fetchedAt)))
        }
        return lines.joinToString("\n")
    }

    override fun onRecycleView() {
        super.onRecycleView()
        update?.cancel()
        update = null
    }
}
