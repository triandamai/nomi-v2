package ai.nomi.app.screens.money

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
import ai.nomi.designsystem.component.NomiWavyProgress
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.layout.NomiPage
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.network.BudgetDto
import ai.nomi.network.CategoryTotalDto
import ai.nomi.network.DayTotalDto
import ai.nomi.network.MoneySummaryDto
import ai.nomi.network.TransactionDto
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle

private const val ZERO_CENTS = 0L
private const val CENTS_DIVISOR = 100.0
private const val CENTS_MULTIPLIER = 100L
private const val FILL_FRACTION = 1f
private const val ZERO_FRACTION = 0f
private const val MIN_AXIS_DIVISOR = 1.0
private const val BAR_SPACING_RATIO = 0.25f
private const val BAR_CORNER_RADIUS_PX = 4f
private const val PERCENT_MULTIPLIER = 100
private const val FONT_WEIGHT_BOLD = 700
private const val FONT_WEIGHT_SEMI = 600
private const val TITLE_SP = 28f
private const val HERO_SP = 32f
private const val SUB_SP = 14f
private const val BODY_SP = 15f
private const val BAR_CHART_ALPHA = 0.85f
private const val EMPTY_DAY_ALPHA = 0.15f
private const val ZERO_INDEX = 0

private val SectionSpacing = 18.dp
private val InnerCardPadding = 16.dp
private val SmallSpacing = 8.dp
private val TinySpacing = 4.dp
private val MediumSpacing = 12.dp
private val ChartHeight = 140.dp
private val CategoryBarHeight = 8.dp
private val IconBadgeSize = 36.dp
private val DialogPadding = 20.dp
private val DialogCorner = 20.dp
private val BottomListPadding = 32.dp

private fun formatCents(cents: Long): String {
    val dollars = cents.toDouble() / CENTS_DIVISOR
    val isNegative = cents < ZERO_CENTS
    val absVal = kotlin.math.abs(dollars)
    val integerPart = absVal.toLong()
    val fractionPart = ((absVal - integerPart) * CENTS_MULTIPLIER).toLong()
    val formattedFraction = fractionPart.toString().padStart(2, '0')
    val prefix = if (isNegative) "-$" else "$"
    return "$prefix$integerPart.$formattedFraction"
}

@Composable
fun MoneyScreen(
    viewModel: MoneyViewModel,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    NomiPage(
        title = "Money",
        modifier = modifier,
        actions = {
            NomiButton(
                onClick = viewModel::openAddTransaction,
                variant = NomiButtonVariant.Filled,
                size = NomiButtonSize.Small,
            ) {
                NomiIcon(NomiIcons.Plus, contentDescription = null, size = 16.dp)
                Spacer(Modifier.width(SmallSpacing))
                Text("Add")
            }
        },
    ) {
        if (state.loading && state.summary == null) {
            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                NomiLoadingIndicator(contentDescription = "Loading money data")
            }
        } else {
            val summary = state.summary
            if (summary != null) {
                MoneyContent(
                    summary = summary,
                    onPreviousMonth = viewModel::onPreviousMonth,
                    onNextMonth = viewModel::onNextMonth,
                    onOpenBudget = viewModel::openSetBudget,
                )
            } else if (state.error != null) {
                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Text(
                        state.error ?: "Failed to load",
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodyLarge,
                    )
                }
            }
        }

        if (state.addTransactionOpen) {
            AddTransactionDialog(
                onDismiss = viewModel::closeAddTransaction,
                onConfirm = { desc, amount, cat, kind, date ->
                    viewModel.createTransaction(desc, amount, cat, kind, date)
                },
                saving = state.saving,
            )
        }

        if (state.setBudgetOpen) {
            SetBudgetDialog(
                categories = state.summary?.byCategory?.map { it.category } ?: emptyList(),
                onDismiss = viewModel::closeSetBudget,
                onConfirm = { cat, limit ->
                    viewModel.setBudget(cat, limit)
                },
                saving = state.saving,
            )
        }
    }
}

