package ai.nomi.feature.chat.views

import ai.nomi.designsystem.component.NomiWavyProgress
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.NomiIdentity
import ai.nomi.designsystem.shape.agentIdentity
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.TextMeasurer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.delay
import kotlinx.datetime.TimeZone

// One composable per display view, on the shared ViewFrame (frontend/src/lib/components/views/*.svelte).

internal const val FILL = 1f
private const val BOLD = 700
private const val SEMIBOLD = 600
private const val BRAND_WEIGHT = 800
internal val TitleWeight = FontWeight(BOLD)
internal val StrongWeight = FontWeight(SEMIBOLD)
internal val GapS = 2.dp
internal val GapXs = 4.dp
internal val GapSm = 6.dp
internal val GapMd = 10.dp
internal val InsetM = 14.dp
internal val GapXl = 16.dp
internal val GapXxl = 24.dp
internal val CellPaddingV = 9.dp
/** Nudges text down to sit level with a dot or mark beside it. */
internal val TextNudge = 3.dp
internal val DotNudge = 5.dp
private val LegendMinWidth = 140.dp
internal val GapM = 8.dp
internal val GapL = 12.dp
internal val FramePadding = 16.dp
internal val Hairline = 1.dp
private const val TITLE_SP = 16f
private const val DETAIL_SP = 14f
private const val SMALL_SP = 13f
private const val ROW_SP = 15f
private const val LINE_HEIGHT = 1.4f

@Composable internal fun detailStyle() = MaterialTheme.typography.bodyMedium.copy(fontSize = DETAIL_SP.sp, lineHeight = (DETAIL_SP * LINE_HEIGHT).sp)
@Composable internal fun rowStyle() = MaterialTheme.typography.bodyLarge.copy(fontSize = ROW_SP.sp, lineHeight = (ROW_SP * LINE_HEIGHT).sp)
@Composable internal fun smallStyle() = MaterialTheme.typography.bodySmall.copy(fontSize = SMALL_SP.sp)

/** `ViewFrame.svelte`: the surface every view sits on — extra-large corner, a hairline, the card colour. */
@Composable
internal fun ViewFrame(title: String?, modifier: Modifier = Modifier, flush: Boolean = false, content: @Composable ColumnScope.() -> Unit) {
    val shape = MaterialTheme.shapes.extraLarge
    Column(
        modifier
            .fillMaxWidth()
            .clip(shape)
            .background(MaterialTheme.colorScheme.surfaceContainerLowest)
            .border(Hairline, MaterialTheme.colorScheme.outlineVariant, shape)
            .padding(if (flush) 0.dp else FramePadding),
        verticalArrangement = Arrangement.spacedBy(GapL),
    ) {
        title?.let {
            Text(
                it,
                style = MaterialTheme.typography.bodyLarge.copy(fontSize = TITLE_SP.sp, fontWeight = TitleWeight),
                color = MaterialTheme.colorScheme.onSurface,
                modifier = if (flush) Modifier.padding(start = FramePadding, end = FramePadding, top = InsetM) else Modifier,
            )
        }
        content()
    }
}

/** A list row's separator (`border-top: 1px outline-variant`). */
@Composable
internal fun Rule() = HorizontalDivider(thickness = Hairline, color = MaterialTheme.colorScheme.outlineVariant)

private val MarkSize = 22.dp
private val BulletSize = 6.dp
private val CheckSize = 14.dp
private val ListRowPadding = 10.dp

