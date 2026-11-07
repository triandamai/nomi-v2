package ai.nomi.designsystem.ds

import ai.nomi.designsystem.generated.NomiCorners
import ai.nomi.designsystem.generated.NomiMotionTokens
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.agentIdentity
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.designsystem.theme.toTween
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.FiniteAnimationSpec
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp

private const val TEXT3_ALPHA = 0.72f
private const val DIFF_ALPHA = 0.14f
private const val DANGER_BORDER_ALPHA = 0.4f
private const val SHADE_ALPHA = 0.08f
private const val DARK_LUMINANCE = 0.5f
private const val LABEL_TRACKING_EM = 0.06f
private const val MICRO_SP = 10
private const val MICRO_LINE_SP = 14
private const val SPIN_MS = 1000
private const val FULL_TURN = 360f
private const val BLINK_MS = 530
private const val DISABLED_ALPHA = 0.38f

/** Hairlines and the card outline (web `outline -outline-offset-1`). */
val Hairline = 1.dp

/**
 * frontend-new's design-system names (ds-twins.css) on this theme's M3 roles, so the components read like their web
 * twins: `surface` is the lowest container, `text3` the variant at 72%, `warning` the tertiary role.
 */
object Ds {
    val bg: Color @Composable @ReadOnlyComposable get() = NomiTheme.extendedColors.ground
    val surface: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.surfaceContainerLowest
    val surface2: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.surfaceContainerHigh
    val border: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.outlineVariant
    val borderStrong: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.outline
    val text: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.onSurface
    val text2: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.onSurfaceVariant
    val text3: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = TEXT3_ALPHA)
    val accent: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.primary
    val accentInk: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.onPrimary
    val accentBg: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.primaryContainer
    val danger: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.error
    val dangerBg: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.errorContainer
    val dangerBorder: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.error.copy(alpha = DANGER_BORDER_ALPHA)
    val warning: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.tertiary
    val warningBg: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.tertiaryContainer
    val diffAdd: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.primary.copy(alpha = DIFF_ALPHA)
    val diffDel: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.error.copy(alpha = DIFF_ALPHA)
    val shade: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.scrim.copy(alpha = SHADE_ALPHA)
    val peer1: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.tertiary
    val peer1Bg: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.tertiaryContainer
    val peer2: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.secondary
    val peer2Bg: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.secondaryContainer
    val codeKeyword: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.tertiary
    val codeString: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.primary
    val codeFn: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.secondary
    val codeNumber: Color @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.error
    /** A phone's hardware, dark in both themes. */
    val bezel = Color(0xFF1B1B1E)

    val dark: Boolean @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.surface.luminance() < DARK_LUMINANCE

    // The type: `text-body`, `text-ui`, `text-caption`, mono `text-label`, `text-micro`.
    val body: TextStyle @Composable @ReadOnlyComposable get() = MaterialTheme.typography.bodyLarge
    val ui: TextStyle @Composable @ReadOnlyComposable get() = MaterialTheme.typography.bodyMedium
    val caption: TextStyle @Composable @ReadOnlyComposable get() = MaterialTheme.typography.bodySmall
    val heading: TextStyle @Composable @ReadOnlyComposable get() = MaterialTheme.typography.titleLarge
    val title: TextStyle @Composable @ReadOnlyComposable get() = MaterialTheme.typography.titleSmall
    val label: TextStyle @Composable @ReadOnlyComposable get() = NomiTheme.metaStyle
    val labelCaps: TextStyle @Composable @ReadOnlyComposable get() = NomiTheme.metaStyle.copy(letterSpacing = LABEL_TRACKING_EM.em)
    val mono: TextStyle @Composable @ReadOnlyComposable get() = MaterialTheme.typography.bodySmall.copy(fontFamily = NomiTheme.metaStyle.fontFamily)
    val micro: TextStyle @Composable @ReadOnlyComposable get() = NomiTheme.metaStyle.copy(fontSize = MICRO_SP.sp, lineHeight = MICRO_LINE_SP.sp)
}

/** The web's radii: `rounded-xs` 4, `rounded-sm` 8, `rounded-md` 16 (M3 large). */
object DsShapes {
    val xs = RoundedCornerShape(NomiCorners.extraSmall.dp)
    val sm = RoundedCornerShape(NomiCorners.small.dp)
    val md = RoundedCornerShape(NomiCorners.large.dp)
    val full = RoundedCornerShape(percent = 50)
}

