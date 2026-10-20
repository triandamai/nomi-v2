package ai.nomi.app.screens.memory

import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiCard
import ai.nomi.designsystem.component.NomiFilterChip
import ai.nomi.designsystem.component.NomiListPanel
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiRow
import ai.nomi.designsystem.component.NomiSectionHeader
import ai.nomi.designsystem.component.NomiTextField
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.layout.NomiPage
import ai.nomi.network.MemoryItemDto
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.BasicAlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlin.math.abs
import kotlin.math.sqrt

private const val FONT_WEIGHT_BOLD = 700
private const val FONT_WEIGHT_SEMI = 600
private const val CANVAS_DOT_RADIUS_PX = 6f
private const val CANVAS_SELECTED_RING_PX = 14f
private const val CANVAS_TAP_DISTANCE_SQ = 900f
private const val STAT_VALUE_SP = 24f
private const val MAX_STRENGTH_PIPS = 3
private const val DEFAULT_EMBEDDING_FALLBACK_DIVISOR = 1000f

private val SectionSpacing = 18.dp
private val InnerCardPadding = 16.dp
private val SmallSpacing = 8.dp
private val MediumSpacing = 12.dp
private val CanvasHeight = 180.dp
private val PipSize = 6.dp
private val DialogPadding = 20.dp
private val DialogCorner = 20.dp
private val BottomListPadding = 32.dp

private fun kindColor(kind: String, defaultColor: Color): Color = when (kind.lowercase()) {
    "fact" -> Color(0xFF38BDF8)
    "preference" -> Color(0xFFFBBF24)
    "goal" -> Color(0xFF34D399)
    "habit" -> Color(0xFFA78BFA)
    else -> defaultColor
}

@Composable
fun MemoryScreen(
    viewModel: MemoryViewModel,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    NomiPage(
        title = "Memory",
        modifier = modifier,
    ) {
        if (state.loading && state.memories.isEmpty()) {
            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                NomiLoadingIndicator(contentDescription = "Loading memory")
            }
        } else {
            MemoryContent(
                state = state,
                onSelectMemory = viewModel::selectMemory,
                onQueryChange = viewModel::setQuery,
                onKindFilter = viewModel::setKindFilter,
                onSortChange = viewModel::setSort,
                onEdit = viewModel::openEdit,
                onConfirm = viewModel::confirmMemory,
                onDelete = viewModel::deleteMemory,
            )
        }

        if (state.editOpen && state.editingMemory != null) {
            EditMemoryDialog(
                memory = state.editingMemory!!,
                onDismiss = viewModel::closeEdit,
                onSave = viewModel::saveEdit,
                saving = state.saving,
            )
        }
    }
}

