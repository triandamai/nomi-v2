package ai.nomi.app.screens.memory

import ai.nomi.data.MemoryRepository
import ai.nomi.network.MemoryItemDto
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

enum class MemorySort {
    Recent,
    Strongest,
    Used,
}

data class MemoryUiState(
    val loading: Boolean = true,
    val memories: List<MemoryItemDto> = emptyList(),
    val filteredMemories: List<MemoryItemDto> = emptyList(),
    val query: String = "",
    val kindFilter: String? = null,
    val sort: MemorySort = MemorySort.Recent,
    val selectedId: String? = null,
    val error: String? = null,
    val editingMemory: MemoryItemDto? = null,
    val editOpen: Boolean = false,
    val saving: Boolean = false,
)

class MemoryViewModel(
    private val repository: MemoryRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(MemoryUiState())
    val uiState: StateFlow<MemoryUiState> = _uiState.asStateFlow()

    init {
        loadMemories()
    }

    fun loadMemories() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(loading = true, error = null)
            val result = repository.getMemories()
            result.onSuccess { list ->
                _uiState.value = _uiState.value.copy(
                    loading = false,
                    memories = list,
                    error = null,
                )
                applyFilters()
            }.onFailure { error ->
                _uiState.value = _uiState.value.copy(
                    loading = false,
                    error = error.message ?: "Failed to load memory",
                )
            }
        }
    }

    fun setQuery(q: String) {
        _uiState.value = _uiState.value.copy(query = q)
        applyFilters()
    }

    fun setKindFilter(kind: String?) {
        _uiState.value = _uiState.value.copy(kindFilter = kind)
        applyFilters()
    }

    fun setSort(sort: MemorySort) {
        _uiState.value = _uiState.value.copy(sort = sort)
        applyFilters()
    }

    fun selectMemory(id: String?) {
        _uiState.value = _uiState.value.copy(selectedId = id)
    }

    private fun applyFilters() {
        val current = _uiState.value
        var items = current.memories

        if (!current.kindFilter.isNullOrBlank()) {
            items = items.filter { it.kind.equals(current.kindFilter, ignoreCase = true) }
        }

        if (current.query.isNotBlank()) {
            val q = current.query.trim().lowercase()
            items = items.filter { it.content.lowercase().contains(q) }
        }

        items = when (current.sort) {
            MemorySort.Recent -> items.sortedByDescending { it.updatedAt }
            MemorySort.Strongest -> items.sortedByDescending { it.weight }
            MemorySort.Used -> items.sortedByDescending { it.uses }
        }

        _uiState.value = _uiState.value.copy(filteredMemories = items)
    }

    fun openEdit(memory: MemoryItemDto) {
        _uiState.value = _uiState.value.copy(editingMemory = memory, editOpen = true)
    }

    fun closeEdit() {
        _uiState.value = _uiState.value.copy(editingMemory = null, editOpen = false)
    }

    fun saveEdit(content: String, kind: String?) {
        val memory = _uiState.value.editingMemory ?: return
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(saving = true)
            repository.editMemory(memory.id, content, kind).onSuccess {
                _uiState.value = _uiState.value.copy(saving = false, editOpen = false, editingMemory = null)
                loadMemories()
            }.onFailure { error ->
                _uiState.value = _uiState.value.copy(
                    saving = false,
                    error = error.message ?: "Failed to edit memory",
                )
            }
        }
    }

    fun confirmMemory(id: String) {
        viewModelScope.launch {
            repository.confirmMemory(id).onSuccess {
                loadMemories()
            }
        }
    }

    fun deleteMemory(id: String) {
        viewModelScope.launch {
            repository.deleteMemory(id).onSuccess {
                loadMemories()
            }
        }
    }
}
