package org.breezyweather.sources.emb

import android.content.Context
import android.util.Base64
import android.util.Log
import breezyweather.domain.location.model.Location
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.*
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.io.IOException
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.TimeUnit
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class EmbRepository @Inject constructor(
    @ApplicationContext private val context: Context,
    httpClient: OkHttpClient,
) {
    private val client = httpClient.newBuilder().callTimeout(10, TimeUnit.SECONDS).build()
    private val prefs by lazy { context.getSharedPreferences("emb-public-reports-v1", Context.MODE_PRIVATE) }
    private val latest = ConcurrentHashMap<String, EmbSnapshot>()
    private val locks = ConcurrentHashMap<String, Mutex>()
    private val feedList by lazy {
        context.assets.list("emb").orEmpty().filter { it == "ncr.json" || it.endsWith(".report.json") }.sorted().mapNotNull { file ->
            runCatching { Json.decodeFromString<EmbFeed>(asset(file)) }.onFailure {
                Log.w("EMB", "Invalid bundled monitoring report: $file", it)
            }.getOrNull()
        }
    }

    internal suspend fun forLocation(location: Location): List<EmbLocalReport> = withContext(Dispatchers.IO) {
        if (!location.countryCode.equals("PH", ignoreCase = true)) return@withContext emptyList()
        val names = listOfNotNull(location.city, location.district, location.admin3, location.admin4)
            .map(::normalizeCity).filter { it.isNotEmpty() }.toSet()
        feedList.mapNotNull { feed ->
            val ids = feed.sites.filter { it.matches(names, location.latitude, location.longitude) }.map { it.station }.toSet()
            if (ids.isEmpty()) return@mapNotNull null
            async { loadReport(feed, ids) }
        }.map { it.await() }
    }

    private suspend fun loadReport(feed: EmbFeed, ids: Set<String>): EmbLocalReport {
        val feedKey = feed.key + ":" + feed.kind
        val lock = locks.getOrPut(feedKey) { Mutex() }
        return lock.withLock {
            val localKey = feedKey + ":" + ids.sorted().joinToString("|")
            val tables = feed.tables.filter { it.fixedStation == null || it.fixedStation in ids }
            val wholeReportKey = if (feed.key == "emb-r3" || tables.size != feed.tables.size) localKey else feedKey
            val saved = prefs.getString(localKey, null)?.let {
                runCatching { Json.decodeFromString<EmbSnapshot>(it) }.getOrNull()
            }
            val now = System.currentTimeMillis()
            if (saved != null && now - saved.fetchedAt in 0 until 600_000L) {
                return@withLock EmbLocalReport(feed, saved)
            }
            try {
                val report = latest[wholeReportKey]?.takeIf { now - it.fetchedAt in 0 until 600_000L }
                    ?: when (feed.key) {
                        "emb-r3" -> fetchCentralLuzon(ids)
                        "emb-r10" -> {
                            val payload = publicJson("https://api.bpit-inc.com/embrx/l").jsonArray
                            EmbSnapshot(now, stations = northernMindanaoStations(payload))
                        }
                        else -> fetch(feed.copy(tables = tables))
                    }.also { latest[wholeReportKey] = it }
                val selected = selectEmbReport(feed, report, saved, ids)
                if (!selected.cachedOnError && selected.snapshot?.stations?.isNotEmpty() == true) {
                    prefs.edit().putString(localKey, Json.encodeToString(selected.snapshot)).apply()
                }
                selected
            } catch (e: CancellationException) {
                throw e
            } catch (_: Exception) {
                selectEmbReport(feed, null, saved, ids)
            }
        }
    }

    private fun asset(name: String): String =
        context.assets.open("emb/$name").bufferedReader().use { it.readText() }

    private fun get(request: Request.Builder): String {
        val ready = request.header("User-Agent", "Panahon-ZH/1.0 (https://github.com/scaredpenguin627/panahon-zh)")
            .header("Referer", "https://app.powerbi.com/").header("Origin", "https://app.powerbi.com").build()
        return client.newCall(ready).execute().use { response ->
            if (!response.isSuccessful) throw IOException("Public EMB report returned HTTP ${response.code}")
            response.body.string()
        }
    }

    private fun publicJson(url: String): JsonElement =
        client.newCall(Request.Builder().url(url).build()).execute().use { response ->
            if (!response.isSuccessful) throw IOException("Regional EMB report returned HTTP ${response.code}")
            Json.parseToJsonElement(response.body.string())
        }

    private suspend fun fetchCentralLuzon(stations: Set<String>): EmbSnapshot = coroutineScope {
        val observations = stations.flatMap { station ->
            listOf("pm10", "pm25").map { pollutant ->
                async(Dispatchers.IO) {
                    val url = "https://embr3-onlinesystems.cloud/air-quality-monitoring/api/tabular/$station/$pollutant/latest"
                    val body = client.newCall(Request.Builder().url(url).build()).execute().use { response ->
                        if (!response.isSuccessful) return@async null
                        response.body.string()
                    }
                    centralLuzonReading(Json.parseToJsonElement(body).jsonObject, station, pollutant)
                }
            }
        }.mapNotNull { it.await() }
        val combined = observations.groupBy { it.name to it.observed }.values.map { group ->
            group.first().copy(readings = group.flatMap { it.readings })
        }
        EmbSnapshot(System.currentTimeMillis(), stations = combined)
    }

    private suspend fun fetch(feed: EmbFeed): EmbSnapshot = coroutineScope {
        val embed = buildJsonObject { put("k", feed.key); put("t", feed.tenant); put("m", 10) }
        val parameter = Base64.encodeToString(embed.toString().toByteArray(Charsets.UTF_8), Base64.NO_WRAP)
        val view = "https://app.powerbi.com/view".toHttpUrl().newBuilder().addQueryParameter("r", parameter).build()
        val page = get(Request.Builder().url(view))
        val redirect = Regex("""resolvedClusterUri\s*=\s*'([^']+)'""").find(page)?.groupValues?.get(1)
            ?: Regex(""""FixedClusterUri":"([^"]+)"""").find(page)?.groupValues?.get(1)
            ?: throw IOException("Public report cluster unavailable")
        val uri = redirect.toHttpUrl()
        require(uri.isHttps && uri.host.endsWith(".analysis.windows.net") && "-redirect." in uri.host)
        val api = uri.newBuilder().host(uri.host.replace("-redirect.", "-api.")).encodedPath("/").build()
        fun request(path: String) = Request.Builder().url(requireNotNull(api.resolve(path))).header("X-PowerBI-ResourceKey", feed.key)
        val model = Json.parseToJsonElement(get(request("public/reports/${feed.key}/modelsAndExploration?preferReadOnlySession=true")))
        val id = model.jsonObject.getValue("exploration").jsonObject.getValue("report").jsonObject.getValue("modelId")
        val recipes = (feed.tables.map { it.recipe } + listOfNotNull(feed.timeQuery)).distinct()
        val results = recipes.associateWith { recipe ->
            async(Dispatchers.IO) {
                val template = Json.parseToJsonElement(asset("$recipe.json")).jsonObject
                val query = JsonObject(template + ("modelId" to id)).toString()
                val body = query.toRequestBody("application/json;charset=UTF-8".toMediaType())
                val result = get(request("public/reports/querydata?synchronous=true").post(body)
                    .header("RequestId", UUID.randomUUID().toString()).header("ActivityId", UUID.randomUUID().toString()))
                decodeEmbTable(Json.parseToJsonElement(result).jsonObject)
            }
        }
        assembleEmbSnapshot(feed, results.mapValues { it.value.await() }, System.currentTimeMillis())
    }
}
