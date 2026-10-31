package ai.nomi.app.screens.projects

import ai.nomi.data.ContainerStat
import ai.nomi.data.DataQueryResult
import ai.nomi.designsystem.component.ButtonGroupOption
import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonGroup
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiFilterChip
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiSwitch
import ai.nomi.designsystem.component.NomiTextField
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.feature.chat.formatBytes
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

private const val FILL = 1f
private const val TITLE_WEIGHT = 700
private const val PERCENT = 100
private const val NO_CHANGE = 0
private const val LOG_LINES_SHOWN = 80
private const val ROWS_SHOWN = 50
private const val CELL_MAX_CHARS = 40
private const val ONE_DECIMAL = 10.0
private const val THOUSAND = 1_000.0
private const val SOURCE_DEV = "development"
private const val SOURCE_PROD = "production"

private val SectionGap = 16.dp
private val RowGap = 8.dp
private val RowPaddingV = 10.dp
private val StatPadding = 12.dp
private val StatGap = 8.dp
private val SparkHeight = 56.dp
private val SparkStroke = 2.dp
private val CellWidth = 120.dp
private val CellPadding = 8.dp
private val LogsMaxHeight = 280.dp
private val ResultMaxHeight = 320.dp
private val CodePadding = 12.dp

/** `DomainPanel.svelte`: production's address, and moving it to another free name. */
@Composable
internal fun ColumnScope.DomainSheet(
    state: AddressState,
    onEdit: (String) -> Unit,
    onSave: () -> Unit,
    onRetry: () -> Unit,
) {
    when (val loaded = state.address) {
        Loadable.Loading -> SheetLoading("Loading address")
        is Loadable.Failed -> SheetRetry(loaded.message, onRetry)
        is Loadable.Ready -> {
            val address = loaded.value
            val hostname = address.hostname
            ToolTitle("Domain", hostname ?: "Gets an address on its first publish")
            if (hostname == null) {
                SheetNote("Publish the project first: production gets its address then.")
                return
            }
            if (!address.canEdit) {
                SheetNote("Only the project's admins can change its address.")
                return
            }
            val check = state.check
            NomiTextField(
                value = state.draft,
                onValueChange = onEdit,
                label = "Name",
                supportingText = when {
                    state.checking -> "Checking…"
                    check?.available == true -> "${check.hostname ?: state.draft} is free"
                    check != null -> check.message ?: "That name is taken"
                    else -> hostname.substringAfter('.').let { "Your app lives at <name>.$it" }
                },
                isError = state.check?.available == false,
                modifier = Modifier.fillMaxWidth(),
            )
            check?.takeIf { !it.available && it.suggestions.isNotEmpty() }?.suggestions?.let { free ->
                Text("Free instead", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = RowGap))
                Row(Modifier.horizontalScroll(rememberScrollState()).padding(vertical = RowGap), horizontalArrangement = Arrangement.spacedBy(RowGap)) {
                    free.forEach { name -> NomiFilterChip(label = name, selected = false, onClick = { onEdit(name) }) }
                }
            }
            state.error?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error) }
            Row(Modifier.fillMaxWidth().padding(top = RowGap), horizontalArrangement = Arrangement.End) {
                NomiButton(
                    onClick = onSave,
                    enabled = !state.saving && state.draft.isNotBlank() && state.draft != address.subdomain && state.check?.available != false,
                ) { Text(if (state.saving) "Saving…" else "Change address") }
            }
        }
    }
}

