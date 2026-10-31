package ai.nomi.network

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

// --- Money ---

@Serializable
data class CategoryTotalDto(
    val category: String,
    val cents: Long,
    val count: Long,
)

@Serializable
data class DayTotalDto(
    val date: String,
    val cents: Long,
)

@Serializable
data class TransactionItemDto(
    val name: String,
    val quantity: Double = 1.0,
    @SerialName("unit_amount_cents") val unitAmountCents: Long? = null,
    @SerialName("amount_cents") val amountCents: Long,
    val category: String? = null,
)

@Serializable
data class TransactionDto(
    val id: String,
    @SerialName("occurred_at") val occurredAt: String,
    @SerialName("amount_cents") val amountCents: Long,
    val category: String,
    val description: String,
    val kind: String,
    val source: String = "manual",
    @SerialName("created_at") val createdAt: String,
    val items: List<TransactionItemDto> = emptyList(),
)

@Serializable
data class BudgetDto(
    val category: String,
    @SerialName("limit_cents") val limitCents: Long,
    @SerialName("spent_cents") val spentCents: Long = 0,
)

@Serializable
data class MoneySummaryDto(
    val month: String,
    @SerialName("start_day") val startDay: Int = 1,
    @SerialName("period_start") val periodStart: String,
    @SerialName("period_end") val periodEnd: String,
    val timezone: String = "UTC",
    @SerialName("total_cents") val totalCents: Long = 0,
    @SerialName("income_cents") val incomeCents: Long = 0,
    @SerialName("previous_total_cents") val previousTotalCents: Long = 0,
    @SerialName("transaction_count") val transactionCount: Long = 0,
    @SerialName("by_category") val byCategory: List<CategoryTotalDto> = emptyList(),
    @SerialName("by_day") val byDay: List<DayTotalDto> = emptyList(),
    val transactions: List<TransactionDto> = emptyList(),
    val months: List<String> = emptyList(),
    val budgets: List<BudgetDto> = emptyList(),
)

@Serializable
data class CreateTransactionRequestDto(
    @SerialName("occurred_at") val occurredAt: String,
    @SerialName("amount_cents") val amountCents: Long,
    val category: String,
    val description: String,
    val kind: String = "expense",
    val items: List<TransactionItemDto> = emptyList(),
)

@Serializable
data class CreateBudgetRequestDto(
    val category: String,
    @SerialName("limit_cents") val limitCents: Long,
)

interface MoneyApi {
    suspend fun summary(month: String? = null): MoneySummaryDto
    /** The backend answers with no body; reload the summary to see it. */
    suspend fun createTransaction(request: CreateTransactionRequestDto)
    suspend fun setBudget(request: CreateBudgetRequestDto)
}

// --- Reminders ---

@Serializable
data class ReminderDto(
    val id: String,
    val title: String,
    val notes: String? = null,
    @SerialName("due_at") val dueAt: String,
    val recurrence: String? = null,
    val status: String = "active",
    @SerialName("created_by") val createdBy: String = "user",
    @SerialName("created_at") val createdAt: String? = null,
)

@Serializable
data class ScheduledTaskDto(
    val id: String,
    val label: String,
    @SerialName("run_at") val runAt: String,
    val recurrence: String? = null,
    val agent: String = "nomi",
    @SerialName("session_id") val sessionId: String,
)

@Serializable
data class RemindersResponseDto(
    val timezone: String = "UTC",
    val upcoming: List<ReminderDto> = emptyList(),
    val past: List<ReminderDto> = emptyList(),
    @SerialName("scheduled_tasks") val scheduledTasks: List<ScheduledTaskDto> = emptyList(),
)

@Serializable
data class CreateReminderRequestDto(
    val title: String,
    @SerialName("due_at") val dueAt: String,
    val recurrence: String? = null,
    val notes: String? = null,
)

@Serializable
data class ReminderActionRequestDto(
    val action: String,
    val minutes: Long? = null,
)

interface RemindersApi {
    suspend fun list(): RemindersResponseDto
    suspend fun create(request: CreateReminderRequestDto): ReminderDto
    suspend fun action(id: String, request: ReminderActionRequestDto)
    suspend fun cancelScheduledTask(id: String)
}

// --- Memory ---

@Serializable
data class MemoryItemDto(
    val id: String,
    val content: String,
    val weight: Double = 1.0,
    val kind: String = "fact",
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
    val uses: Long = 0,
    @SerialName("last_used_at") val lastUsedAt: String? = null,
    @SerialName("needs_check") val needsCheck: Boolean = false,
    val embedding: List<Float> = emptyList(),
)

@Serializable
data class MemoryListResponseDto(
    val memories: List<MemoryItemDto> = emptyList(),
)

@Serializable
data class EditMemoryRequestDto(
    val content: String,
    val kind: String? = null,
)

interface MemoryApi {
    suspend fun list(): MemoryListResponseDto
    suspend fun delete(id: String)
    suspend fun edit(id: String, request: EditMemoryRequestDto)
    suspend fun confirm(id: String)
}

// --- Connections ---

@Serializable
data class GoogleOverviewDto(
    val configured: Boolean = false,
    val email: String? = null,
    val services: List<String> = emptyList(),
)

@Serializable
data class HealthConnectionDto(
    val connected: Boolean = false,
    @SerialName("account_name") val accountName: String? = null,
    @SerialName("connected_at") val connectedAt: String? = null,
)

@Serializable
data class ConnectionsOverviewDto(
    val google: GoogleOverviewDto = GoogleOverviewDto(),
    val hevy: HealthConnectionDto? = null,
)

interface ConnectionsApi {
    suspend fun overview(): ConnectionsOverviewDto
}

// --- Usage ---

@Serializable
data class UsagePlanDto(
    val name: String = "Free",
    @SerialName("monthly_tokens") val monthlyTokens: Long = 0,
)

@Serializable
data class UsageBriefDto(
    val plan: UsagePlanDto = UsagePlanDto(),
    val month: String = "",
    @SerialName("tokens_used") val tokensUsed: Long = 0,
    @SerialName("can_upgrade") val canUpgrade: Boolean = false,
)

interface UsageApi {
    suspend fun brief(): UsageBriefDto
}

// --- Projects ---

@Serializable
data class ProjectDto(
    val id: String,
    @SerialName("session_id") val sessionId: String = "",
    val name: String,
    val description: String? = null,
    val status: String = "planning",
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
    @SerialName("created_by") val createdBy: String = "",
    @SerialName("open_issues") val openIssues: Long = 0,
)

@Serializable
data class ListProjectsResponse(
    @SerialName("org_id") val orgId: String? = null,
    val projects: List<ProjectDto> = emptyList(),
)

interface ProjectsApi {
    suspend fun list(orgId: String? = null): List<ProjectDto>
}

