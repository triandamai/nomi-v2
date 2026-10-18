package ai.nomi.designsystem.brand

import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.MorphingMark
import ai.nomi.designsystem.shape.agentIdentity
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.EaseInOut
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.togetherWith
import androidx.compose.animation.fadeIn
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.delay

/** `.stage`: 40px corners with a tight bottom-left, like the speech bubbles. */
private val StageShape = RoundedCornerShape(topStart = 40.dp, topEnd = 40.dp, bottomEnd = 40.dp, bottomStart = 12.dp)
private val TickerShape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp, bottomEnd = 24.dp, bottomStart = 8.dp)
private val TickerElevation = 18.dp
private val TickerMaxWidth = 440.dp
private val ContentGap = 22.dp
private val WideContentPadding = 44.dp
private val CompactContentPadding = 22.dp
private val CompactHeroSize = 300.dp
private val CompactHeroOffsetX = 70.dp
private val CompactHeroOffsetY = 90.dp
private const val HERO_STEP_MS = 2600L
private const val HERO_TURN_MS = 60_000
private const val TICKER_STEP_MS = 3400L
private const val FLOAT_FROM_DP = -10f
private const val FLOAT_TO_DP = 12f
private const val TILT_FROM_DEG = -8f
private const val TILT_TO_DEG = 10f
private const val LINE_WIDTH_SHARE = 0.04f
private const val LINE_MIN_SP = 36f
private const val LINE_MAX_SP = 60f
private const val COMPACT_LINE_SP = 28f
private const val LINE_HEIGHT_RATIO = 0.98f
private const val HERO_WIDTH_SHARE = 0.78f
private val HeroMaxSize = 560.dp
private const val HERO_OVERHANG_X = 0.14f
private const val HERO_OVERHANG_Y = 0.16f

private val HeroSequence = listOf("cookie9", "flower5", "clover4", "sunny8")

/** One line of the ticker: what a crew member just did (copy from the web's `auth_tick_*`). */
private data class Tick(val agent: String, val name: String, val text: String)

private val Ticks = listOf(
    Tick("money", "Money", "found 3 subscriptions you stopped using"),
    Tick("planning", "Planning", "moved your dentist reminder to 09:00"),
    Tick("coding", "Coding", "published a preview of portfolio-site"),
    Tick("personality", "Personality", "kept replies short, like you asked"),
)

/** A crew member drifting on the stage: where it sits (fractions of the stage) and how it floats. */
private class Drifter(val agent: String, val size: Dp, val x: Float, val y: Float, val floatMs: Int, val wideOnly: Boolean)

private val Drifters = listOf(
    Drifter("planning", 168.dp, x = 0.96f, y = 0.05f, floatMs = 8000, wideOnly = false),
    Drifter("money", 112.dp, x = 0.09f, y = 0.54f, floatMs = 6000, wideOnly = true),
    Drifter("personality", 76.dp, x = 0.40f, y = 0.40f, floatMs = 5000, wideOnly = true),
    Drifter("coding", 132.dp, x = -0.03f, y = 0.96f, floatMs = 9000, wideOnly = true),
)
private val CompactSkySize = 88.dp

/**
 * The auth screens' stage (`AuthShell.svelte`): a dark panel where the crew lives — a large morphing Nomi, the
 * others drifting, the wordmark, the rolling tagline and a ticker of things the crew does. [compact] is the
 * phone band above the form. Decorative: hidden from screen readers. Still under reduced motion.
 */
