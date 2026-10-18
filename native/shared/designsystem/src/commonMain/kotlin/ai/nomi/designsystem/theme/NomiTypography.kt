package ai.nomi.designsystem.theme

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
import org.jetbrains.compose.resources.Font as resourceFont

private val BrandWeights = listOf(
    FontWeight.Normal, FontWeight.Medium, FontWeight.SemiBold, FontWeight.Bold, FontWeight.ExtraBold,
)
private val BodyWeights = listOf(FontWeight.Normal, FontWeight.Medium, FontWeight.SemiBold, FontWeight.Bold)
private val MonoWeights = listOf(FontWeight.Normal, FontWeight.Medium)

@Composable
private fun variableFamily(
    resource: org.jetbrains.compose.resources.FontResource,
    weights: List<FontWeight>,
): FontFamily = FontFamily(
    weights.map { weight ->
        resourceFont(resource, weight = weight, variationSettings = FontVariation.Settings(FontVariation.weight(weight.weight)))
    },
)

@Composable fun bricolageGrotesque(): FontFamily = variableFamily(Res.font.bricolage_grotesque_variable, BrandWeights)
@Composable fun instrumentSans(): FontFamily = variableFamily(Res.font.instrument_sans_variable, BodyWeights)
@Composable fun jetBrainsMono(): FontFamily = variableFamily(Res.font.jetbrains_mono_variable, MonoWeights)

/** Display and headline styles in Bricolage Grotesque (display ExtraBold); title, body and label in Instrument Sans. */
@Composable
fun nomiTypography(): Typography {
    val base = Typography()
    val brand = bricolageGrotesque()
    val body = instrumentSans()
    fun TextStyle.display() = copy(fontFamily = brand, fontWeight = FontWeight.ExtraBold)
    fun TextStyle.headline() = copy(fontFamily = brand)
    fun TextStyle.text() = copy(fontFamily = body)
    return base.copy(
        displayLarge = base.displayLarge.display(),
        displayMedium = base.displayMedium.display(),
        displaySmall = base.displaySmall.display(),
        headlineLarge = base.headlineLarge.headline(),
        headlineMedium = base.headlineMedium.headline(),
        headlineSmall = base.headlineSmall.headline(),
        titleLarge = base.titleLarge.text(),
        titleMedium = base.titleMedium.text(),
        titleSmall = base.titleSmall.text(),
        bodyLarge = base.bodyLarge.text(),
        bodyMedium = base.bodyMedium.text(),
        bodySmall = base.bodySmall.text(),
        labelLarge = base.labelLarge.text(),
        labelMedium = base.labelMedium.text(),
        labelSmall = base.labelSmall.text(),
    )
}

/** Meta text (timestamps, ids, counters): JetBrains Mono at `labelSmall` metrics. */
@Composable
fun nomiMetaStyle(typography: Typography): TextStyle =
    typography.labelSmall.copy(fontFamily = jetBrainsMono())