@Composable
internal fun ListViewBlock(view: ChatView.List, timeZone: TimeZone) {
    val colors = MaterialTheme.colorScheme
    val uri = LocalUriHandler.current
    ViewFrame(view.title) {
        Column {
            view.items.forEachIndexed { i, item ->
                if (i > 0) Rule()
                val done = view.style == ChatView.ListStyle.Check && item.done
                Row(
                    Modifier.fillMaxWidth().padding(top = if (i == 0) 0.dp else ListRowPadding, bottom = if (i == view.items.lastIndex) 0.dp else ListRowPadding),
                    horizontalArrangement = Arrangement.spacedBy(GapL),
                ) {
                    Box(
                        Modifier.padding(top = Hairline).size(MarkSize).clip(CircleShape).then(
                            when {
                                done -> Modifier.background(colors.primary)
                                view.style == ChatView.ListStyle.Check -> Modifier.border(GapS, colors.outline, CircleShape)
                                view.style == ChatView.ListStyle.Numbered -> Modifier.background(colors.surfaceContainerHigh)
                                else -> Modifier
                            },
                        ),
                        contentAlignment = Alignment.Center,
                    ) {
                        when {
                            view.style == ChatView.ListStyle.Numbered ->
                                Text("${i + 1}", style = MaterialTheme.typography.labelSmall.copy(fontWeight = TitleWeight), color = colors.onSurfaceVariant)
                            done -> NomiIcon(NomiIcons.Check, null, tint = colors.onPrimary, size = CheckSize)
                            view.style == ChatView.ListStyle.Bullet -> Box(Modifier.size(BulletSize).clip(CircleShape).background(colors.onSurfaceVariant))
                        }
                    }
                    Column(Modifier.weight(FILL), verticalArrangement = Arrangement.spacedBy(GapS)) {
                        val url = item.url
                        Text(
                            item.title,
                            style = rowStyle().copy(fontWeight = StrongWeight, textDecoration = if (done) TextDecoration.LineThrough else null),
                            color = when {
                                url != null -> colors.primary
                                done -> colors.onSurfaceVariant
                                else -> colors.onSurface
                            },
                            modifier = if (url != null) Modifier.clickable(role = Role.Button) { uri.openUri(url) } else Modifier,
                        )
                        item.detail?.let { Text(it, style = detailStyle(), color = colors.onSurfaceVariant) }
                    }
                    item.meta?.let { Text(formatValue(it, item.format, timeZone), style = rowStyle().copy(fontWeight = StrongWeight), color = colors.onSurface) }
                }
            }
        }
    }
}

/** `TableBlock.svelte`: a table where the bubble is wide, a list (headline, supporting, trailing) where it's narrow. */
private val TableNarrow = 480.dp
private val TableRowMin = 56.dp
private val CellPadding = 10.dp
private val TableMinColumn = 96.dp

@Composable
internal fun TableViewBlock(title: String?, columns: List<ChatView.Column>, rows: List<Map<String, Scalar?>>, timeZone: TimeZone) {
    val colors = MaterialTheme.colorScheme
    fun cell(row: Map<String, Scalar?>, column: ChatView.Column) = row[column.key]?.let { formatValue(it, column.format, timeZone) }.orEmpty()
    val body: @Composable () -> Unit = {
        BoxWithConstraints(Modifier.fillMaxWidth()) {
            if (maxWidth < TableNarrow) {
                val parts = tableListColumns(columns, rows)
                val trailing = parts.trailing
                val headline = parts.headline
                val supporting = listOfNotNull(parts.category) + parts.supporting
                Column(Modifier.fillMaxWidth().clip(MaterialTheme.shapes.large).background(colors.surfaceContainerLow).padding(vertical = GapXs)) {
                    rows.forEachIndexed { i, row ->
                        if (i > 0) Rule()
                        Row(Modifier.fillMaxWidth().heightIn(min = TableRowMin).padding(horizontal = GapL, vertical = GapM), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(GapL)) {
                            Column(Modifier.weight(FILL)) {
                                headline?.let { Text(cell(row, it), style = rowStyle(), color = colors.onSurface, maxLines = 2, overflow = TextOverflow.Ellipsis) }
                                val line = supporting.map { cell(row, it) }.filter { it.isNotEmpty() }.joinToString(" · ")
                                if (line.isNotEmpty()) Text(line, style = detailStyle(), color = colors.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
                            }
                            trailing?.let { Text(cell(row, it), style = detailStyle().copy(fontWeight = StrongWeight), color = colors.onSurface) }
                        }
                    }
                }
            } else {
                val width = maxOf(TableMinColumn, maxWidth / columns.size.coerceAtLeast(1))
                Column(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState())) {
                    Row {
                        columns.forEach { Text(it.label, style = smallStyle().copy(fontWeight = StrongWeight), color = colors.onSurfaceVariant, modifier = Modifier.width(width).padding(CellPadding)) }
                    }
                    rows.forEach { row ->
                        Rule()
                        Row(Modifier.height(IntrinsicSize.Min)) {
                            columns.forEach { column ->
                                val numeric = column.format == ValueFormat.Money || column.format == ValueFormat.Number || column.format == ValueFormat.Percent
                                Text(
                                    cell(row, column),
                                    style = rowStyle().copy(fontWeight = if (numeric) StrongWeight else null),
                                    color = colors.onSurface,
                                    modifier = Modifier.width(width).padding(CellPadding),
                                )
                            }
                        }
                    }
                }
            }
        }
    }
    if (title != null) ViewFrame(title) { body() } else body()
}

