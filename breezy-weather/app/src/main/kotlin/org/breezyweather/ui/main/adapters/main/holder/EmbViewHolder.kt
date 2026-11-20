package org.breezyweather.ui.main.adapters.main.holder

import android.content.ActivityNotFoundException
import android.content.Intent
import android.view.LayoutInflater
import android.view.ViewGroup
import android.widget.LinearLayout
import android.widget.TextView
import com.google.android.material.button.MaterialButton
import androidx.core.view.isVisible
import androidx.core.net.toUri
import androidx.lifecycle.lifecycleScope
import breezyweather.domain.location.model.Location
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import org.breezyweather.R
import org.breezyweather.common.activities.BreezyActivity
import org.breezyweather.domain.weather.model.validAirQuality
import org.breezyweather.sources.emb.EmbLocalReport
import org.breezyweather.sources.emb.reportAgeMillis
import org.breezyweather.sources.emb.hasRecentMonitoring
import org.breezyweather.ui.main.MainActivity
import org.breezyweather.ui.theme.resource.providers.ResourceProvider
import java.text.DateFormat
import java.text.NumberFormat
import java.util.Date

class EmbViewHolder(parent: ViewGroup) : AbstractMainCardViewHolder(
    LayoutInflater.from(parent.context).inflate(R.layout.card_emb_air, parent, false)
) {
    private val summary: MaterialButton = itemView.findViewById(R.id.emb_summary)
    private val details: LinearLayout = itemView.findViewById(R.id.emb_details)
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
        details.isVisible = false
        fun setExpanded(expanded: Boolean) {
            details.isVisible = expanded
            summary.setIconResource(if (expanded) R.drawable.ic_chevron_up else R.drawable.ic_chevron_down)
            val action = if (expanded) R.string.traveller_emb_hide else R.string.traveller_emb_show
            summary.contentDescription = listOf(summary.text, context.getString(action)).joinToString(", ")
        }
        summary.setOnClickListener { setExpanded(!details.isVisible) }
        summary.setText(R.string.emb_loading)
        setExpanded(false)
        val updated = location.weather?.base?.airQualityUpdateTime
        model.text = when {
            location.weather?.validAirQuality == null -> context.getString(R.string.emb_model_missing)
            updated == null -> context.getString(R.string.emb_model)
            else -> context.getString(R.string.emb_model_updated, DateFormat.getDateTimeInstance().format(updated))
        }
        model.text = listOf(model.text, context.getString(R.string.traveller_emb_scale)).joinToString("\n")
        readings.setText(R.string.emb_loading)
        links.removeAllViews()
        update = host.lifecycleScope.launch {
            val reports = try {
                host.embRepository.forLocation(location)
            } catch (e: CancellationException) {
                throw e
            } catch (_: Exception) {
                summary.text = "EMB · " + context.getString(R.string.emb_error)
                setExpanded(details.isVisible)
                readings.setText(R.string.emb_error)
                addLink("https://air.emb.gov.ph/national-air-quality-data/", "")
                return@launch
            }
            val now = System.currentTimeMillis()
            // Only the latest local automated reading belongs in the collapsed summary.
            val station = reports.asSequence().filter { it.feed.kind == "station" }.flatMap { report ->
                report.snapshot?.stations.orEmpty().asSequence().map { site -> report to site }
            }.filter { (report, site) -> site.readings.isNotEmpty() && reportAgeMillis(site.observed ?: report.snapshot?.reported, now) != null }
                .maxByOrNull { (report, site) -> site.observed ?: report.snapshot?.reported.orEmpty() }
            summary.text = when {
                station != null -> {
                    val (report, site) = station
                    val observed = site.observed
                    val time = observed ?: report.snapshot?.reported.orEmpty()
                    val name = report.feed.sites.find { it.station == site.name }?.displayName ?: site.name
                    val reading = site.readings.firstOrNull { it.name == "PM2.5" } ?: site.readings.first()
                    val value = NumberFormat.getNumberInstance().format(reading.value)
                    val measure = if (reading.unit == "AQI") context.getString(R.string.emb_sub_index, reading.name, value)
                        else listOf(reading.name, value, reading.unit).joinToString(" ")
                    listOfNotNull("EMB", name, measure,
                        context.getString(if (observed == null) R.string.emb_report_time else R.string.emb_station_time, time),
                        ageLabel(time, now),
                        context.getString(R.string.emb_cached).takeIf { report.cachedOnError }).joinToString(" · ")
                }
                reports.isEmpty() -> context.getString(R.string.traveller_emb_none)
                reports.all { it.unavailable } -> "EMB · " + context.getString(R.string.emb_error)
                else -> "EMB · " + context.getString(R.string.emb_no_recent)
            }
            setExpanded(details.isVisible)
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

    private fun ageLabel(time: String, now: Long): String? {
        val days = reportAgeMillis(time, now)?.div(86_400_000L)?.toInt() ?: return null
        return if (days < 1) null else context.resources.getQuantityString(R.plurals.traveller_days_ago, days, days)
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
            return listOfNotNull(text, ageLabel(time, now)).joinToString(" · ")
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