/** Paddings every card section uses: `px-3.5 pt-2.5 pb-2.25`. */
internal val CardHeaderPadding = PaddingValues(start = 14.dp, end = 14.dp, top = 10.dp, bottom = 9.dp)
internal val CardFooterPadding = PaddingValues(start = 14.dp, end = 14.dp, top = 9.dp, bottom = 10.dp)
internal val CardBodyPadding = PaddingValues(14.dp)

/** One line of design-system text, ellipsised when [singleLine]. */
@Composable
fun DsText(
    text: String,
    style: TextStyle,
    color: Color,
    modifier: Modifier = Modifier,
    singleLine: Boolean = false,
    weight: FontWeight? = null,
) {
    Text(
        text = text,
        style = if (weight != null) style.copy(fontWeight = weight) else style,
        color = color,
        modifier = modifier,
        maxLines = if (singleLine) 1 else Int.MAX_VALUE,
        overflow = if (singleLine) TextOverflow.Ellipsis else TextOverflow.Clip,
    )
}

/** A line icon in the design system's sizes (12, 14, 16, 18, 20). */
@Composable
fun DsIcon(glyph: NomiGlyph, size: Dp, tint: Color, modifier: Modifier = Modifier, contentDescription: String? = null) {
    NomiIcon(glyph, contentDescription, modifier, tint, size)
}

/** Turns forever, once a second, while [enabled] (loaders); still under reduced motion. */
@Composable
fun Modifier.dsSpin(enabled: Boolean = true): Modifier {
    if (!enabled || NomiTheme.reducedMotion) return this
    val transition = rememberInfiniteTransition(label = "ds-spin")
    val angle by transition.animateFloat(0f, FULL_TURN, infiniteRepeatable(tween(SPIN_MS, easing = LinearEasing)), label = "ds-spin-angle")
    return this.rotate(angle)
}

/** The spinner every "working" state shows (Lucide's LoaderCircle, turning). */
@Composable
fun DsSpinner(size: Dp, tint: Color, modifier: Modifier = Modifier) {
    DsIcon(Lucide.Loader, size, tint, modifier.dsSpin())
}

/** A caret that blinks while a reply streams in. */
@Composable
fun blinkAlpha(): Float {
    if (NomiTheme.reducedMotion) return 1f
    val transition = rememberInfiniteTransition(label = "ds-blink")
    val alpha by transition.animateFloat(1f, 0f, infiniteRepeatable(tween(BLINK_MS, easing = LinearEasing), RepeatMode.Reverse), label = "ds-blink-alpha")
    return alpha
}

/** The motion tokens as specs, snapping under reduced motion. */
object DsMotion {
    @Composable fun <T> fast(): FiniteAnimationSpec<T> = if (NomiTheme.reducedMotion) snap() else NomiMotionTokens.effectsFast.toTween()
    @Composable fun <T> effects(): FiniteAnimationSpec<T> = if (NomiTheme.reducedMotion) snap() else NomiMotionTokens.effectsDefault.toTween()
    @Composable fun <T> spatial(): FiniteAnimationSpec<T> = if (NomiTheme.reducedMotion) snap() else NomiMotionTokens.spatialFast.toTween()
}

/** Folds content open and closed: grows from the top and fades, on the motion tokens. */
@Composable
fun DsFold(visible: Boolean, modifier: Modifier = Modifier, content: @Composable () -> Unit) {
    AnimatedVisibility(
        visible = visible,
        modifier = modifier,
        enter = expandVertically(DsMotion.spatial(), expandFrom = Alignment.Top) + fadeIn(DsMotion.effects()),
        exit = shrinkVertically(DsMotion.spatial(), shrinkTowards = Alignment.Top) + fadeOut(DsMotion.fast()),
    ) { content() }
}

/** A chevron that turns to show whether its fold is open (right when closed, down when open). */
@Composable
fun DsChevron(open: Boolean, size: Dp, tint: Color, modifier: Modifier = Modifier) {
    val angle by animateFloatAsState(if (open) QUARTER_TURN else 0f, DsMotion.spatial(), label = "ds-chevron")
    DsIcon(Lucide.ChevronRight, size, tint, modifier.rotate(angle))
}

