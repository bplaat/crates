package nl.bplaat.plaatnotes.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import nl.bplaat.plaatnotes.ui.viewmodel.NotesViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NoteDetailScreen(
    noteId: String,
    onBack: () -> Unit,
    onDeleted: () -> Unit,
    notesViewModel: NotesViewModel = viewModel()
) {
    val notesState by notesViewModel.notesState.collectAsState()
    var isEditing by remember { mutableStateOf(false) }
    var editBody by remember { mutableStateOf("") }

    LaunchedEffect(noteId) {
        notesViewModel.getNote(noteId)
    }

    LaunchedEffect(notesState.selectedNote) {
        notesState.selectedNote?.let {
            editBody = it.body
        }
    }

    if (notesState.deleteSuccess) {
        LaunchedEffect(Unit) {
            notesViewModel.clearSelectedNote()
            onDeleted()
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Note") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.Filled.ArrowBack, "Back")
                    }
                },
                actions = {
                    if (!isEditing) {
                        IconButton(onClick = {
                            notesState.selectedNote?.let {
                                notesViewModel.deleteNote(it.id)
                            }
                        }) {
                            Icon(Icons.Filled.Delete, "Delete", tint = MaterialTheme.colorScheme.error)
                        }
                    }
                }
            )
        }
    ) { innerPadding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
        ) {
            when {
                notesState.isLoading -> {
                    CircularProgressIndicator(
                        modifier = Modifier.align(Alignment.Center)
                    )
                }
                notesState.selectedNote == null -> {
                    Text(
                        "Note not found",
                        modifier = Modifier.align(Alignment.Center)
                    )
                }
                else -> {
                    val note = notesState.selectedNote!!
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        if (isEditing) {
                            TextField(
                                value = editBody,
                                onValueChange = { editBody = it },
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .weight(1f),
                                label = { Text("Note content") }
                            )

                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.spacedBy(8.dp)
                            ) {
                                Button(
                                    onClick = { isEditing = false },
                                    modifier = Modifier
                                        .weight(1f)
                                        .height(48.dp),
                                    enabled = !notesState.isLoading
                                ) {
                                    Text("Cancel")
                                }
                                Button(
                                    onClick = {
                                        notesViewModel.updateNote(note.id, editBody)
                                        isEditing = false
                                    },
                                    modifier = Modifier
                                        .weight(1f)
                                        .height(48.dp),
                                    enabled = !notesState.isLoading && editBody.isNotEmpty()
                                ) {
                                    Text("Save")
                                }
                            }
                        } else {
                            Text(
                                note.body,
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .weight(1f),
                                style = MaterialTheme.typography.bodyLarge
                            )

                            Text(
                                "Created: ${note.createdAt.take(10)}",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )

                            Button(
                                onClick = { isEditing = true },
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .height(48.dp),
                                enabled = !notesState.isLoading
                            ) {
                                Text("Edit")
                            }
                        }
                    }
                }
            }
        }
    }
}
