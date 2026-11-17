package org.panahon.zh.ui

import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.unit.dp
import org.panahon.zh.nativebridge.BackgroundProfile
import org.panahon.zh.nativebridge.NativeBridge
import kotlin.math.sin

@Composable
fun WeatherBackground(snapshot: String?, modifier: Modifier = Modifier) {
    val profile = NativeBridge.backgroundProfile(snapshot)
    val transition = rememberInfiniteTransition(label = "weather-background")
    val drift by transition.animateFloat(
        initialValue = 0f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(tween(18_000), RepeatMode.Reverse),
        label = "cloud-drift",
    )
    Box(modifier = modifier.fillMaxSize()) {
        Canvas(Modifier.fillMaxSize()) {
            val primary = Color(android.graphics.Color.parseColor(profile.primary))
            val secondary = Color(android.graphics.Color.parseColor(profile.secondary))
            drawRect(Brush.verticalGradient(listOf(primary, secondary)))

            val glow = 0.18f + profile.intensity.coerceIn(0f, 1f) * 0.16f
            val center = Offset(size.width * (0.25f + drift * 0.45f), size.height * 0.2f)
            drawCircle(Color.White.copy(alpha = glow), size.minDimension * 0.42f, center)

            repeat(5) { index ->
                val phase = drift * (index + 1) * 0.9f
                val x = size.width * ((index + 1) / 6f) + sin(phase) * 28.dp.toPx()
                val y = size.height * (0.16f + index * 0.14f)
                val cloudColor = lerp(Color.White, primary, 0.6f).copy(alpha = 0.07f + profile.intensity * 0.06f)
                drawOval(cloudColor, Offset(x - 110.dp.toPx(), y), androidx.compose.ui.geometry.Size(220.dp.toPx(), 52.dp.toPx()))
            }

            if (profile.mode == "rain" || profile.mode == "storm") {
                repeat(22) { index ->
                    val x = ((index * 57f + drift * 240f) % (size.width + 40f)) - 20f
                    val y = (index * 83f + drift * 320f) % (size.height + 80f) - 80f
                    drawLine(Color(0x99D8EDFF), Offset(x, y), Offset(x - 12f, y + 36f), 1.5f)
                }
            }

            if (!profile.isDay || profile.mode == "night") {
                drawCircle(Color(0xB8E7F0FF), size.minDimension * 0.055f, Offset(size.width * 0.76f, size.height * 0.18f))
                repeat(10) { index ->
                    val x = (index * 93f + 21f) % size.width
                    val y = (index * 67f + 31f) % (size.height * 0.58f)
                    drawCircle(Color.White.copy(alpha = 0.45f), 1.3.dp.toPx(), Offset(x, y))
                }
            }

            if (profile.stale) {
                drawArc(Color.White.copy(alpha = 0.16f), 15f, 150f, false, style = Stroke(width = 2.dp.toPx()))
            }
        }
    }
}

