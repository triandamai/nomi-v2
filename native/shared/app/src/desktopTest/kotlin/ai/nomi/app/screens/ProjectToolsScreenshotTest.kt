package ai.nomi.app.screens

import ai.nomi.app.screens.projects.AddressState
import ai.nomi.app.screens.projects.DataState
import ai.nomi.app.screens.projects.DatabaseSheet
import ai.nomi.app.screens.projects.DomainSheet
import ai.nomi.app.screens.projects.HealthState
import ai.nomi.app.screens.projects.Loadable
import ai.nomi.app.screens.projects.MonitorSheet
import ai.nomi.app.screens.projects.SettingsSheet
import ai.nomi.app.screens.projects.SettingsState
import ai.nomi.data.AddressCheck
import ai.nomi.data.ContainerStat
import ai.nomi.data.DataQueryResult
import ai.nomi.data.DataTable
import ai.nomi.data.ProjectAddress
import ai.nomi.data.ProjectData
import ai.nomi.data.ProjectHealth
import ai.nomi.data.ProjectTrafficInfo
import ai.nomi.data.ShowcaseInfo
import ai.nomi.data.TrafficTotals
import ai.nomi.data.TransferTarget
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.unit.dp
import org.junit.Rule
import kotlin.test.Test

private val SheetPadding = 16.dp
private const val MB = 1024L * 1024L

private val Address = AddressState(
    address = Loadable.Ready(ProjectAddress("bakery.nomi.app", "bakery", "https://bakery.nomi.app", canEdit = true)),
    draft = "bread",
    check = AddressCheck(available = false, hostname = null, message = "bread is taken", suggestions = listOf("bread-shop", "bread-2")),
)

private val Totals = TrafficTotals(visitors = 412.0, requests = 5_230.0, errors5xx = 3.0, errors4xx = 41.0, p50Ms = 38.0, p95Ms = 120.0)
private val Before = TrafficTotals(visitors = 380.0, requests = 4_800.0, errors5xx = 6.0, errors4xx = 30.0, p50Ms = 40.0, p95Ms = 140.0)

private val Health = HealthState(
    health = Loadable.Ready(
        ProjectHealth(
            containers = listOf(ContainerStat("web", 3.4, 96 * MB, 512 * MB), ContainerStat("worker", null, null, null)),
            traffic = ProjectTrafficInfo("24h", listOf(120.0, 90.0, 60.0, 80.0, 210.0, 340.0, 300.0, 260.0, 280.0, 190.0), Totals, Before),
            logLines = listOf("GET / 200 12ms", "GET /menu 200 18ms", "POST /api/orders 201 44ms"),
            logNote = null,
        ),
    ),
)

private val Data = DataState(
    data = Loadable.Ready(ProjectData(false, listOf(DataTable("orders", 42, "public"), DataTable("products", 18, "public")), null, null)),
    sql = "select id, item, total from orders limit 3",
    result = DataQueryResult(
        columns = listOf("id", "item", "total"),
        rows = listOf(listOf("1", "Sourdough", "6.50"), listOf("2", "Croissant", "3.20"), listOf("3", "Rye", null)),
        truncated = false,
        rowsAffected = null,
        tookMs = 4.0,
    ),
)

private val Settings = SettingsState(
    showcase = Loadable.Ready(ShowcaseInfo(showcase = true, blurb = "Fresh bread daily", canEdit = true)),
    blurb = "Fresh bread daily",
    targets = listOf(TransferTarget("o2", "Acme", personal = false, ownerName = "Sam"), TransferTarget("o3", "Trian", personal = true, ownerName = null)),
)

class ProjectToolsScreenshotTest {
    @get:Rule val rule = createComposeRule()

    @Composable
    private fun Sheet(content: @Composable ColumnScope.() -> Unit) {
        Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surfaceContainerLow).padding(SheetPadding), content = content)
    }

    @Test fun domainPhoneLight() = rule.captureScreen("project_domain", Device.Phone, false) {
        Sheet { DomainSheet(Address, onEdit = {}, onSave = {}, onRetry = {}) }
    }

    @Test fun monitorPhoneLight() = rule.captureScreen("project_monitor", Device.Phone, false) {
        Sheet { MonitorSheet(Health, onRange = {}, onRetry = {}) }
    }

    @Test fun monitorPhoneDark() = rule.captureScreen("project_monitor", Device.Phone, true) {
        Sheet { MonitorSheet(Health, onRange = {}, onRetry = {}) }
    }

    @Test fun databasePhoneLight() = rule.captureScreen("project_database", Device.Phone, false) {
        Sheet { DatabaseSheet(Data, relay = false, onSource = {}, onSql = {}, onRun = {}, onRetry = {}, onRetrySetup = {}) }
    }

    @Test fun settingsPhoneLight() = rule.captureScreen("project_settings", Device.Phone, false) {
        Sheet {
            SettingsSheet(
                Settings, projectName = "Bakery site", canPublish = true, onBlurb = {}, onShowcase = {}, onTakeOffline = {},
                onSearch = {}, onTransfer = { _, _ -> }, onDelete = {},
            )
        }
    }
}
