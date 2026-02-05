package nl.bplaat.plaatnotes.ui

import androidx.compose.runtime.*
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import nl.bplaat.plaatnotes.ui.screens.*

sealed class Screen(val route: String) {
    data object Login : Screen("login")
    data object Home : Screen("home")
    data object CreateNote : Screen("create_note")
    data class NoteDetail(val noteId: String = "{noteId}") : Screen("note_detail/{noteId}")
    data object Settings : Screen("settings")
}

@Composable
fun PlaatNotesApp(
    userId: String? = null,
    token: String? = null
) {
    val navController = rememberNavController()
    var currentUserId by remember { mutableStateOf(userId) }
    var currentToken by remember { mutableStateOf(token) }

    NavHost(
        navController = navController,
        startDestination = if (currentUserId != null && currentToken != null) {
            Screen.Home.route
        } else {
            Screen.Login.route
        }
    ) {
        composable(Screen.Login.route) {
            LoginScreen(
                onLoginSuccess = { userId, token ->
                    currentUserId = userId
                    currentToken = token
                    navController.navigate(Screen.Home.route) {
                        popUpTo(Screen.Login.route) { inclusive = true }
                    }
                }
            )
        }

        composable(Screen.Home.route) {
            NotesHomeScreen(
                onNoteSelected = { noteId ->
                    navController.navigate("note_detail/$noteId")
                },
                onCreateNew = {
                    navController.navigate(Screen.CreateNote.route)
                }
            )
        }

        composable(Screen.CreateNote.route) {
            CreateNoteScreen(
                onBack = { navController.navigateUp() },
                onNoteCreated = { navController.navigateUp() }
            )
        }

        composable("note_detail/{noteId}") { backStackEntry ->
            val noteId = backStackEntry.arguments?.getString("noteId") ?: return@composable
            NoteDetailScreen(
                noteId = noteId,
                onBack = { navController.navigateUp() },
                onDeleted = { navController.navigateUp() }
            )
        }

        composable(Screen.Settings.route) {
            if (currentUserId != null) {
                SettingsScreen(
                    userId = currentUserId!!,
                    onLogout = {
                        currentUserId = null
                        currentToken = null
                        navController.navigate(Screen.Login.route) {
                            popUpTo(Screen.Home.route) { inclusive = true }
                        }
                    }
                )
            }
        }
    }

    LaunchedEffect(navController) {
        navController.currentBackStackEntryFlow.collect { backStackEntry ->
            val route = backStackEntry.destination.route
            if (route == Screen.Login.route) {
                currentUserId = null
                currentToken = null
            }
        }
    }
}
