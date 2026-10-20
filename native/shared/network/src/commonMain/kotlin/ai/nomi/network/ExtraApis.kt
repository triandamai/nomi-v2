package ai.nomi.network

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.delete
import io.ktor.client.request.get
import io.ktor.client.request.parameter
import io.ktor.client.request.post
import io.ktor.client.request.put
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.isSuccess
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
    suspend fun createTransaction(request: CreateTransactionRequestDto): TransactionDto
    suspend fun setBudget(request: CreateBudgetRequestDto): BudgetDto
}

class HttpMoneyApi(private val client: HttpClient, private val config: ApiConfig) : MoneyApi {
    override suspend fun summary(month: String?): MoneySummaryDto {
        val response = client.get(config.url("/api/money")) {
            if (month != null) parameter("month", month)
        }
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "money summary failed")
        return response.body()
    }

    override suspend fun createTransaction(request: CreateTransactionRequestDto): TransactionDto {
        val response = client.post(config.url("/api/money/transactions")) {
            contentType(ContentType.Application.Json)
            setBody(request)
        }
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "create transaction failed")
        return response.body()
    }

    override suspend fun setBudget(request: CreateBudgetRequestDto): BudgetDto {
        val response = client.post(config.url("/api/money/budgets")) {
            contentType(ContentType.Application.Json)
            setBody(request)
        }
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "set budget failed")
        return response.body()
    }
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

class HttpRemindersApi(private val client: HttpClient, private val config: ApiConfig) : RemindersApi {
    override suspend fun list(): RemindersResponseDto {
        val response = client.get(config.url("/api/reminders"))
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "list reminders failed")
        return response.body()
    }

    override suspend fun create(request: CreateReminderRequestDto): ReminderDto {
        val response = client.post(config.url("/api/reminders")) {
            contentType(ContentType.Application.Json)
            setBody(request)
        }
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "create reminder failed")
        return response.body()
    }

    override suspend fun action(id: String, request: ReminderActionRequestDto) {
        val response = client.post(config.url("/api/reminders/$id/action")) {
            contentType(ContentType.Application.Json)
            setBody(request)
        }
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "reminder action failed")
    }

    override suspend fun cancelScheduledTask(id: String) {
        val response = client.delete(config.url("/api/reminders/tasks/$id"))
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "cancel task failed")
    }
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

class HttpMemoryApi(private val client: HttpClient, private val config: ApiConfig) : MemoryApi {
    override suspend fun list(): MemoryListResponseDto {
        val response = client.get(config.url("/api/memory"))
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "list memories failed")
        return response.body()
    }

    override suspend fun delete(id: String) {
        val response = client.delete(config.url("/api/memory/$id"))
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "delete memory failed")
    }

    override suspend fun edit(id: String, request: EditMemoryRequestDto) {
        val response = client.put(config.url("/api/memory/$id")) {
            contentType(ContentType.Application.Json)
            setBody(request)
        }
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "edit memory failed")
    }

    override suspend fun confirm(id: String) {
        val response = client.post(config.url("/api/memory/$id/confirm"))
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "confirm memory failed")
    }
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

class HttpConnectionsApi(private val client: HttpClient, private val config: ApiConfig) : ConnectionsApi {
    override suspend fun overview(): ConnectionsOverviewDto {
        val response = client.get(config.url("/api/connections"))
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "connections overview failed")
        return response.body()
    }
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

class HttpUsageApi(private val client: HttpClient, private val config: ApiConfig) : UsageApi {
    override suspend fun brief(): UsageBriefDto {
        val response = client.get(config.url("/api/usage/brief"))
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "usage brief failed")
        return response.body()
    }
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

class HttpProjectsApi(private val client: HttpClient, private val config: ApiConfig) : ProjectsApi {
    override suspend fun list(orgId: String?): List<ProjectDto> {
        val url = if (orgId != null) config.url("/api/projects?org_id=$orgId") else config.url("/api/projects")
        val response = client.get(url)
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "projects list failed")
        return response.body<ListProjectsResponse>().projects
    }
}

