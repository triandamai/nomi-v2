package ai.nomi.designsystem.theme

import ai.nomi.designsystem.generated.PenTypeRole
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.LineHeightStyle
import androidx.compose.ui.unit.sp

/** The pen's two families: the sans for everything, the mono for labels, ids and code. */
@Immutable
data class PenFamilies(val sans: FontFamily, val mono: FontFamily)

/**
 * The pen families. Geist and Geist Mono replace these once their files are bundled (plan task 2); until then the sans
 * and mono already bundled stand in, so the sizes, line heights and tracking are the pen's while the faces are not yet.
 */
@Composable
fun penFontFamilies(): PenFamilies {
    val families = nomiFontFamilies()
    return PenFamilies(sans = families.getValue("plain"), mono = families.getValue("mono"))
}

/** A pen type role as a [TextStyle]: size and line height in sp, tracking px as sp, the line box centred like the web's. */
fun penTextStyle(role: PenTypeRole, family: FontFamily, weight: FontWeight = FontWeight.Normal): TextStyle = TextStyle(
    fontFamily = family,
    fontWeight = weight,
    fontSize = role.size.sp,
    lineHeight = role.lineHeight.sp,
    letterSpacing = role.trackingPx.sp,
    lineHeightStyle = LineHeightStyle(LineHeightStyle.Alignment.Center, LineHeightStyle.Trim.None),
)
