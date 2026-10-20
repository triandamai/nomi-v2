package ai.nomi.designsystem.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

fun computeInitials(name: String): String {
    val trimmed = name.trim()
    if (trimmed.isEmpty()) return "?"
    val parts = trimmed.split(Regex("\\s+")).filter { it.isNotEmpty() }.take(2)
    return parts.mapNotNull { it.firstOrNull()?.uppercaseChar() }.joinToString("").ifEmpty { "?" }
}

private const val FONT_RATIO = 0.4f

/**
 * An M3 Expressive circular avatar: renders initials in `primaryContainer` / `onPrimaryContainer`
 * matching web's `Avatar.svelte`.
 */
@Composable
fun NomiAvatar(
    name: String,
    modifier: Modifier = Modifier,
    avatarUrl: String? = null,
    size: Dp = 36.dp,
) {
    val initials = computeInitials(name)
    val colors = MaterialTheme.colorScheme
    Box(
        modifier = modifier
            .size(size)
            .clip(CircleShape)
            .background(colors.primaryContainer),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = initials,
            color = colors.onPrimaryContainer,
            fontSize = (size.value * FONT_RATIO).sp,
            fontWeight = FontWeight.SemiBold,
        )
    }
}
