package nl.bplaat.plaatnotes.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import nl.bplaat.plaatnotes.ui.viewmodel.NotesViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CreateNoteScreen(
    onBack: () -> Unit,
    onNoteCreated: () -> Unit,
    notesViewModel: NotesViewModel = viewModel()
) {
    val notesState by notesViewModel.notesState.collectAsState()
    var noteBody by remember { mutableStateOf("") }

    LaunchedEffect(notesState.notes.size) {
        if (notesState.notes.isNotEmpty() && !notesState.isLoading) {
            onNoteCreated()
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("New Note") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.Filled.ArrowBack, "Back")
                    }
                }
            )
        }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            TextField(
                value = noteBody,
                onValueChange = { noteBody = it },
                modifier = Modifier
                    .fillMaxWidth()
                    .weight(1f),
                label = { Text("Note content") },
                enabled = !notesState.isLoading
            )

            if (notesState.error != null) {
                Text(
                    text = "Error: ${notesState.error}",
                    color = MaterialTheme.colorScheme.error,
                    style = MaterialTheme.typography.bodySmall
                )
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Button(
                    onClick = onBack,
                    modifier = Modifier
                        .weight(1f)
                        .height(48.dp),
                    enabled = !notesState.isLoading
                ) {
                    Text("Cancel")
                }
                Button(
                    onClick = { notesViewModel.createNote(noteBody) },
                    modifier = Modifier
                        .weight(1f)
                        .height(48.dp),
                    enabled = !notesState.isLoading && noteBody.isNotEmpty()
                ) {
                    if (notesState.isLoading) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(24.dp),
                            color = MaterialTheme.colorScheme.onPrimary
                        )
                    } else {
                        Text("Create")
                    }
                }
            }
        }
    }
}
