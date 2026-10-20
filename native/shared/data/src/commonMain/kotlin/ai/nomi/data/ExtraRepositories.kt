package ai.nomi.data

import ai.nomi.network.ConnectionsApi
import ai.nomi.network.ConnectionsOverviewDto
import ai.nomi.network.CreateBudgetRequestDto
import ai.nomi.network.CreateReminderRequestDto
import ai.nomi.network.CreateTransactionRequestDto
import ai.nomi.network.EditMemoryRequestDto
import ai.nomi.network.MemoryApi
import ai.nomi.network.MemoryItemDto
import ai.nomi.network.MoneyApi
import ai.nomi.network.MoneySummaryDto
import ai.nomi.network.ReminderActionRequestDto
import ai.nomi.network.ReminderDto
import ai.nomi.network.RemindersApi
import ai.nomi.network.RemindersResponseDto
import ai.nomi.network.TransactionDto
import ai.nomi.network.BudgetDto

class MoneyRepository(
    private val api: MoneyApi,
    private val session: SessionScope,
) {
    suspend fun getSummary(month: String? = null): Result<MoneySummaryDto> = runCatching {
        session.run { api.summary(month) }
    }

    suspend fun createTransaction(request: CreateTransactionRequestDto): Result<TransactionDto> = runCatching {
        session.run { api.createTransaction(request) }
    }

    suspend fun setBudget(request: CreateBudgetRequestDto): Result<BudgetDto> = runCatching {
        session.run { api.setBudget(request) }
    }
}

class RemindersRepository(
    private val api: RemindersApi,
    private val session: SessionScope,
) {
    suspend fun getReminders(): Result<RemindersResponseDto> = runCatching {
        session.run { api.list() }
    }

    suspend fun createReminder(request: CreateReminderRequestDto): Result<ReminderDto> = runCatching {
        session.run { api.create(request) }
    }

    suspend fun action(id: String, action: String, minutes: Long? = null): Result<Unit> = runCatching {
        session.run { api.action(id, ReminderActionRequestDto(action = action, minutes = minutes)) }
    }

    suspend fun cancelScheduledTask(id: String): Result<Unit> = runCatching {
        session.run { api.cancelScheduledTask(id) }
    }
}

class MemoryRepository(
    private val api: MemoryApi,
    private val session: SessionScope,
) {
    suspend fun getMemories(): Result<List<MemoryItemDto>> = runCatching {
        session.run { api.list().memories }
    }

    suspend fun deleteMemory(id: String): Result<Unit> = runCatching {
        session.run { api.delete(id) }
    }

    suspend fun editMemory(id: String, content: String, kind: String?): Result<Unit> = runCatching {
        session.run { api.edit(id, EditMemoryRequestDto(content = content, kind = kind)) }
    }

    suspend fun confirmMemory(id: String): Result<Unit> = runCatching {
        session.run { api.confirm(id) }
    }
}

class ConnectionsRepository(
    private val api: ConnectionsApi,
    private val session: SessionScope,
) {
    suspend fun getOverview(): Result<ConnectionsOverviewDto> = runCatching {
        session.run { api.overview() }
    }
}

class UsageRepository(
    private val api: ai.nomi.network.UsageApi,
    private val session: SessionScope,
) {
    private val mutableBrief = kotlinx.coroutines.flow.MutableStateFlow<ai.nomi.network.UsageBriefDto?>(null)
    val brief: kotlinx.coroutines.flow.StateFlow<ai.nomi.network.UsageBriefDto?> = mutableBrief

    suspend fun refresh(): Result<ai.nomi.network.UsageBriefDto> = runCatching {
        session.run {
            val res = api.brief()
            mutableBrief.value = res
            res
        }
    }

    suspend fun getBrief(): Result<ai.nomi.network.UsageBriefDto> = runCatching {
        session.run {
            val res = api.brief()
            mutableBrief.value = res
            res
        }
    }
}

class ProjectsRepository(
    private val api: ai.nomi.network.ProjectsApi,
    private val session: SessionScope,
) {
    suspend fun list(orgId: String? = null): Result<List<ai.nomi.network.ProjectDto>> = runCatching {
        session.run { api.list(orgId) }
    }
}

