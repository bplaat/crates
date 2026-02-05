package nl.bplaat.plaatnotes

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import nl.bplaat.plaatnotes.core.storage.TokenStorage
import nl.bplaat.plaatnotes.ui.screens.LoginScreen
import nl.bplaat.plaatnotes.ui.screens.NotesHomeScreen
import nl.bplaat.plaatnotes.ui.screens.NoteDetailScreen
import nl.bplaat.plaatnotes.ui.screens.CreateNoteScreen
import nl.bplaat.plaatnotes.ui.screens.SettingsScreen
import nl.bplaat.plaatnotes.ui.theme.PlaatNotesTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            PlaatNotesTheme {
                PlaatNotesApp(context = this@MainActivity)
            }
        }
    }
}

@Composable
fun PlaatNotesApp(context: android.content.Context) {
    var userId by remember { mutableStateOf<String?>(null) }
    var token by remember { mutableStateOf<String?>(null) }
    val navController = rememberNavController()

    LaunchedEffect(Unit) {
        TokenStorage.getToken(context).collect { savedToken ->
            if (!savedToken.isNullOrEmpty()) {
                token = savedToken
            }
        }
        TokenStorage.getUserId(context).collect { savedUserId ->
            if (!savedUserId.isNullOrEmpty()) {
                userId = savedUserId
            }
        }
    }

    when {
        userId != null && token != null -> {
            MainAppNavigation(
                userId = userId!!,
                token = token!!,
                navController = navController,
                onLogout = {
                    userId = null
                    token = null
                    navController.navigate("login") {
                        popUpTo(0) { inclusive = true }
                    }
                }
            )
        }
        else -> {
            LoginScreen(
                onLoginSuccess = { id, t ->
                    userId = id
                    token = t
                }
            )
        }
    }
}

@Composable
fun MainAppNavigation(
    userId: String,
    token: String,
    navController: NavHostController,
    onLogout: () -> Unit
) {
    var currentTab by remember { mutableStateOf("home") }

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        bottomBar = {
            NavigationBar {
                NavigationBarItem(
                    icon = { Icon(Icons.Filled.Home, contentDescription = "Home") },
                    label = { Text("Notes") },
                    selected = currentTab == "home",
                    onClick = {
                        currentTab = "home"
                        navController.navigate("home") {
                            popUpTo("home") { inclusive = true }
                        }
                    }
                )
                NavigationBarItem(
                    icon = { Icon(Icons.Filled.Settings, contentDescription = "Settings") },
                    label = { Text("Settings") },
                    selected = currentTab == "settings",
                    onClick = {
                        currentTab = "settings"
                        navController.navigate("settings") {
                            popUpTo("home")
                        }
                    }
                )
            }
        }
    ) { innerPadding ->
        NavHost(
            navController = navController,
            startDestination = "home",
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
        ) {
            composable("home") {
                currentTab = "home"
                NotesHomeScreen(
                    onNoteSelected = { noteId ->
                        navController.navigate("note_detail/$noteId")
                    },
                    onCreateNew = {
                        navController.navigate("create_note")
                    }
                )
            }

            composable("create_note") {
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

            composable("settings") {
                currentTab = "settings"
                SettingsScreen(
                    userId = userId,
                    onLogout = onLogout
                )
            }

            composable("login") {
                LoginScreen(
                    onLoginSuccess = { _, _ ->
                        onLogout()
                    }
                )
            }
        }
    }
}
