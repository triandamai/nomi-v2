package ai.nomi.app.screens.connections

import ai.nomi.data.ConnectionsRepository
import ai.nomi.network.ConnectionsOverviewDto
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class ConnectorItem(
    val id: String,
    val name: String,
    val category: String,
    val agent: String,
    val description: String,
    val connected: Boolean,
    val statusText: String,
    val soon: Boolean = false,
)

data class ConnectionsUiState(
    val loading: Boolean = true,
    val overview: ConnectionsOverviewDto? = null,
    val connectors: List<ConnectorItem> = emptyList(),
    val filteredConnectors: List<ConnectorItem> = emptyList(),
    val query: String = "",
    val error: String? = null,
)

class ConnectionsViewModel(
    private val repository: ConnectionsRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(ConnectionsUiState())
    val uiState: StateFlow<ConnectionsUiState> = _uiState.asStateFlow()

    init {
        loadOverview()
    }

    fun loadOverview() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(loading = true, error = null)
            val result = repository.getOverview()
            result.onSuccess { overview ->
                val list = buildConnectors(overview)
                _uiState.value = _uiState.value.copy(
                    loading = false,
                    overview = overview,
                    connectors = list,
                    filteredConnectors = filterList(list, _uiState.value.query),
                    error = null,
                )
            }.onFailure { error ->
                _uiState.value = _uiState.value.copy(
                    loading = false,
                    error = error.message ?: "Failed to load connections",
                )
            }
        }
    }

    fun setQuery(q: String) {
        _uiState.value = _uiState.value.copy(
            query = q,
            filteredConnectors = filterList(_uiState.value.connectors, q),
        )
    }

    private fun filterList(list: List<ConnectorItem>, query: String): List<ConnectorItem> {
        if (query.isBlank()) return list
        val q = query.trim().lowercase()
        return list.filter {
            it.name.lowercase().contains(q) ||
                it.description.lowercase().contains(q) ||
                it.agent.lowercase().contains(q)
        }
    }

    private fun buildConnectors(overview: ConnectionsOverviewDto): List<ConnectorItem> {
        val googleConnected = !overview.google.email.isNullOrBlank()
        val googleStatus = if (googleConnected) overview.google.email ?: "Connected" else "Not connected"

        val hevyConnected = overview.hevy?.connected == true
        val hevyStatus = if (hevyConnected) overview.hevy?.accountName ?: "Connected" else "Not connected"

        return listOf(
            ConnectorItem(
                id = "google",
                name = "Google Workspace",
                category = "Productivity",
                agent = "workspace",
                description = "Calendar, Gmail and Drive access for planning and scheduling",
                connected = googleConnected,
                statusText = googleStatus,
                soon = false,
            ),
            ConnectorItem(
                id = "hevy",
                name = "Hevy",
                category = "Health",
                agent = "health",
                description = "Workout logs, exercises and personal records tracking",
                connected = hevyConnected,
                statusText = hevyStatus,
                soon = false,
            ),
            ConnectorItem(
                id = "samsung_health",
                name = "Samsung Health",
                category = "Health",
                agent = "health",
                description = "Daily steps, sleep duration, heart rate and activity stats",
                connected = false,
                statusText = "Coming soon",
                soon = true,
            ),
            ConnectorItem(
                id = "apple_health",
                name = "Apple Health",
                category = "Health",
                agent = "health",
                description = "Activity rings, workouts, sleep stages and vitals",
                connected = false,
                statusText = "Coming soon",
                soon = true,
            ),
        )
    }
}