@Composable
fun NomiAuthStage(compact: Boolean, modifier: Modifier = Modifier) {
    val ink = NomiTheme.extendedColors.onStage
    BoxWithConstraints(
        modifier
            .clip(StageShape)
            .background(NomiTheme.extendedColors.stage)
            .clearAndSetSemantics {},
    ) {
        val heroSize = if (compact) CompactHeroSize else minOf(HeroMaxSize, maxWidth * HERO_WIDTH_SHARE)
        val heroX = if (compact) maxWidth - heroSize + CompactHeroOffsetX else maxWidth - heroSize + heroSize * HERO_OVERHANG_X
        val heroY = if (compact) maxHeight - heroSize + CompactHeroOffsetY else maxHeight - heroSize + heroSize * HERO_OVERHANG_Y
        MorphingMark(heroSize, HeroSequence, HERO_STEP_MS, Modifier.offset(heroX, heroY), face = true, turnMs = HERO_TURN_MS)
        Drifters.filter { !compact || !it.wideOnly }.forEach { d ->
            val size = if (compact) CompactSkySize else d.size
            Floating(d.floatMs, Modifier.offset((maxWidth - size) * d.x, (maxHeight - size) * d.y)) {
                AgentShape(agentIdentity(d.agent), size)
            }
        }

        val lineSize = if (compact) COMPACT_LINE_SP else (maxWidth.value * LINE_WIDTH_SHARE).coerceIn(LINE_MIN_SP, LINE_MAX_SP)
        Column(
            Modifier.fillMaxSize().padding(if (compact) CompactContentPadding else WideContentPadding),
            verticalArrangement = Arrangement.spacedBy(ContentGap),
        ) {
            NomiWordmark(markSize = 44.dp, wordSize = 30f, color = ink)
            TaglineWheel(brandStyle(lineSize.sp, (lineSize * LINE_HEIGHT_RATIO).sp), ink)
            Spacer(Modifier.weight(1f))
            Ticker(Modifier.widthIn(max = TickerMaxWidth).fillMaxWidth())
        }
    }
}

@Composable
private fun Floating(periodMs: Int, modifier: Modifier, content: @Composable () -> Unit) {
    if (NomiTheme.reducedMotion) {
        Box(modifier) { content() }
        return
    }
    val drift = rememberInfiniteTransition(label = "drift")
    val t by drift.animateFloat(0f, 1f, infiniteRepeatable(tween(periodMs, easing = EaseInOut), RepeatMode.Reverse), label = "t")
    Box(
        modifier.graphicsLayer {
            translationY = (FLOAT_FROM_DP + (FLOAT_TO_DP - FLOAT_FROM_DP) * t) * density
            rotationZ = TILT_FROM_DEG + (TILT_TO_DEG - TILT_FROM_DEG) * t
        },
    ) { content() }
}

private val TickerAvatar = 36.dp
private val TickerGap = 12.dp
private const val TICKER_NAME_WEIGHT = 700
private const val TICKER_TIME = "just now"

@Composable
private fun Ticker(modifier: Modifier) {
    var index by remember { mutableIntStateOf(0) }
    if (!NomiTheme.reducedMotion) {
        LaunchedEffect(Unit) {
            while (true) {
                delay(TICKER_STEP_MS)
                index = (index + 1) % Ticks.size
            }
        }
    }
    val spatial = MaterialTheme.motionScheme.defaultSpatialSpec<androidx.compose.ui.unit.IntOffset>()
    val effects = MaterialTheme.motionScheme.fastEffectsSpec<Float>()
    AnimatedContent(
        targetState = Ticks[index],
        modifier = modifier,
        transitionSpec = { (slideInVertically(spatial) { it / 2 } + fadeIn(effects)) togetherWith (slideOutVertically(spatial) { -it / 3 } + fadeOut(effects)) },
        label = "ticker",
    ) { tick ->
        Row(
            Modifier
                .fillMaxWidth()
                .shadow(TickerElevation, TickerShape, clip = false)
                .background(MaterialTheme.colorScheme.surfaceContainerLowest, TickerShape)
                .padding(start = 12.dp, top = 12.dp, end = 16.dp, bottom = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(TickerGap),
        ) {
            AgentShape(agentIdentity(tick.agent), TickerAvatar, working = true)
            Column(Modifier.weight(1f)) {
                Text(tick.name, style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight(TICKER_NAME_WEIGHT)), color = MaterialTheme.colorScheme.onSurface)
                Text(tick.text, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
            }
            Text(TICKER_TIME, style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.align(Alignment.Top))
        }
    }
}
