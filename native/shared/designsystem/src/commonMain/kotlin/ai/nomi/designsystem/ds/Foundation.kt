package ai.nomi.designsystem.ds

import ai.nomi.designsystem.generated.CrewColors
import ai.nomi.designsystem.generated.NomiCrewColors
import ai.nomi.designsystem.generated.NomiPenRadius
import ai.nomi.designsystem.generated.NomiPenTracking
import ai.nomi.designsystem.generated.NomiPenType
import ai.nomi.designsystem.generated.NomiMotionTokens
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.agentIdentity
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.designsystem.theme.penTextStyle
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
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.FlowRowScope
import androidx.compose.foundation.layout.widthIn
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
private const val BEZEL_ARGB = 0xFF1B1B1E
private const val DEFAULT_CREW = "nomi"
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
 * frontend-new's design-system names on the pen tokens: `Ds.surface2` is the web's `bg-surface-2`, `Ds.ui` its
 * `text-ui`, so a component reads like its Svelte twin.
 */
object Ds {
    val bg: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.bg)
    val bgClear: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.bgClear)
    val surface: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.surface)
    val surface2: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.surface2)
    val border: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.border)
    val borderStrong: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.borderStrong)
    val text: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.text)
    val text2: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.text2)
    val text3: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.text3)
    val accent: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.accent)
    val accentInk: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.accentInk)
    val accentBg: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.accentBg)
    val danger: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.danger)
    val dangerBg: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.dangerBg)
    val dangerBorder: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.dangerBorder)
    val warning: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.warning)
    val warningBg: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.warningBg)
    val focus: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.focus)
    val overlay: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.overlay)
    val shadow: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.shadow)
    val heat1: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.heat1)
    val heat2: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.heat2)
    val heat3: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.heat3)
    val heat4: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.heat4)
    val diffAdd: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.diffAdd)
    val diffDel: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.diffDel)
    val peer1: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.peer1)
    val peer1Bg: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.peer1Bg)
    val peer2: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.peer2)
    val peer2Bg: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.peer2Bg)
    val codeKeyword: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.codeKeyword)
    val codeString: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.codeString)
    val codeFn: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.codeFn)
    val codeNumber: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.codeNumber)
    val codeType: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.codeType)
    val kbBg: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.kbBg)
    val kbKey: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.kbKey)
    val kbKeyAlt: Color @Composable @ReadOnlyComposable get() = Color(NomiTheme.pen.kbKeyAlt)

    /** A wash of black on a coloured bar, the same in both themes (app.css `--color-shade`). */
    val shade = Color.Black.copy(alpha = SHADE_ALPHA)

    /** A phone's hardware, dark in both themes (app.css `--color-bezel`). */
    val bezel = Color(BEZEL_ARGB)

    val dark: Boolean @Composable @ReadOnlyComposable get() = NomiTheme.dark

    // The type, by the web's text-* names. Sans unless the web draws it in mono.
    val display: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.display, NomiTheme.families.sans, FontWeight.SemiBold)
    val heading: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.heading, NomiTheme.families.sans, FontWeight.SemiBold)
    val body: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.body, NomiTheme.families.sans)
    val ui: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.ui, NomiTheme.families.sans)
    val caption: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.caption, NomiTheme.families.sans)

    /** `font-medium text-ui`: a row's or card's title. */
    val title: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.ui, NomiTheme.families.sans, FontWeight.Medium)
    val label: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.label, NomiTheme.families.mono)

    /** `text-label tracking-label uppercase`: a section's label. */
    val labelCaps: TextStyle @Composable @ReadOnlyComposable get() = label.copy(letterSpacing = NomiPenTracking.label.sp)
    val mono: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.caption, NomiTheme.families.mono)
    val micro: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.micro, NomiTheme.families.mono)
    val nano: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.nano, NomiTheme.families.mono)

    // Phones: 16 body so a field never zooms, 17 sheet titles, 14 second lines, the tab's large title.
    val phone: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.phone, NomiTheme.families.sans)
    val phoneSm: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.phoneSm, NomiTheme.families.sans)
    val phoneTitle: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.phoneTitle, NomiTheme.families.sans, FontWeight.SemiBold)
    val phoneLarge: TextStyle @Composable @ReadOnlyComposable get() =
        penTextStyle(NomiPenType.phoneLarge, NomiTheme.families.sans, FontWeight.SemiBold).copy(letterSpacing = NomiPenTracking.phoneLarge.sp)
    val phoneAction: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.phoneAction, NomiTheme.families.sans)
    val phoneDisplay: TextStyle @Composable @ReadOnlyComposable get() = penTextStyle(NomiPenType.phoneDisplay, NomiTheme.families.sans, FontWeight.SemiBold)
}

/** A crew member's own colour (its mascot), by agent name; unknown agents read as Nomi. */
@Composable
@ReadOnlyComposable
fun crewColor(agent: String): Color = Color(crewColors(agent).base)

/** A crew member's tinted ground in this scheme. */
@Composable
@ReadOnlyComposable
fun crewBg(agent: String): Color = crewColors(agent).let { Color(if (Ds.dark) it.bgDark else it.bgLight) }

/** A crew member's ink on its tinted ground in this scheme. */
@Composable
@ReadOnlyComposable
fun crewInk(agent: String): Color = crewColors(agent).let { Color(if (Ds.dark) it.inkDark else it.inkLight) }

private fun crewColors(agent: String): CrewColors =
    NomiCrewColors.all[agent.lowercase()] ?: NomiCrewColors.all.getValue(DEFAULT_CREW)

/** The web's radii: `rounded-xs` 4, `rounded-sm` 6, `rounded-md` 10, and the in-between ones the pen draws. */
object DsShapes {
    val xs2 = RoundedCornerShape(NomiPenRadius.xs2.dp)
    val xs = RoundedCornerShape(NomiPenRadius.xs.dp)
    val sm = RoundedCornerShape(NomiPenRadius.sm.dp)
    val tile = RoundedCornerShape(NomiPenRadius.tile.dp)
    val inner = RoundedCornerShape(NomiPenRadius.inner.dp)
    val md = RoundedCornerShape(NomiPenRadius.md.dp)
    val lg = RoundedCornerShape(NomiPenRadius.lg.dp)
    val xl = RoundedCornerShape(NomiPenRadius.xl.dp)
    val xl2 = RoundedCornerShape(NomiPenRadius.xl2.dp)
    val sheet = RoundedCornerShape(topStart = NomiPenRadius.sheet.dp, topEnd = NomiPenRadius.sheet.dp)
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
    footer: (@Composable FlowRowScope.() -> Unit)? = null,
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
            FlowRow(
                Modifier.fillMaxWidth().padding(CardFooterPadding),
                horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.End),
                verticalArrangement = Arrangement.spacedBy(8.dp),
                itemVerticalAlignment = Alignment.CenterVertically,
                content = footer,
            )
        }
    }
}

/** The narrowest a card footer's note gets before the buttons wrap under it (the web's `basis-40`). */
val FooterNoteMin = 160.dp

/** A card footer's note: takes the room the buttons leave, never narrower than [FooterNoteMin]. */
fun Modifier.footerNote(scope: RowScope): Modifier = with(scope) { this@footerNote.weight(1f).widthIn(min = FooterNoteMin) }

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
