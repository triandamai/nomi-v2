package ai.nomi.data

import ai.nomi.network.AttachmentsApi
import ai.nomi.network.BudgetDto
import ai.nomi.network.CategoryTotalDto
import ai.nomi.network.ConnectionsApi
import ai.nomi.network.ConnectionsOverviewDto
import ai.nomi.network.CreateBudgetRequestDto
import ai.nomi.network.CreateReminderRequestDto
import ai.nomi.network.CreateTransactionRequestDto
import ai.nomi.network.CrewMemberDto
import ai.nomi.network.DayTotalDto
import ai.nomi.network.EditMemoryRequestDto
import ai.nomi.network.GoogleOverviewDto
import ai.nomi.network.HealthConnectionDto
import ai.nomi.network.HomeApi
import ai.nomi.network.HomeSummaryDto
import ai.nomi.network.MemoryApi
import ai.nomi.network.MemoryItemDto
import ai.nomi.network.MemoryListResponseDto
import ai.nomi.network.MoneyApi
import ai.nomi.network.MoneySummaryDto
import ai.nomi.network.OutItemDto
import ai.nomi.network.PlanItemDto
import ai.nomi.network.PreferencesDto
import ai.nomi.network.ProfileDto
import ai.nomi.network.ProjectDto
import ai.nomi.network.ProjectsApi
import ai.nomi.network.ReminderActionRequestDto
import ai.nomi.network.ReminderDto
import ai.nomi.network.RemindersApi
import ai.nomi.network.RemindersResponseDto
import ai.nomi.network.ScheduledTaskDto
import ai.nomi.network.TodayItemDto
import ai.nomi.network.TransactionDto
import ai.nomi.network.TransactionItemDto
import ai.nomi.network.UpdatePreferencesDto
import ai.nomi.network.UsageApi
import ai.nomi.network.UsageBriefDto
import ai.nomi.network.UsagePlanDto
import ai.nomi.sdk.NewReminder
import ai.nomi.sdk.NewTransaction
import ai.nomi.sdk.NewTransactionItem
import ai.nomi.sdk.PreferencesUpdate
import ai.nomi.sdk.ReminderAction

// The account pages' APIs on the Rust engine, behind the interfaces the repositories already use.

private const val ACTION_DONE = "done"
private const val ACTION_CANCEL = "cancel"
private const val ACTION_SNOOZE = "snooze"

/** The backend's snooze when none is given. */
private const val DEFAULT_SNOOZE_MINUTES = 10L

class SdkHomeApi(private val engine: NomiEngine) : HomeApi {
    override suspend fun profile(): ProfileDto = translated {
        engine.get().fetchProfile().let { ProfileDto(it.displayName, it.username, it.email, it.avatarUrl) }
    }

    override suspend fun crew(): List<CrewMemberDto> = translated {
        engine.get().crew().map {
            CrewMemberDto(it.agentType, it.name, it.role, it.isDynamic, it.shape, it.tone, it.motion, it.state, it.status)
        }
    }

    override suspend fun preferences(): PreferencesDto = translated { engine.get().preferences().toDto() }

    override suspend fun updatePreferences(request: UpdatePreferencesDto): PreferencesDto = translated {
        val update = PreferencesUpdate(
            theme = request.theme,
            accentColor = request.accentColor,
            timezone = request.timezone,
            homeLayout = request.homeLayout,
            drawerPins = request.drawerPins,
            drawerRecents = request.drawerRecents,
            drawerRecentsLimit = request.drawerRecentsLimit?.toUInt(),
        )
        engine.get().updatePreferences(update).toDto()
    }

    override suspend fun summary(): HomeSummaryDto = translated {
        val home = engine.get().homeSummary()
        HomeSummaryDto(
            since = home.since,
            timezone = home.timezone,
            whileYouWereOut = home.whileYouWereOut.map { OutItemDto(it.kind, it.agent, it.title, it.detail, it.sessionId, it.at) },
            today = home.today.map { TodayItemDto(it.id, it.runAt, it.label, it.agent, it.recurrence) },
            plans = home.plans.map { PlanItemDto(it.kind, it.title, it.agent, it.done, it.total, it.sessionId, it.updatedAt) },
            whileYouWereOutTotal = home.whileYouWereOutTotal.toInt(),
            todayTotal = home.todayTotal.toInt(),
            plansTotal = home.plansTotal.toInt(),
        )
    }

    private fun ai.nomi.sdk.Preferences.toDto() = PreferencesDto(
        theme = theme,
        accentColor = accentColor,
        timezone = timezone,
        homeLayout = homeLayout,
        drawerPins = drawerPins,
        drawerRecents = drawerRecents,
        drawerRecentsLimit = drawerRecentsLimit.toInt(),
    )
}

