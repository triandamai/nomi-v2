package ai.nomi.app.screens.projects

import ai.nomi.data.AddressChange
import ai.nomi.data.AddressCheck
import ai.nomi.data.DataQueryResult
import ai.nomi.data.ProjectAddress
import ai.nomi.data.ProjectData
import ai.nomi.data.ProjectHealth
import ai.nomi.data.ProjectToolsRepository
import ai.nomi.data.ShowcaseInfo
import ai.nomi.data.TransferTarget
import ai.nomi.network.ApiException
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/** Traffic periods the monitor offers, as the web's. */
val TRAFFIC_RANGES = listOf("1h", "24h", "7d")
private const val DEFAULT_RANGE = "24h"
/** How long typing settles before a name is checked or organizations are searched. */
private const val TYPING_SETTLE_MS = 300L
private const val LOAD_FAILED = "Couldn't load this. Check your connection and try again."
private const val SAVE_FAILED = "Couldn't save that. Try again."

/** The Domain sheet: the address, the name being typed, and what the backend says about it. */
data class AddressState(
    val address: Loadable<ProjectAddress> = Loadable.Loading,
    val draft: String = "",
    val check: AddressCheck? = null,
    val checking: Boolean = false,
    val saving: Boolean = false,
    val error: String? = null,
)

/** The Monitor sheet over [range]. */
data class HealthState(val range: String = DEFAULT_RANGE, val health: Loadable<ProjectHealth> = Loadable.Loading)

/** The Database sheet: which copy, its tables, the query typed and its answer. */
data class DataState(
    val production: Boolean = false,
    val data: Loadable<ProjectData> = Loadable.Loading,
    val sql: String = "",
    val running: Boolean = false,
    val result: DataQueryResult? = null,
    val queryError: String? = null,
)

/** The Settings sheet: showcase, and moving or deleting the project. */
data class SettingsState(
    val showcase: Loadable<ShowcaseInfo> = Loadable.Loading,
    val blurb: String = "",
    val search: String = "",
    val targets: List<TransferTarget> = emptyList(),
    val busy: Boolean = false,
    val error: String? = null,
)

/**
 * A project's tools beyond the workspace ([projectId], named [projectName]): its address, how production is doing,
 * its data ([relay] for a Relay backend) and its settings. [left] turns true once the project moved away or was
 * deleted, so the workspace can close.
 */
