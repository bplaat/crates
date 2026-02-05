package nl.bplaat.plaatnotes.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import nl.bplaat.plaatnotes.data.models.Note
import nl.bplaat.plaatnotes.data.repository.ApiResult
import nl.bplaat.plaatnotes.data.repository.NoteRepository

data class NotesUiState(
    val notes: List<Note> = emptyList(),
    val isLoading: Boolean = false,
    val error: String? = null,
    val selectedNote: Note? = null,
    val deleteSuccess: Boolean = false
)

class NotesViewModel : ViewModel() {
    private val noteRepository = NoteRepository()
    private val _notesState = MutableStateFlow(NotesUiState())
    val notesState: StateFlow<NotesUiState> = _notesState.asStateFlow()

    fun loadNotes(query: String? = null) {
        viewModelScope.launch {
            _notesState.value = _notesState.value.copy(isLoading = true, error = null)
            val result = noteRepository.getNotes(query = query)
            when (result) {
                is ApiResult.Success -> {
                    _notesState.value = _notesState.value.copy(
                        isLoading = false,
                        notes = result.data.data
                    )
                }
                is ApiResult.Error -> {
                    _notesState.value = _notesState.value.copy(
                        isLoading = false,
                        error = result.message
                    )
                }
                is ApiResult.Loading -> {}
            }
        }
    }

    fun createNote(body: String) {
        viewModelScope.launch {
            _notesState.value = _notesState.value.copy(isLoading = true, error = null)
            val result = noteRepository.createNote(body)
            when (result) {
                is ApiResult.Success -> {
                    _notesState.value = _notesState.value.copy(
                        isLoading = false,
                        notes = listOf(result.data) + _notesState.value.notes
                    )
                }
                is ApiResult.Error -> {
                    _notesState.value = _notesState.value.copy(
                        isLoading = false,
                        error = result.message
                    )
                }
                is ApiResult.Loading -> {}
            }
        }
    }

    fun getNote(noteId: String) {
        viewModelScope.launch {
            _notesState.value = _notesState.value.copy(isLoading = true, error = null)
            val result = noteRepository.getNote(noteId)
            when (result) {
                is ApiResult.Success -> {
                    _notesState.value = _notesState.value.copy(
                        isLoading = false,
                        selectedNote = result.data
                    )
                }
                is ApiResult.Error -> {
                    _notesState.value = _notesState.value.copy(
                        isLoading = false,
                        error = result.message
                    )
                }
                is ApiResult.Loading -> {}
            }
        }
    }

    fun updateNote(noteId: String, body: String) {
        viewModelScope.launch {
            _notesState.value = _notesState.value.copy(isLoading = true, error = null)
            val result = noteRepository.updateNote(noteId, body)
            when (result) {
                is ApiResult.Success -> {
                    val updatedNotes = _notesState.value.notes.map {
                        if (it.id == noteId) result.data else it
                    }
                    _notesState.value = _notesState.value.copy(
                        isLoading = false,
                        notes = updatedNotes,
                        selectedNote = result.data
                    )
                }
                is ApiResult.Error -> {
                    _notesState.value = _notesState.value.copy(
                        isLoading = false,
                        error = result.message
                    )
                }
                is ApiResult.Loading -> {}
            }
        }
    }

    fun deleteNote(noteId: String) {
        viewModelScope.launch {
            _notesState.value = _notesState.value.copy(isLoading = true, error = null)
            val result = noteRepository.deleteNote(noteId)
            when (result) {
                is ApiResult.Success -> {
                    val updatedNotes = _notesState.value.notes.filter { it.id != noteId }
                    _notesState.value = _notesState.value.copy(
                        isLoading = false,
                        notes = updatedNotes,
                        selectedNote = null,
                        deleteSuccess = true
                    )
                }
                is ApiResult.Error -> {
                    _notesState.value = _notesState.value.copy(
                        isLoading = false,
                        error = result.message
                    )
                }
                is ApiResult.Loading -> {}
            }
        }
    }

    fun clearSelectedNote() {
        _notesState.value = _notesState.value.copy(selectedNote = null, deleteSuccess = false)
    }
}