/** `MonitorPanel.svelte`: production's traffic, its containers and its latest logs. */
@Composable
internal fun ColumnScope.MonitorSheet(state: HealthState, onRange: (String) -> Unit, onRetry: () -> Unit) {
    ToolTitle("Monitor", "How production is doing")
    NomiButtonGroup(
        options = TRAFFIC_RANGES.map { ButtonGroupOption(it, it) },
        selected = state.range,
        onSelect = onRange,
        modifier = Modifier.fillMaxWidth().padding(bottom = SectionGap),
    )
    when (val loaded = state.health) {
        Loadable.Loading -> SheetLoading("Loading production")
        is Loadable.Failed -> SheetRetry(loaded.message, onRetry)
        is Loadable.Ready -> Column(Modifier.verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(SectionGap)) {
            val health = loaded.value
            val traffic = health.traffic
            if (traffic == null && health.containers.isEmpty() && health.logLines.isEmpty()) {
                SheetNote(health.logNote ?: "Nothing runs in production yet. Publish to see its traffic here.")
                return@Column
            }
            traffic?.let { t ->
                Row(horizontalArrangement = Arrangement.spacedBy(StatGap)) {
                    Stat("Visitors", t.totals.visitors, t.previous.visitors, Modifier.weight(FILL))
                    Stat("Requests", t.totals.requests, t.previous.requests, Modifier.weight(FILL))
                }
                Row(horizontalArrangement = Arrangement.spacedBy(StatGap)) {
                    Stat("Server errors", t.totals.errors5xx, t.previous.errors5xx, Modifier.weight(FILL))
                    Stat("p95", t.totals.p95Ms, t.previous.p95Ms, Modifier.weight(FILL), suffix = " ms")
                }
                if (t.requestsPerBucket.any { it > 0 }) Spark(t.requestsPerBucket)
            }
            if (health.containers.isNotEmpty()) {
                Section("Containers") { health.containers.forEach { ContainerRow(it) } }
            }
            Section("Logs") {
                if (health.logLines.isEmpty()) {
                    Text(health.logNote ?: "No log lines yet.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                } else {
                    Text(
                        health.logLines.takeLast(LOG_LINES_SHOWN).joinToString("\n"),
                        style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
                        color = MaterialTheme.colorScheme.onSurface,
                        softWrap = false,
                        modifier = Modifier
                            .fillMaxWidth()
                            .heightIn(max = LogsMaxHeight)
                            .clip(MaterialTheme.shapes.medium)
                            .background(MaterialTheme.colorScheme.surfaceContainerLowest)
                            .verticalScroll(rememberScrollState())
                            .horizontalScroll(rememberScrollState())
                            .padding(CodePadding),
                    )
                }
            }
        }
    }
}

@Composable
private fun Stat(label: String, value: Double?, previous: Double?, modifier: Modifier, suffix: String = "") {
    val colors = MaterialTheme.colorScheme
    Column(
        modifier.clip(MaterialTheme.shapes.medium).background(colors.surfaceContainerLowest).padding(StatPadding),
        verticalArrangement = Arrangement.spacedBy(RowGap / 2),
    ) {
        Text(label, style = MaterialTheme.typography.labelMedium, color = colors.onSurfaceVariant)
        Text(
            value?.let { compact(it) + suffix } ?: "—",
            style = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight(TITLE_WEIGHT)),
            color = colors.onSurface,
        )
        change(value, previous)?.let { Text(it, style = NomiTheme.metaStyle, color = colors.onSurfaceVariant) }
    }
}

/** "+12% vs before", or null when either period is unknown or empty. */
private fun change(value: Double?, previous: Double?): String? {
    if (value == null || previous == null || previous == 0.0) return null
    val percent = ((value - previous) / previous * PERCENT).toInt()
    return when {
        percent > NO_CHANGE -> "+$percent% vs before"
        percent < NO_CHANGE -> "$percent% vs before"
        else -> "Same as before"
    }
}

/** 1234 → "1.2k", 31 → "31". */
private fun compact(value: Double): String =
    if (value >= THOUSAND) "${(value / THOUSAND * ONE_DECIMAL).toInt() / ONE_DECIMAL}k" else value.toInt().toString()

/** Requests per bucket as a single line (`--nomi-chart`'s single-series role is the primary colour here). */
@Composable
private fun Spark(points: List<Double>) {
    val color = MaterialTheme.colorScheme.primary
    val max = points.maxOrNull()?.takeIf { it > 0 } ?: return
    Canvas(Modifier.fillMaxWidth().height(SparkHeight)) {
        if (points.size < 2) return@Canvas
        val step = size.width / (points.size - 1)
        val stroke = SparkStroke.toPx()
        points.zipWithNext().forEachIndexed { index, (from, to) ->
            drawLine(
                color,
                Offset(index * step, size.height - (from / max).toFloat() * size.height),
                Offset((index + 1) * step, size.height - (to / max).toFloat() * size.height),
                strokeWidth = stroke,
                cap = StrokeCap.Round,
            )
        }
    }
}