private val CompareNarrow = 460.dp
private val CompareLabelWidth = 96.dp
private const val COMPARE_LABEL_SHARE = 0.8f

@Composable
internal fun ComparisonViewBlock(view: ChatView.Comparison, timeZone: TimeZone) {
    val colors = MaterialTheme.colorScheme
    fun value(row: ChatView.ComparisonRow, i: Int) = row.values.getOrNull(i)?.let { formatValue(it, null, timeZone) }.orEmpty()
    ViewFrame(view.title) {
        BoxWithConstraints(Modifier.fillMaxWidth()) {
            if (maxWidth < CompareNarrow) {
                Column(verticalArrangement = Arrangement.spacedBy(GapMd)) {
                    view.options.forEachIndexed { i, option ->
                        val best = view.recommended == i
                        Column(
                            Modifier.fillMaxWidth().clip(MaterialTheme.shapes.large).background(colors.surfaceContainerLow)
                                .then(if (best) Modifier.border(GapS, colors.primary, MaterialTheme.shapes.large) else Modifier)
                                .padding(horizontal = InsetM, vertical = GapL),
                            verticalArrangement = Arrangement.spacedBy(GapSm),
                        ) {
                            Row(horizontalArrangement = Arrangement.spacedBy(GapM), verticalAlignment = Alignment.CenterVertically) {
                                Text(option, style = rowStyle().copy(fontWeight = TitleWeight))
                                if (best) RecommendedBadge()
                            }
                            view.rows.forEach { row ->
                                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(GapL)) {
                                    Text(row.label, style = rowStyle(), color = colors.onSurfaceVariant, modifier = Modifier.weight(FILL))
                                    Text(value(row, i), style = rowStyle().copy(fontWeight = StrongWeight), color = colors.onSurface)
                                }
                            }
                        }
                    }
                }
            } else {
                val labelWidth = maxOf(CompareLabelWidth, maxWidth / (view.options.size + COMPARE_LABEL_SHARE) * COMPARE_LABEL_SHARE)
                Column {
                    Row(Modifier.height(IntrinsicSize.Min)) {
                        Box(Modifier.width(labelWidth))
                        view.options.forEachIndexed { i, option ->
                            val best = view.recommended == i
                            Column(
                                Modifier.weight(FILL).fillMaxHeight()
                                    .then(if (best) Modifier.clip(RoundedCornerShape(topStart = GapL, topEnd = GapL)).background(colors.surfaceContainerLow) else Modifier)
                                    .padding(horizontal = CellPadding, vertical = CellPaddingV),
                                verticalArrangement = Arrangement.spacedBy(GapXs),
                            ) {
                                Text(option, style = rowStyle().copy(fontWeight = TitleWeight), color = colors.onSurface)
                                if (best) RecommendedBadge()
                            }
                        }
                    }
                    view.rows.forEach { row ->
                        Rule()
                        Row(Modifier.height(IntrinsicSize.Min)) {
                            Text(row.label, style = detailStyle(), color = colors.onSurfaceVariant, modifier = Modifier.width(labelWidth).padding(horizontal = CellPadding, vertical = CellPaddingV))
                            view.options.indices.forEach { i ->
                                Text(
                                    value(row, i),
                                    style = rowStyle(),
                                    color = colors.onSurface,
                                    modifier = Modifier.weight(FILL).fillMaxHeight()
                                        .then(if (view.recommended == i) Modifier.background(colors.surfaceContainerLow) else Modifier)
                                        .padding(horizontal = CellPadding, vertical = CellPaddingV),
                                )
                            }
                        }
                    }
                }
            }
        }
        view.verdict?.let { Text(it, style = rowStyle(), color = colors.onSurface) }
    }
}

@Composable
internal fun RecommendedBadge() {
    Text(
        "Recommended",
        style = MaterialTheme.typography.labelSmall.copy(fontWeight = StrongWeight),
        color = MaterialTheme.colorScheme.onPrimaryContainer,
        modifier = Modifier.clip(CircleShape).background(MaterialTheme.colorScheme.primaryContainer).padding(horizontal = GapM, vertical = GapS),
    )
}

private val DotSize = 28.dp
private val StepLine = 2.dp
private val StepGap = 16.dp

