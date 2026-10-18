package ai.nomi.designsystem.theme

import ai.nomi.designsystem.generated.NomiTypescale
import ai.nomi.designsystem.generated.TypeRole
import ai.nomi.designsystem.resources.Res
import ai.nomi.designsystem.resources.bricolage_grotesque_variable
import ai.nomi.designsystem.resources.instrument_sans_variable
import ai.nomi.designsystem.resources.jetbrains_mono_variable
import androidx.compose.material3.Typography
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import org.jetbrains.compose.resources.FontResource
import org.jetbrains.compose.resources.Font as resourceFont

private const val FAMILY_BRAND = "brand"
private const val FAMILY_MONO = "mono"

/** Every baseline and emphasized role, so each family can declare exactly the weights Nomi uses. */
private val allRoles: List<TypeRole> = with(NomiTypescale) {
    listOf(
        displayLarge, displayMedium, displaySmall, headlineLarge, headlineMedium, headlineSmall,
        titleLarge, titleMedium, titleSmall, bodyLarge, bodyMedium, bodySmall,
        labelLarge, labelMedium, labelSmall,
        emphasizedDisplayLarge, emphasizedDisplayMedium, emphasizedDisplaySmall,
        emphasizedHeadlineLarge, emphasizedHeadlineMedium, emphasizedHeadlineSmall,
        emphasizedTitleLarge, emphasizedTitleMedium, emphasizedTitleSmall,
        emphasizedBodyLarge, emphasizedBodyMedium, emphasizedBodySmall,
        emphasizedLabelLarge, emphasizedLabelMedium, emphasizedLabelSmall,
    )
}

/** One Font entry per distinct weight, each pinning the variable font's wght axis to exactly that value. */
@Composable
private fun variableFamily(resource: FontResource, weights: Set<Int>): FontFamily = FontFamily(
    weights.sorted().map { w ->
        resourceFont(resource, weight = FontWeight(w), variationSettings = FontVariation.Settings(FontVariation.weight(w)))
    },
)

/** The families Nomi's typescale names; "title" and "plain" are both Instrument Sans on the web. */
@Composable
fun nomiFontFamilies(): Map<String, FontFamily> {
    fun weightsOf(vararg families: String) = allRoles.filter { it.family in families }.map { it.weight }.toSet()
    val brand = variableFamily(Res.font.bricolage_grotesque_variable, weightsOf(FAMILY_BRAND))
    val sans = variableFamily(Res.font.instrument_sans_variable, weightsOf("title", "plain"))
    val mono = variableFamily(Res.font.jetbrains_mono_variable, weightsOf(FAMILY_MONO) + NomiTypescale.labelSmall.weight)
    return mapOf(FAMILY_BRAND to brand, "title" to sans, "plain" to sans, FAMILY_MONO to mono)
}

private fun TypeRole.toTextStyle(families: Map<String, FontFamily>) = TextStyle(
    fontFamily = families.getValue(family),
    fontWeight = FontWeight(weight),
    fontSize = sizeSp.sp,
    lineHeight = lineHeightSp.sp,
    letterSpacing = trackingEm.em,
)

/** M3 [Typography] from the generated typescale (sizes, line heights, tracking and weights as on the web). */
@Composable
fun nomiTypography(families: Map<String, FontFamily> = nomiFontFamilies()): Typography = with(NomiTypescale) {
    Typography(
        displayLarge = displayLarge.toTextStyle(families),
        displayMedium = displayMedium.toTextStyle(families),
        displaySmall = displaySmall.toTextStyle(families),
        headlineLarge = headlineLarge.toTextStyle(families),
        headlineMedium = headlineMedium.toTextStyle(families),
        headlineSmall = headlineSmall.toTextStyle(families),
        titleLarge = titleLarge.toTextStyle(families),
        titleMedium = titleMedium.toTextStyle(families),
        titleSmall = titleSmall.toTextStyle(families),
        bodyLarge = bodyLarge.toTextStyle(families),
        bodyMedium = bodyMedium.toTextStyle(families),
        bodySmall = bodySmall.toTextStyle(families),
        labelLarge = labelLarge.toTextStyle(families),
        labelMedium = labelMedium.toTextStyle(families),
        labelSmall = labelSmall.toTextStyle(families),
    )
}

/** Meta text (timestamps, ids, counters): JetBrains Mono at `labelSmall` metrics. */
fun nomiMetaStyle(typography: Typography, families: Map<String, FontFamily>): TextStyle =
    typography.labelSmall.copy(fontFamily = families.getValue(FAMILY_MONO))