@Composable
private fun ContainerRow(container: ContainerStat) {
    Row(Modifier.fillMaxWidth().padding(vertical = RowPaddingV / 2), verticalAlignment = Alignment.CenterVertically) {
        Text(container.service, style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.SemiBold), color = MaterialTheme.colorScheme.onSurface, modifier = Modifier.weight(FILL))
        val cpu = container.cpuPercent?.let { "CPU ${(it * ONE_DECIMAL).toInt() / ONE_DECIMAL}%" }
        val memory = container.memoryBytes?.let { used ->
            "RAM ${formatBytes(used)}" + (container.memoryLimitBytes?.let { " / ${formatBytes(it)}" } ?: "")
        }
        Text(listOfNotNull(cpu, memory).joinToString(" · ").ifEmpty { "Not reporting" }, style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

/** `DatabasePanel.svelte` / `RelayPanel.svelte`: the tables, and a query box with its answer. */
@Composable
internal fun ColumnScope.DatabaseSheet(
    state: DataState,
    relay: Boolean,
    onSource: (production: Boolean) -> Unit,
    onSql: (String) -> Unit,
    onRun: () -> Unit,
    onRetry: () -> Unit,
    onRetrySetup: () -> Unit,
) {
    ToolTitle(if (relay) "Relay" else "Database", if (state.production) "Production data" else "Development data")
    NomiButtonGroup(
        options = listOf(ButtonGroupOption(SOURCE_DEV, "Development"), ButtonGroupOption(SOURCE_PROD, "Production")),
        selected = if (state.production) SOURCE_PROD else SOURCE_DEV,
        onSelect = { onSource(it == SOURCE_PROD) },
        modifier = Modifier.fillMaxWidth().padding(bottom = SectionGap),
    )
    when (val loaded = state.data) {
        Loadable.Loading -> SheetLoading("Loading tables")
        is Loadable.Failed -> SheetRetry(loaded.message, onRetry)
        is Loadable.Ready -> Column(Modifier.verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(SectionGap)) {
            val data = loaded.value
            if (data.setupState != null) {
                SheetNote(
                    when (data.setupState) {
                        "failed" -> "Setting up its backend failed${data.setupError?.let { ": $it" } ?: "."}"
                        "sleeping" -> "Its backend is asleep; it wakes on the next visit."
                        else -> "Its backend is being set up."
                    },
                )
                if (data.setupState == "failed") NomiButton(onClick = onRetrySetup, variant = NomiButtonVariant.Tonal) { Text("Try again") }
                return@Column
            }
            Section("Tables") {
                if (data.tables.isEmpty()) {
                    Text("No tables yet.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                data.tables.forEach { table ->
                    Row(Modifier.fillMaxWidth().clickable(role = Role.Button, onClickLabel = "Query ${table.name}") { onSql("select * from ${table.name} limit $ROWS_SHOWN") }.padding(vertical = RowPaddingV / 2)) {
                        Text(table.name, style = MaterialTheme.typography.bodyMedium.copy(fontFamily = FontFamily.Monospace), color = MaterialTheme.colorScheme.onSurface, modifier = Modifier.weight(FILL))
                        Text(
                            listOfNotNull(table.rows?.let { "~$it rows" }, table.detail).joinToString(" · "),
                            style = NomiTheme.metaStyle,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }
            Section("Query") {
                NomiTextField(
                    value = state.sql,
                    onValueChange = onSql,
                    label = "SQL",
                    placeholder = "select * from …",
                    singleLine = false,
                    supportingText = if (state.production) "Production is read-only here." else null,
                    modifier = Modifier.fillMaxWidth(),
                )
                Row(Modifier.fillMaxWidth().padding(top = RowGap), horizontalArrangement = Arrangement.End) {
                    NomiButton(onClick = onRun, enabled = state.sql.isNotBlank() && !state.running) { Text(if (state.running) "Running…" else "Run") }
                }
                state.queryError?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error) }
                state.result?.let { ResultTable(it) }
            }
        }
    }
}

@Composable
private fun ResultTable(result: DataQueryResult) {
    val colors = MaterialTheme.colorScheme
    val summary = buildString {
        if (result.columns.isEmpty()) append("${result.rowsAffected ?: 0} rows changed") else append("${result.rows.size} rows")
        if (result.truncated) append(" (more not shown)")
        append(" · ${result.tookMs.toInt()} ms")
    }
    Text(summary, style = NomiTheme.metaStyle, color = colors.onSurfaceVariant, modifier = Modifier.padding(vertical = RowGap))
    if (result.columns.isEmpty()) return
    Column(
        Modifier
            .fillMaxWidth()
            .heightIn(max = ResultMaxHeight)
            .clip(MaterialTheme.shapes.medium)
            .background(colors.surfaceContainerLowest)
            .verticalScroll(rememberScrollState())
            .horizontalScroll(rememberScrollState()),
    ) {
        Row(Modifier.background(colors.surfaceContainer)) {
            result.columns.forEach { Cell(it, header = true) }
        }
        result.rows.take(ROWS_SHOWN).forEach { row ->
            Row { row.forEach { Cell(it ?: "null", header = false, faint = it == null) } }
            HorizontalDivider(Modifier.width(CellWidth * result.columns.size), color = colors.outlineVariant)
        }
    }
}

@Composable
private fun Cell(text: String, header: Boolean, faint: Boolean = false) {
    Text(
        text.take(CELL_MAX_CHARS),
        style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace, fontWeight = if (header) FontWeight.Bold else FontWeight.Normal),
        color = if (faint) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onSurface,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = Modifier.width(CellWidth).padding(CellPadding),
    )
}

/** `SettingsPanel.svelte`: the showcase, taking production offline, moving and deleting the project. */
@Composable
internal fun ColumnScope.SettingsSheet(
    state: SettingsState,
    projectName: String,
    canPublish: Boolean,
    onBlurb: (String) -> Unit,
    onShowcase: (Boolean) -> Unit,
    onTakeOffline: () -> Unit,
    onSearch: (String) -> Unit,
    onTransfer: (orgId: String, typedName: String) -> Unit,
    onDelete: (typedName: String) -> Unit,
) {
    var confirming by rememberSaveable { mutableStateOf<String?>(null) }
    var typed by rememberSaveable { mutableStateOf("") }
    var target by rememberSaveable { mutableStateOf<String?>(null) }
    ToolTitle("Settings", projectName)
    Column(Modifier.verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(SectionGap)) {
        state.error?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error) }
        val showcase = (state.showcase as? Loadable.Ready)?.value
        Section("Showcase") {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("Show this app on the public showcase", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurface, modifier = Modifier.weight(FILL))
                NomiSwitch(checked = showcase?.showcase == true, onCheckedChange = onShowcase, enabled = showcase?.canEdit == true && !state.busy)
            }
            if (showcase?.canEdit == true) {
                NomiTextField(value = state.blurb, onValueChange = onBlurb, label = "Blurb", placeholder = "One line about it", modifier = Modifier.fillMaxWidth())
            }
        }
        if (canPublish) {
            Section("Production") {
                Text("Take the live app down. Publishing again brings it back.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                NomiButton(onClick = onTakeOffline, variant = NomiButtonVariant.Outlined, enabled = !state.busy) { Text("Take offline") }
            }
        }
        Section("Move to another organization") {
            NomiTextField(value = state.search, onValueChange = onSearch, label = "Search organizations", modifier = Modifier.fillMaxWidth())
            state.targets.forEach { org ->
                Row(
                    Modifier.fillMaxWidth().clip(MaterialTheme.shapes.small).clickable(role = Role.Button) {
                        target = org.id
                        confirming = CONFIRM_TRANSFER
                        typed = ""
                    }.padding(vertical = RowPaddingV / 2),
                ) {
                    Text(org.name, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurface, modifier = Modifier.weight(FILL))
                    Text(if (org.personal) "Personal" else org.ownerName.orEmpty(), style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
        Section("Delete") {
            Text("Deletes the project, its files and its publishes for good.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            NomiButton(
                onClick = {
                    confirming = CONFIRM_DELETE
                    typed = ""
                },
                variant = NomiButtonVariant.Outlined,
                colors = ButtonDefaults.outlinedButtonColors(contentColor = MaterialTheme.colorScheme.error),
                enabled = !state.busy,
            ) { Text("Delete project") }
        }
        confirming?.let { which ->
            Section(if (which == CONFIRM_DELETE) "Type the project's name to delete it" else "Type the project's name to move it") {
                NomiTextField(value = typed, onValueChange = { typed = it }, label = projectName, modifier = Modifier.fillMaxWidth())
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(RowGap, Alignment.End)) {
                    NomiButton(onClick = { confirming = null }, variant = NomiButtonVariant.Text) { Text("Cancel") }
                    NomiButton(
                        onClick = { if (which == CONFIRM_DELETE) onDelete(typed) else target?.let { onTransfer(it, typed) } },
                        enabled = typed.trim() == projectName.trim() && !state.busy,
                        size = NomiButtonSize.Small,
                    ) { Text(if (which == CONFIRM_DELETE) "Delete" else "Move") }
                }
            }
        }
    }
}

private const val CONFIRM_DELETE = "delete"
private const val CONFIRM_TRANSFER = "transfer"

@Composable
private fun ToolTitle(title: String, subtitle: String) {
    Column(Modifier.fillMaxWidth().padding(bottom = SectionGap)) {
        Text(title, style = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight(TITLE_WEIGHT)), color = MaterialTheme.colorScheme.onSurface)
        Text(subtitle, style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

@Composable
private fun Section(title: String, content: @Composable ColumnScope.() -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(RowGap)) {
        Text(title, style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.SemiBold), color = MaterialTheme.colorScheme.onSurface)
        content()
    }
}

@Composable
private fun SheetNote(text: String) {
    Text(text, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(vertical = RowGap))
}

@Composable
private fun SheetLoading(label: String) {
    Box(Modifier.fillMaxWidth().padding(SectionGap), contentAlignment = Alignment.Center) { NomiLoadingIndicator(contentDescription = label) }
}

@Composable
private fun SheetRetry(message: String, onRetry: () -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(RowGap)) {
        Text(message, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        NomiButton(onClick = onRetry, variant = NomiButtonVariant.Tonal) { Text("Try again") }
    }
}