@Composable
internal fun StepperViewBlock(view: ChatView.Stepper) {
    val colors = MaterialTheme.colorScheme
    ViewFrame(view.title) {
        Column {
            view.steps.forEachIndexed { i, step ->
                val last = i == view.steps.lastIndex
                Row(Modifier.fillMaxWidth().height(IntrinsicSize.Min), horizontalArrangement = Arrangement.spacedBy(GapL)) {
                    Column(Modifier.width(DotSize).fillMaxHeight(), horizontalAlignment = Alignment.CenterHorizontally) {
                        val (fill, ring, ink) = when (step.status) {
                            ChatView.StepStatus.Done -> Triple(colors.primary, null, colors.onPrimary)
                            ChatView.StepStatus.Current -> Triple(colors.primaryContainer, colors.primary, colors.onPrimaryContainer)
                            ChatView.StepStatus.Failed -> Triple(colors.errorContainer, null, colors.onErrorContainer)
                            else -> Triple(Color.Transparent, colors.outlineVariant, colors.onSurfaceVariant)
                        }
                        Box(
                            Modifier.size(DotSize).clip(CircleShape).background(fill).then(if (ring != null) Modifier.border(GapS, ring, CircleShape) else Modifier),
                            contentAlignment = Alignment.Center,
                        ) {
                            when (step.status) {
                                ChatView.StepStatus.Done -> NomiIcon(NomiIcons.Check, null, tint = ink, size = CheckSize)
                                ChatView.StepStatus.Failed -> NomiIcon(NomiIcons.Close, null, tint = ink, size = CheckSize)
                                else -> Text("${i + 1}", style = smallStyle().copy(fontWeight = TitleWeight), color = ink)
                            }
                        }
                        if (!last) {
                            Box(
                                Modifier.padding(vertical = GapS).width(StepLine).weight(FILL).clip(RoundedCornerShape(Hairline))
                                    .background(if (step.status == ChatView.StepStatus.Done) colors.primary else colors.outlineVariant),
                            )
                        }
                    }
                    Column(
                        Modifier.weight(FILL).padding(top = TextNudge, bottom = if (last) 0.dp else StepGap).semantics(mergeDescendants = true) {
                            contentDescription = "${step.title} (${step.status.spoken})"
                        },
                        verticalArrangement = Arrangement.spacedBy(GapS),
                    ) {
                        val quiet = step.status == ChatView.StepStatus.Upcoming || step.status == ChatView.StepStatus.Skipped
                        Text(
                            step.title,
                            style = rowStyle().copy(
                                fontWeight = if (quiet) FontWeight.Medium else StrongWeight,
                                textDecoration = if (step.status == ChatView.StepStatus.Skipped) TextDecoration.LineThrough else null,
                            ),
                            color = if (quiet) colors.onSurfaceVariant else colors.onSurface,
                        )
                        step.detail?.let { Text(it, style = detailStyle(), color = colors.onSurfaceVariant) }
                    }
                }
            }
        }
    }
}

/** One width for every row's "when" (the web's grid column), so the dots line up. */
private val WhenWidth = 84.dp
private val EventDot = 12.dp
private val EventRing = 3.dp

@Composable
internal fun TimelineViewBlock(view: ChatView.Timeline, timeZone: TimeZone) {
    val colors = MaterialTheme.colorScheme
    val whens = view.events.map { formatWhen(it.at, timeZone) }
    ViewFrame(view.title) {
        Column {
            view.events.forEachIndexed { i, event ->
                val w = whens[i]
                // A day is said once: later events that day show only their time.
                val day = if (i > 0 && whens[i - 1].day == w.day && w.time != null) null else w.day
                val last = i == view.events.lastIndex
                Row(Modifier.fillMaxWidth().height(IntrinsicSize.Min), horizontalArrangement = Arrangement.spacedBy(GapL)) {
                    Column(Modifier.width(WhenWidth), horizontalAlignment = Alignment.End, verticalArrangement = Arrangement.spacedBy(Hairline)) {
                        day?.let { Text(it, style = smallStyle().copy(fontWeight = StrongWeight), color = colors.onSurfaceVariant, maxLines = 1) }
                        w.time?.let { Text(it, style = rowStyle().copy(fontWeight = StrongWeight), color = colors.onSurface, maxLines = 1) }
                    }
                    Column(Modifier.width(EventDot).fillMaxHeight(), horizontalAlignment = Alignment.CenterHorizontally) {
                        Box(Modifier.padding(top = DotNudge).size(EventDot).clip(CircleShape).background(colors.surfaceContainerLowest).border(EventRing, colors.primary, CircleShape))
                        if (!last) Box(Modifier.width(StepLine).weight(FILL).background(colors.outlineVariant))
                    }
                    Column(Modifier.weight(FILL).padding(bottom = if (last) 0.dp else StepGap), verticalArrangement = Arrangement.spacedBy(GapS)) {
                        Text(event.title, style = rowStyle().copy(fontWeight = StrongWeight), color = colors.onSurface)
                        event.detail?.let { Text(it, style = detailStyle(), color = colors.onSurfaceVariant) }
                    }
                }
            }
        }
    }
}