private const val QUARTER_TURN = 90f

/** The web's `outline -outline-offset-1`: a hairline inside the shape. */
fun Modifier.dsOutline(color: Color, shape: Shape = DsShapes.sm, width: Dp = Hairline): Modifier = this.border(width, color, shape)

/** Disabled controls fade to 38%, like the web's. */
fun Modifier.dsDisabled(disabled: Boolean): Modifier = if (disabled) this.alpha(DISABLED_ALPHA) else this

/** A plain row of quiet text you can press ("Undo", "Retry"): caption in text-2, a 44dp target. */
@Composable
fun DsTextAction(label: String, onClick: () -> Unit, modifier: Modifier = Modifier, color: Color = Ds.text2, underline: Boolean = false) {
    val style = if (underline) Ds.caption.copy(textDecoration = TextDecoration.Underline) else Ds.caption
    Box(
        modifier
            .heightIn(min = MinTouch)
            .clip(DsShapes.xs)
            .clickable(role = Role.Button, onClick = onClick)
            .padding(horizontal = 2.dp),
        contentAlignment = Alignment.Center,
    ) { Text(label, style = style, color = color) }
}

/** Fingers get 44dp, as the web's `pointer-coarse:min-h-11`. */
val MinTouch = 44.dp

/**
 * The card every structured reply is drawn in: a rounded outline on the page ground, a header strip on the
 * surface, the body, and an optional footer, each set apart by a hairline. Its height animates as states change.
 */
@Composable
fun DsCard(
    modifier: Modifier = Modifier,
    strong: Boolean = true,
    background: Color = Ds.bg,
    header: (@Composable RowScope.() -> Unit)? = null,
    footer: (@Composable RowScope.() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(
        modifier
            .fillMaxWidth()
            .clip(DsShapes.md)
            .background(background)
            .dsOutline(if (strong) Ds.borderStrong else Ds.border, DsShapes.md)
            .animateContentSize(DsMotion.spatial()),
    ) {
        if (header != null) {
            Row(
                Modifier.fillMaxWidth().background(Ds.surface).padding(CardHeaderPadding),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                content = header,
            )
            HorizontalDivider(thickness = Hairline, color = Ds.border)
        }
        content()
        if (footer != null) {
            HorizontalDivider(thickness = Hairline, color = Ds.border)
            Row(
                Modifier.fillMaxWidth().padding(CardFooterPadding),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                content = footer,
            )
        }
    }
}

/** A card header's title: `text-ui font-medium`. */
@Composable
fun RowScope.DsCardTitle(text: String, color: Color = Ds.text, singleLine: Boolean = false) {
    DsText(text, Ds.ui, color, Modifier.weight(1f), singleLine = singleLine, weight = FontWeight.Medium)
}

/** A thin horizontal rule in the border colour. */
@Composable
fun DsRule(modifier: Modifier = Modifier, color: Color = Ds.border) {
    Box(modifier.fillMaxWidth().height(Hairline).background(color))
}

/** A colour mixed into the surface (`color-mix(in srgb, x p%, surface)`). */
@Composable
fun Color.mixedInto(base: Color, fraction: Float): Color = this.copy(alpha = fraction).compositeOver(base)

/** Two-digit step numbers in mono: 01, 02, … */
fun stepNumber(index: Int): String = (index + 1).toString().padStart(2, '0')

/** Where a step is: done, on now, still to come, or failed. */
enum class StepState { Done, Running, Waiting, Failed }

/** A step's mark: a check, a turning loader, a dashed circle, or an alert, in its colour. */
@Composable
fun StepMark(state: StepState, size: Dp = 14.dp, modifier: Modifier = Modifier) {
    when (state) {
        StepState.Done -> DsIcon(Lucide.Check, size, Ds.accent, modifier)
        StepState.Running -> DsSpinner(size, Ds.text2, modifier)
        StepState.Waiting -> DsIcon(Lucide.CircleDashed, size, Ds.text3, modifier)
        StepState.Failed -> DsIcon(Lucide.Alert, size, Ds.danger, modifier)
    }
}
