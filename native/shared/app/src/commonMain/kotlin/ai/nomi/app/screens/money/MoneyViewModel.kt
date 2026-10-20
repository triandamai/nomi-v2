package ai.nomi.app.screens.money

import ai.nomi.data.MoneyRepository
import ai.nomi.network.CategoryTotalDto
import ai.nomi.network.CreateBudgetRequestDto
import ai.nomi.network.CreateTransactionRequestDto
import ai.nomi.network.DayTotalDto
import ai.nomi.network.MoneySummaryDto
import ai.nomi.network.TransactionDto
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

private const val ZERO_CENTS = 0L
private const val CENTS_PER_UNIT = 100L
private const val CENTS_PER_UNIT_DOUBLE = 100.0
private const val MONTH_SHIFT_BACKWARD = -1
private const val MONTH_SHIFT_FORWARD = 1
private const val MONTHS_IN_YEAR = 12
private const val MONTH_PAD_LENGTH = 2
private const val MONTH_INDEX_OFFSET = 1
private const val EMPTY_STRING = ""
private const val KIND_EXPENSE = "expense"
private const val KIND_INCOME = "income"

data class MoneyUiState(
    val loading: Boolean = true,
    val currentMonth: String = EMPTY_STRING,
    val summary: MoneySummaryDto? = null,
    val error: String? = null,
    val addTransactionOpen: Boolean = false,
    val setBudgetOpen: Boolean = false,
    val saving: Boolean = false,
)

class MoneyViewModel(
    private val repository: MoneyRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(MoneyUiState())
    val uiState: StateFlow<MoneyUiState> = _uiState.asStateFlow()

    init {
        loadMonth(null)
    }

    fun loadMonth(month: String?) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(loading = true, error = null)
            val result = repository.getSummary(month)
            result.onSuccess { summary ->
                _uiState.value = _uiState.value.copy(
                    loading = false,
                    currentMonth = summary.month,
                    summary = summary,
                    error = null,
                )
            }.onFailure { error ->
                _uiState.value = _uiState.value.copy(
                    loading = false,
                    error = error.message ?: "Failed to load money data",
                )
            }
        }
    }

    fun shiftMonth(delta: Int) {
        val current = _uiState.value.currentMonth.takeIf { it.isNotBlank() } ?: return
        val parts = current.split('-')
        val year = parts.getOrNull(0)?.toIntOrNull() ?: return
        val monthNum = parts.getOrNull(1)?.toIntOrNull() ?: return
        val totalMonths = year * MONTHS_IN_YEAR + (monthNum - MONTH_INDEX_OFFSET) + delta
        val nextYear = totalMonths / MONTHS_IN_YEAR
        val nextMonth = (totalMonths % MONTHS_IN_YEAR) + MONTH_INDEX_OFFSET
        val formatted = "$nextYear-${nextMonth.toString().padStart(MONTH_PAD_LENGTH, '0')}"
        loadMonth(formatted)
    }

    fun onPreviousMonth() = shiftMonth(MONTH_SHIFT_BACKWARD)
    fun onNextMonth() = shiftMonth(MONTH_SHIFT_FORWARD)

    fun openAddTransaction() {
        _uiState.value = _uiState.value.copy(addTransactionOpen = true)
    }

    fun closeAddTransaction() {
        _uiState.value = _uiState.value.copy(addTransactionOpen = false)
    }

    fun openSetBudget() {
        _uiState.value = _uiState.value.copy(setBudgetOpen = true)
    }

    fun closeSetBudget() {
        _uiState.value = _uiState.value.copy(setBudgetOpen = false)
    }

    fun createTransaction(
        description: String,
        amountText: String,
        category: String,
        kind: String,
        occurredAt: String,
    ) {
        val amountValue = amountText.toDoubleOrNull() ?: return
        val amountCents = (amountValue * CENTS_PER_UNIT_DOUBLE).toLong()
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(saving = true)
            val req = CreateTransactionRequestDto(
                occurredAt = occurredAt,
                amountCents = amountCents,
                category = category.trim(),
                description = description.trim(),
                kind = kind,
            )
            repository.createTransaction(req).onSuccess {
                _uiState.value = _uiState.value.copy(saving = false, addTransactionOpen = false)
                loadMonth(_uiState.value.currentMonth)
            }.onFailure { error ->
                _uiState.value = _uiState.value.copy(
                    saving = false,
                    error = error.message ?: "Failed to save transaction",
                )
            }
        }
    }

    fun setBudget(category: String, limitText: String) {
        val limitValue = limitText.toDoubleOrNull() ?: return
        val limitCents = (limitValue * CENTS_PER_UNIT_DOUBLE).toLong()
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(saving = true)
            val req = CreateBudgetRequestDto(
                category = category.trim(),
                limitCents = limitCents,
            )
            repository.setBudget(req).onSuccess {
                _uiState.value = _uiState.value.copy(saving = false, setBudgetOpen = false)
                loadMonth(_uiState.value.currentMonth)
            }.onFailure { error ->
                _uiState.value = _uiState.value.copy(
                    saving = false,
                    error = error.message ?: "Failed to set budget",
                )
            }
        }
    }
}