private const val ANSWER_SP = 34f
private const val ANSWER_TRACKING_EM = -0.025f
private const val BRAND_LINE_HEIGHT = 1.1f

@Composable
internal fun toneInk(tone: Tone): Color = when (tone) {
    Tone.Positive -> MaterialTheme.colorScheme.primary
    Tone.Negative -> MaterialTheme.colorScheme.error
    Tone.Warning -> MaterialTheme.colorScheme.tertiary
    Tone.Neutral -> MaterialTheme.colorScheme.onSurface
}

@Composable
internal fun brandFigure(size: Float) = MaterialTheme.typography.displaySmall.copy(
    fontSize = size.sp,
    lineHeight = (size * BRAND_LINE_HEIGHT).sp,
    fontWeight = FontWeight(BRAND_WEIGHT),
    letterSpacing = ANSWER_TRACKING_EM.em,
)

/** One result that should stand out: a large value, what it is, a line of context. */
@Composable
internal fun AnswerViewBlock(view: ChatView.Answer, timeZone: TimeZone) {
    val colors = MaterialTheme.colorScheme
    ViewFrame(null) {
        Column(verticalArrangement = Arrangement.spacedBy(GapXs)) {
            view.label?.let { Text(it, style = detailStyle(), color = colors.onSurfaceVariant) }
            Text(formatValue(view.value, view.format, timeZone), style = brandFigure(ANSWER_SP), color = toneInk(view.tone))
            view.detail?.let { Text(it, style = rowStyle(), color = colors.onSurfaceVariant) }
        }
    }
}

private val ChartHeight = 200.dp
private val ChartPadTop = 8.dp
private val ChartPadRight = 8.dp
private val ChartPadBottom = 28.dp
private val ChartPadLeft = 52.dp
private val LabelSpacing = 56.dp
private val MaxBar = 28.dp
private val MinBar = 2.dp
private val BarCorner = 4.dp
private const val BAR_GAP_PX = 2f
private const val BAND_SHARE = 0.72f
private const val LINE_WIDTH_DP = 2.5f
private const val POINT_RADIUS_DP = 3.5f
private const val POINTS_HIDE_ABOVE = 20
private const val LABEL_MAX = 10
private const val TICK_SP = 11f
private val DashIntervals = floatArrayOf(2f, 4f)
private val DonutSize = 180.dp
private const val DONUT_RADIUS = 70f
private const val DONUT_STROKE = 26f
private const val DONUT_BOX = 180f
private const val DONUT_GAP = 2f
private const val DONUT_TOTAL_SP = 18f
private const val FULL_CIRCLE = 360f
private const val START_ANGLE = -90f
private const val DONUT_FADE = 0.7f
private const val PERCENT = 100
private const val MIN_PLOT_PX = 60f
private const val DONUT_TOTAL_RISE = 4f
private const val DONUT_CAPTION_DROP = 16f