@Composable
private fun MoneyContent(
    summary: MoneySummaryDto,
    onPreviousMonth: () -> Unit,
    onNextMonth: () -> Unit,
    onOpenBudget: () -> Unit,
) {
    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(bottom = BottomListPadding),
        verticalArrangement = Arrangement.spacedBy(SectionSpacing),
    ) {
        // Month Selector
        MonthHeader(
            month = summary.month,
            onPrevious = onPreviousMonth,
            onNext = onNextMonth,
        )

        // Summary Hero Card
        SpendingHeroCard(summary)

        // Daily Chart
        if (summary.byDay.isNotEmpty()) {
            NomiSectionHeader("Daily Spending")
            DailySpendingChart(summary.byDay)
        }

        // Category Breakdown
        if (summary.byCategory.isNotEmpty() || summary.budgets.isNotEmpty()) {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                NomiSectionHeader("Categories & Budgets")
                NomiButton(
                    onClick = onOpenBudget,
                    variant = NomiButtonVariant.Tonal,
                    size = NomiButtonSize.Small,
                ) {
                    Text("Budget")
                }
            }
            CategoryBreakdownCard(summary.byCategory, summary.budgets, summary.totalCents)
        }

        // Transactions List
        NomiSectionHeader("Transactions (${summary.transactions.size})")
        if (summary.transactions.isEmpty()) {
            NomiListPanel {
                Box(Modifier.fillMaxWidth().padding(InnerCardPadding), contentAlignment = Alignment.Center) {
                    Text(
                        "No transactions in this period",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        } else {
            NomiListPanel {
                summary.transactions.forEach { tx ->
                    TransactionRow(tx)
                }
            }
        }
    }
}

@Composable
private fun MonthHeader(
    month: String,
    onPrevious: () -> Unit,
    onNext: () -> Unit,
) {
    Row(
        Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconButton(onClick = onPrevious) {
            NomiIcon(NomiIcons.ChevronLeft, contentDescription = "Previous month")
        }
        Text(
            text = month,
            style = MaterialTheme.typography.titleLarge.copy(
                fontWeight = FontWeight(FONT_WEIGHT_BOLD),
                fontSize = 20.sp,
            ),
            color = MaterialTheme.colorScheme.onSurface,
        )
        IconButton(onClick = onNext) {
            NomiIcon(NomiIcons.ChevronRight, contentDescription = "Next month")
        }
    }
}

@Composable
private fun SpendingHeroCard(summary: MoneySummaryDto) {
    NomiCard(Modifier.fillMaxWidth()) {
        Column(
            Modifier.fillMaxWidth().padding(InnerCardPadding),
            verticalArrangement = Arrangement.spacedBy(MediumSpacing),
        ) {
            Text(
                "Total Spent",
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                formatCents(summary.totalCents),
                style = MaterialTheme.typography.displayMedium.copy(
                    fontWeight = FontWeight(FONT_WEIGHT_BOLD),
                    fontSize = HERO_SP.sp,
                ),
                color = MaterialTheme.colorScheme.onSurface,
            )

            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(SectionSpacing),
            ) {
                if (summary.incomeCents > ZERO_CENTS) {
                    Column {
                        Text(
                            "Income",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        Text(
                            formatCents(summary.incomeCents),
                            style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight(FONT_WEIGHT_SEMI)),
                            color = MaterialTheme.colorScheme.primary,
                        )
                    }
                }
                if (summary.previousTotalCents > ZERO_CENTS) {
                    val delta = summary.totalCents - summary.previousTotalCents
                    val percent = ((delta.toDouble() / summary.previousTotalCents) * PERCENT_MULTIPLIER).toLong()
                    val deltaText = if (delta >= ZERO_CENTS) "+$percent% vs last mo" else "$percent% vs last mo"
                    Column {
                        Text(
                            "Previous Month",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        Text(
                            deltaText,
                            style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight(FONT_WEIGHT_SEMI)),
                            color = if (delta > ZERO_CENTS) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary,
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun DailySpendingChart(byDay: List<DayTotalDto>) {
    val colors = MaterialTheme.colorScheme
    val barColor = colors.primary
    val maxCents = byDay.maxOfOrNull { it.cents }?.coerceAtLeast(1L) ?: 1L

    NomiCard(Modifier.fillMaxWidth()) {
        Box(
            Modifier
                .fillMaxWidth()
                .height(ChartHeight)
                .padding(InnerCardPadding),
        ) {
            Canvas(Modifier.fillMaxSize()) {
                val totalBars = byDay.size
                if (totalBars == ZERO_INDEX) return@Canvas

                val slotWidth = size.width / totalBars.toFloat()
                val barWidth = slotWidth * (FILL_FRACTION - BAR_SPACING_RATIO)
                val spacing = slotWidth * BAR_SPACING_RATIO

                byDay.forEachIndexed { index, day ->
                    val barHeight = if (day.cents > ZERO_CENTS) {
                        (day.cents.toFloat() / maxCents.toFloat()) * size.height
                    } else {
                        4f
                    }
                    val left = index * slotWidth + (spacing / 2f)
                    val top = size.height - barHeight
                    val color = if (day.cents > ZERO_CENTS) barColor.copy(alpha = BAR_CHART_ALPHA) else barColor.copy(alpha = EMPTY_DAY_ALPHA)

                    drawRoundRect(
                        color = color,
                        topLeft = Offset(left, top),
                        size = Size(barWidth, barHeight),
                        cornerRadius = CornerRadius(BAR_CORNER_RADIUS_PX, BAR_CORNER_RADIUS_PX),
                    )
                }
            }
        }
    }
}

@Composable
private fun CategoryBreakdownCard(
    categories: List<CategoryTotalDto>,
    budgets: List<BudgetDto>,
    totalCents: Long,
) {
    val budgetMap = remember(budgets) { budgets.associateBy { it.category } }

    NomiCard(Modifier.fillMaxWidth()) {
        Column(
            Modifier.fillMaxWidth().padding(InnerCardPadding),
            verticalArrangement = Arrangement.spacedBy(MediumSpacing),
        ) {
            categories.forEach { cat ->
                val budget = budgetMap[cat.category]
                val share = if (totalCents > ZERO_CENTS) {
                    ((cat.cents.toDouble() / totalCents.toDouble()) * PERCENT_MULTIPLIER).toInt()
                } else {
                    0
                }
                Column(verticalArrangement = Arrangement.spacedBy(TinySpacing)) {
                    Row(
                        Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                    ) {
                        Text(
                            cat.category,
                            style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight(FONT_WEIGHT_SEMI)),
                            color = MaterialTheme.colorScheme.onSurface,
                        )
                        Text(
                            "${formatCents(cat.cents)} ($share%)",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    val progressFraction = if (budget != null && budget.limitCents > ZERO_CENTS) {
                        (cat.cents.toFloat() / budget.limitCents.toFloat()).coerceIn(ZERO_FRACTION, FILL_FRACTION)
                    } else {
                        (share.toFloat() / 100f).coerceIn(ZERO_FRACTION, FILL_FRACTION)
                    }
                    NomiWavyProgress(
                        progress = { progressFraction },
                        contentDescription = "${cat.category} progress",
                        modifier = Modifier.fillMaxWidth().height(CategoryBarHeight),
                    )
                    if (budget != null) {
                        Text(
                            "Budget: ${formatCents(budget.limitCents)}",
                            style = MaterialTheme.typography.labelSmall,
                            color = if (cat.cents > budget.limitCents) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun TransactionRow(tx: TransactionDto) {
    val isExpense = tx.kind != "income"
    val amountFormatted = if (isExpense) "-${formatCents(tx.amountCents)}" else "+${formatCents(tx.amountCents)}"

    NomiRow(
        title = tx.description.ifBlank { tx.category },
        onClick = null,
        line = "${tx.category} · ${tx.occurredAt.take(10)}",
        meta = amountFormatted,
        leading = {
            Box(
                Modifier
                    .size(IconBadgeSize)
                    .clip(CircleShape)
                    .background(MaterialTheme.colorScheme.surfaceContainerHighest),
                contentAlignment = Alignment.Center,
            ) {
                NomiIcon(NomiIcons.Wallet, contentDescription = null, tint = MaterialTheme.colorScheme.primary)
            }
        },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun AddTransactionDialog(
    onDismiss: () -> Unit,
    onConfirm: (description: String, amount: String, category: String, kind: String, occurredAt: String) -> Unit,
    saving: Boolean,
) {
    var description by remember { mutableStateOf("") }
    var amount by remember { mutableStateOf("") }
    var category by remember { mutableStateOf("General") }
    var kind by remember { mutableStateOf("expense") }
    var occurredAt by remember { mutableStateOf("2026-10-19") }

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
                    "Add Transaction",
                    style = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight(FONT_WEIGHT_BOLD)),
                    color = MaterialTheme.colorScheme.onSurface,
                )

                Row(horizontalArrangement = Arrangement.spacedBy(SmallSpacing)) {
                    NomiFilterChip(
                        label = "Expense",
                        selected = kind == "expense",
                        onClick = { kind = "expense" },
                    )
                    NomiFilterChip(
                        label = "Income",
                        selected = kind == "income",
                        onClick = { kind = "income" },
                    )
                }

                NomiTextField(
                    value = description,
                    onValueChange = { description = it },
                    label = "Description",
                    placeholder = "e.g. Groceries",
                    modifier = Modifier.fillMaxWidth(),
                )

                NomiTextField(
                    value = amount,
                    onValueChange = { amount = it },
                    label = "Amount",
                    placeholder = "e.g. 12.50",
                    modifier = Modifier.fillMaxWidth(),
                )

                NomiTextField(
                    value = category,
                    onValueChange = { category = it },
                    label = "Category",
                    placeholder = "Food, Rent, Tech",
                    modifier = Modifier.fillMaxWidth(),
                )

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
                            if (amount.isNotBlank()) {
                                onConfirm(description, amount, category, kind, occurredAt)
                            }
                        },
                        variant = NomiButtonVariant.Filled,
                        enabled = !saving && amount.isNotBlank(),
                    ) {
                        Text(if (saving) "Saving..." else "Save")
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun SetBudgetDialog(
    categories: List<String>,
    onDismiss: () -> Unit,
    onConfirm: (category: String, limit: String) -> Unit,
    saving: Boolean,
) {
    var category by remember { mutableStateOf(categories.firstOrNull() ?: "Food") }
    var limit by remember { mutableStateOf("") }

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
                    "Set Budget",
                    style = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight(FONT_WEIGHT_BOLD)),
                    color = MaterialTheme.colorScheme.onSurface,
                )

                NomiTextField(
                    value = category,
                    onValueChange = { category = it },
                    label = "Category",
                    placeholder = "Food, Transport...",
                    modifier = Modifier.fillMaxWidth(),
                )

                NomiTextField(
                    value = limit,
                    onValueChange = { limit = it },
                    label = "Monthly Limit",
                    placeholder = "e.g. 500",
                    modifier = Modifier.fillMaxWidth(),
                )

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
                            if (limit.isNotBlank()) {
                                onConfirm(category, limit)
                            }
                        },
                        variant = NomiButtonVariant.Filled,
                        enabled = !saving && limit.isNotBlank(),
                    ) {
                        Text(if (saving) "Saving..." else "Set")
                    }
                }
            }
        }
    }
}