@Composable
private fun MemoryContent(
    state: MemoryUiState,
    onSelectMemory: (String?) -> Unit,
    onQueryChange: (String) -> Unit,
    onKindFilter: (String?) -> Unit,
    onSortChange: (MemorySort) -> Unit,
    onEdit: (MemoryItemDto) -> Unit,
    onConfirm: (String) -> Unit,
    onDelete: (String) -> Unit,
) {
    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(bottom = BottomListPadding),
        verticalArrangement = Arrangement.spacedBy(SectionSpacing),
    ) {
        // Stats Row
        MemoryStatsRow(state.memories)

        // 2D Vector Map Canvas
        if (state.memories.isNotEmpty()) {
            NomiSectionHeader("Memory Map")
            MemoryVectorCanvas(
                memories = state.memories,
                selectedId = state.selectedId,
                onSelectMemory = onSelectMemory,
            )
        }

        // Search & Filter Toolbar
        MemoryFilterToolbar(
            query = state.query,
            onQueryChange = onQueryChange,
            selectedKind = state.kindFilter,
            onKindFilter = onKindFilter,
            currentSort = state.sort,
            onSortChange = onSortChange,
        )

        // Memory List
        NomiSectionHeader("Memories (${state.filteredMemories.size})")
        if (state.filteredMemories.isEmpty()) {
            NomiListPanel {
                Box(Modifier.fillMaxWidth().padding(InnerCardPadding), contentAlignment = Alignment.Center) {
                    Text(
                        if (state.query.isNotBlank()) "No memories match \"${state.query}\"" else "No memories stored yet",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        } else {
            NomiListPanel {
                state.filteredMemories.forEach { memory ->
                    MemoryItemRow(
                        memory = memory,
                        onEdit = { onEdit(memory) },
                        onConfirm = { onConfirm(memory.id) },
                        onDelete = { onDelete(memory.id) },
                    )
                }
            }
        }
    }
}

@Composable
private fun MemoryStatsRow(memories: List<MemoryItemDto>) {
    val totalCount = memories.size
    val totalRecalled = memories.sumOf { it.uses }
    val needsCheckCount = memories.count { it.needsCheck }

    Row(
        Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(SmallSpacing),
    ) {
        StatCard(
            title = "Memories",
            value = totalCount.toString(),
            modifier = Modifier.weight(1f),
        )
        StatCard(
            title = "Recalled",
            value = totalRecalled.toString(),
            modifier = Modifier.weight(1f),
        )
        StatCard(
            title = "To Confirm",
            value = needsCheckCount.toString(),
            modifier = Modifier.weight(1f),
            alert = needsCheckCount > 0,
        )
    }
}

@Composable
private fun StatCard(
    title: String,
    value: String,
    modifier: Modifier = Modifier,
    alert: Boolean = false,
) {
    NomiCard(modifier) {
        Column(
            Modifier.fillMaxWidth().padding(InnerCardPadding),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Text(
                title,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                value,
                style = MaterialTheme.typography.titleLarge.copy(
                    fontWeight = FontWeight(FONT_WEIGHT_BOLD),
                    fontSize = STAT_VALUE_SP.sp,
                ),
                color = if (alert) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurface,
            )
        }
    }
}

@Composable
private fun MemoryVectorCanvas(
    memories: List<MemoryItemDto>,
    selectedId: String?,
    onSelectMemory: (String?) -> Unit,
) {
    val primaryColor = MaterialTheme.colorScheme.primary
    val surfaceColor = MaterialTheme.colorScheme.surfaceContainerLowest

    val selectedMemory = remember(selectedId, memories) {
        memories.firstOrNull { it.id == selectedId }
    }

    NomiCard(Modifier.fillMaxWidth()) {
        Column(Modifier.fillMaxWidth().padding(InnerCardPadding)) {
            Box(
                Modifier
                    .fillMaxWidth()
                    .height(CanvasHeight)
                    .clip(RoundedCornerShape(12.dp))
                    .background(surfaceColor),
            ) {
                Canvas(
                    Modifier
                        .fillMaxSize()
                        .pointerInput(memories) {
                            detectTapGestures { tapOffset ->
                                val w = size.width.toFloat()
                                val h = size.height.toFloat()
                                val nearest = memories.minByOrNull { m ->
                                    val (px, py) = calculatePoint(m, w, h)
                                    val dx = px - tapOffset.x
                                    val dy = py - tapOffset.y
                                    dx * dx + dy * dy
                                }
                                if (nearest != null) {
                                    val (nx, ny) = calculatePoint(nearest, w, h)
                                    val distSq = (nx - tapOffset.x) * (nx - tapOffset.x) + (ny - tapOffset.y) * (ny - tapOffset.y)
                                    if (distSq <= CANVAS_TAP_DISTANCE_SQ) {
                                        onSelectMemory(nearest.id)
                                    } else {
                                        onSelectMemory(null)
                                    }
                                }
                            }
                        },
                ) {
                    val w = size.width
                    val h = size.height

                    memories.forEach { memory ->
                        val (px, py) = calculatePoint(memory, w, h)
                        val color = kindColor(memory.kind, primaryColor)
                        val isSelected = memory.id == selectedId

                        if (isSelected) {
                            drawCircle(
                                color = color.copy(alpha = 0.35f),
                                radius = CANVAS_SELECTED_RING_PX,
                                center = Offset(px, py),
                            )
                        }

                        drawCircle(
                            color = color,
                            radius = if (isSelected) CANVAS_DOT_RADIUS_PX + 2f else CANVAS_DOT_RADIUS_PX,
                            center = Offset(px, py),
                        )
                    }
                }
            }

            if (selectedMemory != null) {
                Spacer(Modifier.height(SmallSpacing))
                Text(
                    text = selectedMemory.content,
                    style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight(FONT_WEIGHT_SEMI)),
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

private fun calculatePoint(memory: MemoryItemDto, width: Float, height: Float): Pair<Float, Float> {
    val padding = 24f
    val innerW = width - (padding * 2f)
    val innerH = height - (padding * 2f)

    return if (memory.embedding.size >= 2) {
        val ex = ((memory.embedding[0] + 1f) / 2f).coerceIn(0f, 1f)
        val ey = ((memory.embedding[1] + 1f) / 2f).coerceIn(0f, 1f)
        (padding + ex * innerW) to (padding + ey * innerH)
    } else {
        val hash = abs(memory.id.hashCode())
        val fx = (hash % 100) / 100f
        val fy = ((hash / 100) % 100) / 100f
        (padding + fx * innerW) to (padding + fy * innerH)
    }
}

@Composable
private fun MemoryFilterToolbar(
    query: String,
    onQueryChange: (String) -> Unit,
    selectedKind: String?,
    onKindFilter: (String?) -> Unit,
    currentSort: MemorySort,
    onSortChange: (MemorySort) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(SmallSpacing)) {
        NomiTextField(
            value = query,
            onValueChange = onQueryChange,
            label = "Search",
            placeholder = "Search memories...",
            modifier = Modifier.fillMaxWidth(),
        )

        Row(
            Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()),
            horizontalArrangement = Arrangement.spacedBy(SmallSpacing),
        ) {
            NomiFilterChip(
                label = "All",
                selected = selectedKind == null,
                onClick = { onKindFilter(null) },
            )
            listOf("Fact", "Preference", "Goal", "Habit").forEach { kind ->
                NomiFilterChip(
                    label = kind,
                    selected = selectedKind.equals(kind, ignoreCase = true),
                    onClick = { onKindFilter(if (selectedKind.equals(kind, ignoreCase = true)) null else kind) },
                )
            }
        }
    }
}

@Composable
private fun MemoryItemRow(
    memory: MemoryItemDto,
    onEdit: () -> Unit,
    onConfirm: () -> Unit,
    onDelete: () -> Unit,
) {
    val color = kindColor(memory.kind, MaterialTheme.colorScheme.primary)

    Row(
        Modifier
            .fillMaxWidth()
            .padding(horizontal = InnerCardPadding, vertical = SmallSpacing),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(SmallSpacing),
    ) {
        Box(
            Modifier
                .size(12.dp)
                .clip(CircleShape)
                .background(color),
        )
        Column(Modifier.weight(1f)) {
            Text(
                memory.content,
                style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight(FONT_WEIGHT_SEMI)),
                color = MaterialTheme.colorScheme.onSurface,
            )
            Text(
                "${memory.kind.replaceFirstChar { it.uppercase() }} · used ${memory.uses} times",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        if (memory.needsCheck) {
            NomiButton(
                onClick = onConfirm,
                variant = NomiButtonVariant.Tonal,
                size = NomiButtonSize.Small,
            ) {
                Text("Confirm")
            }
            Spacer(Modifier.width(SmallSpacing))
        }
        IconButton(onClick = onEdit) {
            NomiIcon(NomiIcons.Pencil, contentDescription = "Edit memory")
        }
        IconButton(onClick = onDelete) {
            NomiIcon(NomiIcons.Delete, contentDescription = "Delete memory")
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun EditMemoryDialog(
    memory: MemoryItemDto,
    onDismiss: () -> Unit,
    onSave: (content: String, kind: String?) -> Unit,
    saving: Boolean,
) {
    var content by remember { mutableStateOf(memory.content) }
    var kind by remember { mutableStateOf(memory.kind) }

    val kindOptions = listOf("fact", "preference", "goal", "habit")

    BasicAlertDialog(onDismissRequest = onDismiss) {
        Surface(
            shape = RoundedCornerShape(DialogCorner),
            color = MaterialTheme.colorScheme.surfaceContainerHigh,
            modifier = Modifier.padding(DialogPadding),
        ) {
            Column(
                Modifier.padding(DialogPadding),
                verticalArrangement = Arrangement.spacedBy(MediumSpacing),
            ) {
                Text(
                    "Edit Memory",
                    style = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight(FONT_WEIGHT_BOLD)),
                    color = MaterialTheme.colorScheme.onSurface,
                )

                NomiTextField(
                    value = content,
                    onValueChange = { content = it },
                    label = "Memory",
                    placeholder = "Memory content",
                    modifier = Modifier.fillMaxWidth(),
                )

                Row(horizontalArrangement = Arrangement.spacedBy(SmallSpacing)) {
                    kindOptions.forEach { opt ->
                        NomiFilterChip(
                            label = opt.replaceFirstChar { it.uppercase() },
                            selected = kind == opt,
                            onClick = { kind = opt },
                        )
                    }
                }

                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    NomiButton(onClick = onDismiss, variant = NomiButtonVariant.Text) {
                        Text("Cancel")
                    }
                    Spacer(Modifier.width(SmallSpacing))
                    NomiButton(
                        onClick = {
                            if (content.isNotBlank()) {
                                onSave(content, kind)
                            }
                        },
                        variant = NomiButtonVariant.Filled,
                        enabled = !saving && content.isNotBlank(),
                    ) {
                        Text(if (saving) "Saving..." else "Save")
                    }
                }
            }
        }
    }
}