/** The chart's series colours: the chart colour (primary), then tertiary, secondary, error, outline, inverse primary. */
@Composable
private fun seriesColors(): List<Color> = with(MaterialTheme.colorScheme) { listOf(primary, tertiary, secondary, error, outline, inversePrimary) }

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun ChartViewBlock(view: ChatView.Chart) {
    val colors = MaterialTheme.colorScheme
    val palette = seriesColors()
    val measurer = rememberTextMeasurer()
    val tickStyle = TextStyle(fontSize = TICK_SP.sp, color = colors.onSurfaceVariant, fontFamily = NomiTheme.metaStyle.fontFamily)
    ViewFrame(view.title) {
        if (view.kind == ChatView.ChartKind.Donut) {
            val values = view.labels.indices.map { i -> (view.series.firstOrNull()?.values?.getOrNull(i) ?: 0.0).coerceAtLeast(0.0) }
            val total = values.sum()
            val n = view.labels.size
            val sliceColors = view.labels.indices.map { i -> lerpColor(colors.surfaceContainerLowest, colors.primary, 1f - (i * DONUT_FADE) / maxOf(1, n - 1)) }
            FlowRow(horizontalArrangement = Arrangement.spacedBy(GapXxl), verticalArrangement = Arrangement.spacedBy(GapXl), itemVerticalAlignment = Alignment.CenterVertically) {
                val totalStyle = TextStyle(fontSize = DONUT_TOTAL_SP.sp, fontWeight = FontWeight(BRAND_WEIGHT), color = colors.onSurface)
                val track = colors.surfaceContainerHigh
                Canvas(Modifier.size(DonutSize).semantics { contentDescription = view.title ?: view.series.firstOrNull()?.name.orEmpty() }) {
                    val unit = size.minDimension / DONUT_BOX
                    val radius = DONUT_RADIUS * unit
                    val stroke = DONUT_STROKE * unit
                    val topLeft = Offset(center.x - radius, center.y - radius)
                    val arc = Size(radius * 2, radius * 2)
                    drawArc(track, 0f, FULL_CIRCLE, false, topLeft, arc, style = Stroke(stroke))
                    var start = START_ANGLE
                    values.forEachIndexed { i, v ->
                        val sweep = if (total > 0) (v / total).toFloat() * FULL_CIRCLE else 0f
                        if (sweep > 0f) drawArc(sliceColors[i], start, (sweep - DONUT_GAP).coerceAtLeast(0f), false, topLeft, arc, style = Stroke(stroke))
                        start += sweep
                    }
                    centeredText(measurer, formatTick(total, view.format), totalStyle, Offset(center.x, center.y - DONUT_TOTAL_RISE * unit))
                    centeredText(measurer, view.series.firstOrNull()?.name.orEmpty(), tickStyle, Offset(center.x, center.y + DONUT_CAPTION_DROP * unit))
                }
                Column(Modifier.widthIn(min = LegendMinWidth), verticalArrangement = Arrangement.spacedBy(GapSm)) {
                    view.labels.forEachIndexed { i, label ->
                        LegendRow(sliceColors[i], label, if (total > 0) "${(values[i] / total * PERCENT).toInt()}%" else "0%")
                    }
                }
            }
        } else {
            val all = view.series.flatMap { it.values }
            val low = minOf(0.0, all.minOrNull() ?: 0.0)
            val ticks = chartScale((all.maxOrNull() ?: 0.0).coerceAtLeast(0.0) - low).map { it + low }
            val top = ticks.lastOrNull() ?: 1.0
            Canvas(Modifier.fillMaxWidth().height(ChartHeight).semantics { contentDescription = view.title ?: view.series.joinToString(", ") { it.name } }) {
                val left = ChartPadLeft.toPx()
                val plotW = (size.width - left - ChartPadRight.toPx()).coerceAtLeast(MIN_PLOT_PX)
                val plotH = size.height - ChartPadTop.toPx() - ChartPadBottom.toPx()
                fun y(v: Double) = (ChartPadTop.toPx() + plotH - ((v - low) / (top - low).let { if (it == 0.0) 1.0 else it }) * plotH).toFloat()
                val band = plotW / view.labels.size.coerceAtLeast(1)
                fun x(i: Int) = left + band * i + band / 2
                ticks.forEach { t ->
                    drawLine(colors.outlineVariant, Offset(left, y(t)), Offset(left + plotW, y(t)), Hairline.toPx(), pathEffect = if (t == 0.0) null else PathEffect.dashPathEffect(DashIntervals.map { it * density }.toFloatArray()))
                    val label = measurer.measure(AnnotatedString(formatTick(t, view.format)), tickStyle)
                    drawText(label, topLeft = Offset(left - GapM.toPx() - label.size.width, y(t) - label.size.height / 2))
                }
                val every = maxOf(1, kotlin.math.ceil(view.labels.size / maxOf(1f, (plotW / LabelSpacing.toPx()).toInt().toFloat())).toInt())
                view.labels.forEachIndexed { i, raw ->
                    if (i % every == 0) {
                        val text = if (raw.length > LABEL_MAX) raw.take(LABEL_MAX - 1) + "…" else raw
                        val label = measurer.measure(AnnotatedString(text), tickStyle)
                        drawText(label, topLeft = Offset(x(i) - label.size.width / 2, size.height - GapM.toPx() - label.size.height))
                    }
                }
                if (view.kind == ChatView.ChartKind.Bar) {
                    val barW = ((band * BAND_SHARE) / view.series.size.coerceAtLeast(1)).coerceIn(MinBar.toPx(), MaxBar.toPx())
                    view.series.forEachIndexed { s, series ->
                        series.values.forEachIndexed { i, v ->
                            val bx = x(i) - barW * view.series.size / 2 + s * barW
                            val yTop = minOf(y(v), y(0.0))
                            val h = maxOf(1f, kotlin.math.abs(y(0.0) - y(v)))
                            val r = minOf(BarCorner.toPx(), (barW - BAR_GAP_PX) / 2)
                            drawRoundRect(palette[s % palette.size], Offset(bx + BAR_GAP_PX / 2, yTop), Size(barW - BAR_GAP_PX, h), CornerRadius(r, r))
                        }
                    }
                } else {
                    view.series.forEachIndexed { s, series ->
                        val color = palette[s % palette.size]
                        val path = Path()
                        series.values.forEachIndexed { i, v -> if (i == 0) path.moveTo(x(i), y(v)) else path.lineTo(x(i), y(v)) }
                        drawPath(path, color, style = Stroke(LINE_WIDTH_DP * density, cap = StrokeCap.Round, join = StrokeJoin.Round))
                        if (view.labels.size <= POINTS_HIDE_ABOVE) series.values.forEachIndexed { i, v -> drawCircle(color, POINT_RADIUS_DP * density, Offset(x(i), y(v))) }
                    }
                }
            }
            if (view.series.size > 1) {
                FlowRow(horizontalArrangement = Arrangement.spacedBy(GapXl), verticalArrangement = Arrangement.spacedBy(GapSm)) {
                    view.series.forEachIndexed { s, series -> LegendRow(palette[s % palette.size], series.name, null) }
                }
            }
        }
    }
}

