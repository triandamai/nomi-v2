package ai.nomi.designsystem.brand

import ai.nomi.designsystem.component.nomiFocusRing
import ai.nomi.designsystem.generated.NomiCorners
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.NomiIdentity
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.delay

/** The brand face at its heaviest, tightly tracked: the wordmark and the auth stage's headline. */
private val BrandTracking = (-0.045).em
private const val BRAND_WEIGHT = 800
private const val WORD = "nomi"

/** `.nomi-brand`: Nomi's shape with its face, then "nomi". */
@Composable
fun NomiWordmark(modifier: Modifier = Modifier, markSize: Dp = 40.dp, wordSize: Float = 28f, color: Color = MaterialTheme.colorScheme.onSurface) {
    Row(modifier, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(markSize / WORDMARK_GAP_DIVISOR)) {
        AgentShape(NomiIdentity, markSize, face = true)
        Text(WORD, style = brandStyle(wordSize.sp, lineHeight = wordSize.sp), color = color)
    }
}

private const val WORDMARK_GAP_DIVISOR = 4

/** The brand face (Bricolage Grotesque) at weight 800 with the web's tight tracking. */
@Composable
fun brandStyle(size: androidx.compose.ui.unit.TextUnit, lineHeight: androidx.compose.ui.unit.TextUnit = size): TextStyle =
    MaterialTheme.typography.displayLarge.copy(
        fontSize = size,
        lineHeight = lineHeight,
        fontWeight = FontWeight(BRAND_WEIGHT),
        letterSpacing = BrandTracking,
    )

/** A page's title: the web's `md-display-small` heading every list and account page opens with. */
@Composable
fun NomiPageTitle(text: String, modifier: Modifier = Modifier) {
    Text(
        text,
        style = MaterialTheme.typography.displaySmall,
        color = MaterialTheme.colorScheme.onSurface,
        modifier = modifier.semantics { heading() },
    )
}

private val FabHeight = 56.dp
private val FabStartPadding = 14.dp
private val FabEndPadding = 16.dp
private val FabGap = 12.dp
private val FabIconSize = 24.dp
private const val FAB_LABEL_WEIGHT = 650

/**
 * `.nomi-fab`: the chrome's one gradient object — starting a conversation is the hero action. The action
 * gradient, a large-increased corner that tightens on press; [compact] is the 56dp square the rail shows.
 */
@Composable
fun NomiNewChatButton(onClick: () -> Unit, modifier: Modifier = Modifier, label: String = "New chat", compact: Boolean = false) {
    val interaction = remember { MutableInteractionSource() }
    val pressed by interaction.collectIsPressedAsState()
    val rest = if (compact) NomiCorners.large else NomiCorners.largeIncreased
    val corner by animateDpAsState(
        (if (pressed) NomiCorners.medium else rest).dp,
        if (NomiTheme.reducedMotion) androidx.compose.animation.core.snap() else MaterialTheme.motionScheme.fastSpatialSpec(),
    )
    val shape = RoundedCornerShape(corner)
    val ink = NomiTheme.extendedColors.onAction
    Row(
        modifier
            .height(FabHeight)
            .then(if (compact) Modifier.size(FabHeight) else Modifier)
            .nomiFocusRing(shape)
            .background(NomiTheme.extendedColors.actionBrush, shape)
            .clickable(interaction, indication = androidx.compose.material3.ripple(color = ink), role = Role.Button, onClickLabel = label, onClick = onClick)
            .padding(start = if (compact) 0.dp else FabStartPadding, end = if (compact) 0.dp else FabEndPadding),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = if (compact) Arrangement.Center else Arrangement.spacedBy(FabGap),
    ) {
        NomiIcon(NomiIcons.Plus, contentDescription = if (compact) label else null, tint = ink, size = FabIconSize)
        if (!compact) {
            Text(label, style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight(FAB_LABEL_WEIGHT)), color = ink)
        }
    }
}

/** `$lib/tagline`: the verbs the auth stage's line rolls through. */
val TaglineVerbs = listOf("remembers", "builds", "works")
private const val TAGLINE_BEFORE = "A small crew of agents that"
private const val TAGLINE_AFTER = "for you."
private const val TAGLINE_STEP_MS = 2200L
private const val LEAD_WIDTH_EM = 6.6f

/**
 * "A small crew of agents that [remembers] for you.", the verb rolling up like a wheel every [stepMs].
 * Under reduced motion it holds the first verb.
 */
@Composable
fun TaglineWheel(style: TextStyle, color: Color, modifier: Modifier = Modifier, stepMs: Long = TAGLINE_STEP_MS) {
    var index by remember { mutableIntStateOf(0) }
    val still = NomiTheme.reducedMotion
    if (!still) {
        LaunchedEffect(Unit) {
            while (true) {
                delay(stepMs)
                index = (index + 1) % TaglineVerbs.size
            }
        }
    }
    val spatial = MaterialTheme.motionScheme.defaultSpatialSpec<androidx.compose.ui.unit.IntOffset>()
    // `.stage__line { max-width: 11ch }`: the lead-in wraps into two balanced lines, the verb line stays whole.
    val leadWidth = with(androidx.compose.ui.platform.LocalDensity.current) { (style.fontSize * LEAD_WIDTH_EM).toDp() }
    androidx.compose.foundation.layout.Column(modifier.semantics(mergeDescendants = true) {}) {
        Text(TAGLINE_BEFORE, style = style, color = color, modifier = Modifier.widthIn(max = leadWidth))
        Row {
            AnimatedContent(
                targetState = index,
                transitionSpec = { slideInVertically(spatial) { it } togetherWith slideOutVertically(spatial) { -it } },
                label = "tagline",
            ) { i -> Text(TaglineVerbs[i], style = style, color = color) }
            Text(" $TAGLINE_AFTER", style = style, color = color)
        }
    }
}
