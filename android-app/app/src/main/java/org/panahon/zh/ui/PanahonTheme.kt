package org.panahon.zh.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

private val DarkColors = darkColorScheme(
    primary = Color(0xFFB8D9FF),
    onPrimary = Color(0xFF003256),
    secondary = Color(0xFFC8D5E6),
    background = Color(0xFF101722),
    surface = Color(0xFF151F2C),
    onBackground = Color(0xFFF1F4F8),
    onSurface = Color(0xFFF1F4F8),
)

@Composable
fun PanahonTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = DarkColors,
        typography = MaterialTheme.typography,
        content = content,
    )
}