private val Swatch = 10.dp

@Composable
private fun LegendRow(color: Color, name: String, value: String?) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(GapM)) {
        Box(Modifier.size(Swatch).clip(RoundedCornerShape(TextNudge)).background(color))
        Text(name, style = smallStyle(), color = MaterialTheme.colorScheme.onSurfaceVariant)
        value?.let { Text(it, style = smallStyle().copy(fontWeight = StrongWeight), color = MaterialTheme.colorScheme.onSurface) }
    }
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.centeredText(measurer: TextMeasurer, text: String, style: TextStyle, at: Offset) {
    val layout = measurer.measure(AnnotatedString(text), style)
    drawText(layout, topLeft = Offset(at.x - layout.size.width / 2, at.y - layout.size.height / 2))
}

/** `color-mix(in srgb, to share%, from)`. */
private fun lerpColor(from: Color, to: Color, share: Float) = Color(
    red = from.red + (to.red - from.red) * share,
    green = from.green + (to.green - from.green) * share,
    blue = from.blue + (to.blue - from.blue) * share,
    alpha = 1f,
)

private val LinkTextPaddingH = 16.dp
private val LinkTextPaddingV = 14.dp

/** A page worth opening, as a preview card; the whole card opens it in the browser. */
@Composable
internal fun LinkViewBlock(view: ChatView.LinkPreview) {
    val colors = MaterialTheme.colorScheme
    val uri = LocalUriHandler.current
    val host = remember(view.url) { view.url.substringAfter("://").substringBefore('/').removePrefix("www.").ifEmpty { view.url } }
    val shape = MaterialTheme.shapes.extraLarge
    Column(
        Modifier.fillMaxWidth().clip(shape).background(colors.surfaceContainerLowest).border(Hairline, colors.outlineVariant, shape)
            .clickable(role = Role.Button, onClickLabel = "Open ${view.title ?: host}") { uri.openUri(view.url) }
            .padding(horizontal = LinkTextPaddingH, vertical = LinkTextPaddingV),
        verticalArrangement = Arrangement.spacedBy(GapXs),
    ) {
        Text(view.site ?: host, style = smallStyle(), color = colors.onSurfaceVariant)
        Text(view.title ?: view.url, style = rowStyle().copy(fontWeight = TitleWeight), color = colors.primary)
        view.description?.let { Text(it, style = detailStyle(), color = colors.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis) }
    }
}

private val TaskAvatar = 32.dp