class ProjectToolsViewModel(
    private val projectId: String,
    private val projectName: String,
    private val relay: Boolean,
    private val repo: ProjectToolsRepository,
    private val log: (String) -> Unit = {},
) : ViewModel() {
    private val mutableAddress = MutableStateFlow(AddressState())
    val address: StateFlow<AddressState> = mutableAddress.asStateFlow()

    private val mutableHealth = MutableStateFlow(HealthState())
    val health: StateFlow<HealthState> = mutableHealth.asStateFlow()

    private val mutableData = MutableStateFlow(DataState())
    val data: StateFlow<DataState> = mutableData.asStateFlow()

    private val mutableSettings = MutableStateFlow(SettingsState())
    val settings: StateFlow<SettingsState> = mutableSettings.asStateFlow()

    private val mutableLeft = MutableStateFlow(false)
    val left: StateFlow<Boolean> = mutableLeft.asStateFlow()

    private var checkJob: Job? = null
    private var searchJob: Job? = null

    fun loadAddress() {
        viewModelScope.launch {
            val address = attempt("address") { repo.address(projectId) }
            mutableAddress.update { state ->
                state.copy(
                    address = address?.let { Loadable.Ready(it) } ?: Loadable.Failed(LOAD_FAILED),
                    draft = if (state.draft.isEmpty()) address?.subdomain.orEmpty() else state.draft,
                )
            }
        }
    }

    /** The name typed: checked once typing settles. */
    fun editAddress(subdomain: String) {
        mutableAddress.update { it.copy(draft = subdomain, check = null, error = null) }
        checkJob?.cancel()
        val current = (mutableAddress.value.address as? Loadable.Ready)?.value?.subdomain
        if (subdomain.isBlank() || subdomain == current) return
        checkJob = viewModelScope.launch {
            delay(TYPING_SETTLE_MS)
            mutableAddress.update { it.copy(checking = true) }
            val check = attempt("check address") { repo.checkAddress(projectId, subdomain.trim()) }
            mutableAddress.update { it.copy(checking = false, check = check) }
        }
    }

    fun saveAddress() {
        val name = mutableAddress.value.draft.trim()
        if (name.isEmpty()) return
        viewModelScope.launch {
            mutableAddress.update { it.copy(saving = true, error = null) }
            val change = attempt("change address") { repo.changeAddress(projectId, name) }
            when (change) {
                is AddressChange.Changed -> {
                    mutableAddress.update { it.copy(saving = false, check = null) }
                    loadAddress()
                }
                is AddressChange.Refused -> mutableAddress.update { it.copy(saving = false, check = change.check) }
                null -> mutableAddress.update { it.copy(saving = false, error = SAVE_FAILED) }
            }
        }
    }

    fun loadHealth(range: String = mutableHealth.value.range) {
        mutableHealth.value = HealthState(range = range)
        viewModelScope.launch {
            val health = attempt("health") { repo.health(projectId, range) }
            mutableHealth.update { state ->
                if (state.range != range) state else state.copy(health = health?.let { Loadable.Ready(it) } ?: Loadable.Failed(LOAD_FAILED))
            }
        }
    }

    fun loadData(production: Boolean = mutableData.value.production) {
        mutableData.update { it.copy(production = production, data = Loadable.Loading, result = null, queryError = null) }
        viewModelScope.launch {
            val data = attempt("data") { repo.data(projectId, relay, production) }
            mutableData.update { state ->
                if (state.production != production) state else state.copy(data = data?.let { Loadable.Ready(it) } ?: Loadable.Failed(LOAD_FAILED))
            }
        }
    }

    fun retryRelay() {
        viewModelScope.launch {
            attempt("retry relay") { repo.retryRelay(projectId, mutableData.value.production) }
            loadData()
        }
    }

    fun editQuery(sql: String) = mutableData.update { it.copy(sql = sql) }

    /** Runs the typed query; the backend's refusal (bad SQL, read-only production) shows as the error. */
    fun runQuery() {
        val state = mutableData.value
        if (state.sql.isBlank() || state.running) return
        viewModelScope.launch {
            mutableData.update { it.copy(running = true, queryError = null) }
            try {
                val result = repo.query(projectId, state.sql.trim(), relay, state.production)
                mutableData.update { it.copy(running = false, result = result) }
            } catch (e: CancellationException) {
                throw e
            } catch (refused: ApiException) {
                mutableData.update { it.copy(running = false, result = null, queryError = refused.message ?: SAVE_FAILED) }
            } catch (e: Exception) {
                log("project tools: query failed: $e")
                mutableData.update { it.copy(running = false, result = null, queryError = LOAD_FAILED) }
            }
        }
    }

    fun loadSettings() {
        viewModelScope.launch {
            val showcase = attempt("showcase") { repo.showcase(projectId) }
            mutableSettings.update {
                it.copy(showcase = showcase?.let { s -> Loadable.Ready(s) } ?: Loadable.Failed(LOAD_FAILED), blurb = showcase?.blurb.orEmpty())
            }
        }
        searchTargets("")
    }

    fun editBlurb(blurb: String) = mutableSettings.update { it.copy(blurb = blurb) }

    fun setShowcase(on: Boolean) {
        viewModelScope.launch {
            mutableSettings.update { it.copy(busy = true, error = null) }
            val blurb = mutableSettings.value.blurb.trim().ifEmpty { null }
            val saved = attempt("showcase") { repo.setShowcase(projectId, on, blurb) }
            mutableSettings.update {
                if (saved == null) it.copy(busy = false, error = SAVE_FAILED) else it.copy(busy = false, showcase = Loadable.Ready(saved))
            }
        }
    }

    fun takeOffline() = act("take offline") { repo.takeOffline(projectId) }

    /** Organizations matching [query], once typing settles. */
    fun searchTargets(query: String) {
        mutableSettings.update { it.copy(search = query) }
        searchJob?.cancel()
        searchJob = viewModelScope.launch {
            if (query.isNotEmpty()) delay(TYPING_SETTLE_MS)
            val targets = attempt("transfer targets") { repo.transferTargets(projectId, query) }.orEmpty()
            mutableSettings.update { it.copy(targets = targets) }
        }
    }

    /** Moves the project to [orgId]; [typedName] must be the project's name, as the web asks. */
    fun transfer(orgId: String, typedName: String) {
        if (typedName.trim() != projectName.trim()) return
        act("transfer", leaves = true) { repo.transfer(projectId, orgId, typedName.trim()) }
    }

    /** Deletes the project; [typedName] must be its name. */
    fun delete(typedName: String) {
        if (typedName.trim() != projectName.trim()) return
        act("delete", leaves = true) { repo.delete(projectId) }
    }

    private fun act(what: String, leaves: Boolean = false, block: suspend () -> Unit) {
        viewModelScope.launch {
            mutableSettings.update { it.copy(busy = true, error = null) }
            val done = attempt(what) { block() } != null
            mutableSettings.update { it.copy(busy = false, error = if (done) null else SAVE_FAILED) }
            if (done && leaves) mutableLeft.value = true
        }
    }

    private suspend fun <T> attempt(what: String, block: suspend () -> T): T? = try {
        block()
    } catch (e: CancellationException) {
        throw e
    } catch (e: Exception) {
        log("project tools: $what failed: $e")
        null
    }
}