class SdkMoneyApi(private val engine: NomiEngine) : MoneyApi {
    override suspend fun summary(month: String?): MoneySummaryDto = translated {
        val money = engine.get().moneySummary(month)
        MoneySummaryDto(
            month = money.month,
            startDay = money.startDay.toInt(),
            periodStart = money.periodStart,
            periodEnd = money.periodEnd,
            timezone = money.timezone,
            totalCents = money.totalCents,
            incomeCents = money.incomeCents,
            previousTotalCents = money.previousTotalCents,
            transactionCount = money.transactionCount,
            byCategory = money.byCategory.map { CategoryTotalDto(it.category, it.cents, it.count) },
            byDay = money.byDay.map { DayTotalDto(it.date, it.cents) },
            transactions = money.transactions.map { transaction ->
                TransactionDto(
                    id = transaction.id,
                    occurredAt = transaction.occurredAt,
                    amountCents = transaction.amountCents,
                    category = transaction.category,
                    description = transaction.description,
                    kind = transaction.kind,
                    source = transaction.source,
                    createdAt = transaction.createdAt,
                    items = transaction.items.map {
                        TransactionItemDto(it.name, it.quantity, it.unitAmountCents, it.amountCents, it.category)
                    },
                )
            },
            months = money.months,
            budgets = money.budgets.map { BudgetDto(it.category, it.limitCents, it.spentCents) },
        )
    }

    override suspend fun createTransaction(request: CreateTransactionRequestDto) = translated {
        engine.get().addTransaction(
            NewTransaction(
                kind = request.kind,
                amountCents = request.amountCents,
                category = request.category,
                description = request.description,
                occurredAt = request.occurredAt.ifBlank { null },
                items = request.items.map {
                    NewTransactionItem(it.name, it.quantity, it.unitAmountCents, it.amountCents, it.category)
                },
            ),
        )
    }

    override suspend fun setBudget(request: CreateBudgetRequestDto) = translated {
        engine.get().setBudget(request.category, request.limitCents)
    }
}

class SdkRemindersApi(private val engine: NomiEngine) : RemindersApi {
    override suspend fun list(): RemindersResponseDto = translated {
        val overview = engine.get().reminders()
        RemindersResponseDto(
            timezone = overview.timezone,
            upcoming = overview.upcoming.map { it.toDto() },
            past = overview.past.map { it.toDto() },
            scheduledTasks = overview.scheduledTasks.map {
                ScheduledTaskDto(it.id, it.label, it.runAt, it.recurrence, it.agent, it.sessionId)
            },
        )
    }

    override suspend fun create(request: CreateReminderRequestDto): ReminderDto = translated {
        engine.get().createReminder(NewReminder(request.title, request.dueAt, request.recurrence, request.notes)).toDto()
    }

    override suspend fun action(id: String, request: ReminderActionRequestDto) = translated {
        val action = when (request.action) {
            ACTION_DONE -> ReminderAction.Done
            ACTION_CANCEL -> ReminderAction.Cancel
            ACTION_SNOOZE -> ReminderAction.Snooze((request.minutes ?: DEFAULT_SNOOZE_MINUTES).toUInt())
            else -> throw IllegalArgumentException("unknown reminder action ${request.action}")
        }
        engine.get().actOnReminder(id, action)
    }

    override suspend fun cancelScheduledTask(id: String) = translated { engine.get().cancelScheduledTask(id) }

    private fun ai.nomi.sdk.Reminder.toDto() =
        ReminderDto(id, title, notes, dueAt, recurrence, status, createdBy, createdAt)
}

class SdkMemoryApi(private val engine: NomiEngine) : MemoryApi {
    override suspend fun list(): MemoryListResponseDto = translated {
        MemoryListResponseDto(
            engine.get().memories().map {
                MemoryItemDto(
                    it.id, it.content, it.weight, it.kind, it.createdAt, it.updatedAt, it.uses, it.lastUsedAt,
                    it.needsCheck, it.embedding,
                )
            },
        )
    }

    override suspend fun delete(id: String) = translated { engine.get().deleteMemory(id) }

    override suspend fun edit(id: String, request: EditMemoryRequestDto) =
        translated { engine.get().editMemory(id, request.content, request.kind) }

    override suspend fun confirm(id: String) = translated { engine.get().confirmMemory(id) }
}

class SdkConnectionsApi(private val engine: NomiEngine) : ConnectionsApi {
    override suspend fun overview(): ConnectionsOverviewDto = translated {
        val connections = engine.get().connections()
        ConnectionsOverviewDto(
            google = connections.google.let { GoogleOverviewDto(it.configured, it.email, it.services) },
            hevy = connections.hevy?.let { HealthConnectionDto(it.connected, it.accountName, it.connectedAt) },
        )
    }
}

class SdkUsageApi(private val engine: NomiEngine) : UsageApi {
    override suspend fun brief(): UsageBriefDto = translated {
        val usage = engine.get().usageBrief()
        UsageBriefDto(UsagePlanDto(usage.plan.name, usage.plan.monthlyTokens), usage.month, usage.tokensUsed, usage.canUpgrade)
    }
}

class SdkProjectsApi(private val engine: NomiEngine) : ProjectsApi {
    override suspend fun list(orgId: String?): List<ProjectDto> = translated {
        engine.get().projects(orgId).map {
            ProjectDto(it.id, it.sessionId, it.name, it.description, it.status, it.createdAt, it.updatedAt, it.createdBy, it.openIssues)
        }
    }
}

class SdkAttachmentsApi(private val engine: NomiEngine) : AttachmentsApi {
    override suspend fun preview(id: String): ByteArray = translated { engine.get().attachmentPreview(id) }

    override suspend fun content(id: String): ByteArray = translated { engine.get().attachmentContent(id) }
}