/** Work going on in the background: what it is, who has it, where it stands, how far along. */
@Composable
internal fun TaskViewBlock(view: ChatView.Task, agent: String?) {
    val colors = MaterialTheme.colorScheme
    val who = view.agent ?: agent
    val identity = who?.let { agentIdentity(it) } ?: NomiIdentity
    ViewFrame(null) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(GapL)) {
            AgentShape(identity, TaskAvatar, working = view.status == ChatView.TaskStatus.Running, face = identity == NomiIdentity)
            Column(Modifier.weight(FILL), verticalArrangement = Arrangement.spacedBy(GapS)) {
                Text(view.title, style = rowStyle().copy(fontWeight = TitleWeight), color = colors.onSurface)
                view.detail?.let { Text(it, style = detailStyle(), color = colors.onSurfaceVariant) }
            }
            val (bg, ink) = when (view.status) {
                ChatView.TaskStatus.Running -> colors.primaryContainer to colors.onPrimaryContainer
                ChatView.TaskStatus.Done -> colors.secondaryContainer to colors.onSecondaryContainer
                ChatView.TaskStatus.Failed -> colors.errorContainer to colors.onErrorContainer
                ChatView.TaskStatus.Queued -> colors.surfaceContainerHigh to colors.onSurfaceVariant
            }
            Text(view.status.label, style = smallStyle().copy(fontWeight = StrongWeight), color = ink, modifier = Modifier.clip(CircleShape).background(bg).padding(horizontal = GapMd, vertical = GapXs))
        }
        val progress = view.progress
        if (progress != null && view.status != ChatView.TaskStatus.Failed) {
            val shown = if (view.status == ChatView.TaskStatus.Done) 1f else progress.toFloat().coerceIn(0f, 1f)
            NomiWavyProgress(progress = { shown }, contentDescription = view.title, modifier = Modifier.fillMaxWidth())
        }
    }
}

private val ShellMaxHeight = 320.dp
private const val COPIED_RESET_MS = 1500L
private const val SHELL_SP = 13f
private const val SHELL_LINE_HEIGHT = 1.5f
private const val SHELL_BAR_SP = 12f
private const val BAR_INK_ALPHA = 0.14f
private const val CWD_ALPHA = 0.7f
private val ShellCopySize = 44.dp

/** A command and what it printed, as a terminal; opens scrolled to the end of a long output. Copy takes the command. */
@Composable
internal fun ShellViewBlock(view: ChatView.Shell) {
    val colors = MaterialTheme.colorScheme
    val clipboard = LocalClipboardManager.current
    var copied by remember { mutableStateOf(false) }
    LaunchedEffect(copied) {
        if (copied) {
            delay(COPIED_RESET_MS)
            copied = false
        }
    }
    val mono = NomiTheme.metaStyle.fontFamily
    val ink = colors.inverseOnSurface
    Column(Modifier.fillMaxWidth().clip(MaterialTheme.shapes.large).background(colors.inverseSurface)) {
        Row(Modifier.fillMaxWidth().padding(start = InsetM, end = GapXs), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(GapMd)) {
            Text(view.cwd.orEmpty(), style = TextStyle(fontFamily = mono, fontSize = SHELL_BAR_SP.sp), color = ink.copy(alpha = CWD_ALPHA), maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(FILL))
            view.exitCode?.let { code ->
                val failed = code != 0
                Text(
                    "exit $code",
                    style = TextStyle(fontFamily = mono, fontSize = SHELL_BAR_SP.sp),
                    color = if (failed) colors.onError else ink,
                    modifier = Modifier.clip(CircleShape).background(if (failed) colors.error else ink.copy(alpha = BAR_INK_ALPHA)).padding(horizontal = GapM, vertical = GapS),
                )
            }
            IconButton(onClick = { clipboard.setText(AnnotatedString(view.command)); copied = true }, modifier = Modifier.size(ShellCopySize)) {
                NomiIcon(if (copied) NomiIcons.Check else NomiIcons.Copy, contentDescription = if (copied) "Copied" else "Copy command", tint = ink, size = GapXl)
            }
        }
        HorizontalDivider(thickness = Hairline, color = ink.copy(alpha = BAR_INK_ALPHA))
        val scroll = rememberScrollState()
        LaunchedEffect(view.output) { scroll.scrollTo(scroll.maxValue) }
        val screen = androidx.compose.ui.text.buildAnnotatedString {
            pushStyle(androidx.compose.ui.text.SpanStyle(color = colors.inversePrimary)); append("$ "); pop()
            pushStyle(androidx.compose.ui.text.SpanStyle(fontWeight = TitleWeight)); append(view.command); pop()
            view.output?.let { append("\n"); append(it) }
        }
        Text(
            screen,
            style = TextStyle(fontFamily = mono, fontSize = SHELL_SP.sp, lineHeight = (SHELL_SP * SHELL_LINE_HEIGHT).sp),
            color = ink,
            modifier = Modifier.fillMaxWidth().heightIn(max = ShellMaxHeight).verticalScroll(scroll).padding(horizontal = InsetM, vertical = GapL),
        )
    }
}
