package nl.bplaat.plaatnotes.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ExitToApp
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import nl.bplaat.plaatnotes.data.models.UserRole
import nl.bplaat.plaatnotes.ui.viewmodel.AuthViewModel
import nl.bplaat.plaatnotes.ui.viewmodel.UserViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    userId: String,
    onLogout: () -> Unit,
    authViewModel: AuthViewModel = viewModel(),
    userViewModel: UserViewModel = viewModel()
) {
    val userState by userViewModel.userState.collectAsState()
    var editMode by remember { mutableStateOf(false) }
    var passwordMode by remember { mutableStateOf(false) }

    var firstName by remember { mutableStateOf("") }
    var lastName by remember { mutableStateOf("") }
    var email by remember { mutableStateOf("") }

    var oldPassword by remember { mutableStateOf("") }
    var newPassword by remember { mutableStateOf("") }
    var confirmPassword by remember { mutableStateOf("") }

    LaunchedEffect(userId) {
        userViewModel.loadUser(userId)
    }

    LaunchedEffect(userState.user) {
        userState.user?.let {
            firstName = it.firstName
            lastName = it.lastName
            email = it.email
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Settings") },
                actions = {
                    IconButton(onClick = {
                        authViewModel.logout()
                        onLogout()
                    }) {
                        Icon(Icons.Filled.ExitToApp, "Logout")
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
                userState.isLoading && userState.user == null -> {
                    CircularProgressIndicator(
                        modifier = Modifier.align(Alignment.Center)
                    )
                }
                userState.user == null -> {
                    Text(
                        "Failed to load user",
                        modifier = Modifier.align(Alignment.Center)
                    )
                }
                else -> {
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .verticalScroll(rememberScrollState())
                            .padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        Text(
                            "Profile Information",
                            style = MaterialTheme.typography.headlineSmall
                        )

                        if (userState.updateSuccess) {
                            Text(
                                "Profile updated successfully",
                                color = MaterialTheme.colorScheme.primary,
                                style = MaterialTheme.typography.bodySmall
                            )
                        }

                        if (userState.error != null && !passwordMode) {
                            Text(
                                "Error: ${userState.error}",
                                color = MaterialTheme.colorScheme.error,
                                style = MaterialTheme.typography.bodySmall
                            )
                        }

                        if (!editMode) {
                            OutlinedCard(modifier = Modifier.fillMaxWidth()) {
                                Column(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(16.dp),
                                    verticalArrangement = Arrangement.spacedBy(8.dp)
                                ) {
                                    Text("Name: $firstName $lastName")
                                    Text("Email: $email")
                                }
                            }

                            Button(
                                onClick = { editMode = true },
                                modifier = Modifier.fillMaxWidth()
                            ) {
                                Text("Edit Profile")
                            }
                        } else {
                            TextField(
                                value = firstName,
                                onValueChange = { firstName = it },
                                label = { Text("First Name") },
                                modifier = Modifier.fillMaxWidth(),
                                enabled = !userState.isLoading
                            )

                            TextField(
                                value = lastName,
                                onValueChange = { lastName = it },
                                label = { Text("Last Name") },
                                modifier = Modifier.fillMaxWidth(),
                                enabled = !userState.isLoading
                            )

                            TextField(
                                value = email,
                                onValueChange = { email = it },
                                label = { Text("Email") },
                                modifier = Modifier.fillMaxWidth(),
                                enabled = !userState.isLoading
                            )

                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.spacedBy(8.dp)
                            ) {
                                Button(
                                    onClick = {
                                        editMode = false
                                        userViewModel.clearMessages()
                                    },
                                    modifier = Modifier
                                        .weight(1f)
                                        .height(48.dp),
                                    enabled = !userState.isLoading
                                ) {
                                    Text("Cancel")
                                }
                                Button(
                                    onClick = {
                                        userViewModel.updateUser(
                                            userId,
                                            firstName,
                                            lastName,
                                            email,
                                            UserRole.NORMAL
                                        )
                                        editMode = false
                                    },
                                    modifier = Modifier
                                        .weight(1f)
                                        .height(48.dp),
                                    enabled = !userState.isLoading
                                ) {
                                    Text("Save")
                                }
                            }
                        }

                        Divider()

                        Text(
                            "Change Password",
                            style = MaterialTheme.typography.headlineSmall
                        )

                        if (userState.passwordChangeSuccess) {
                            Text(
                                "Password changed successfully",
                                color = MaterialTheme.colorScheme.primary,
                                style = MaterialTheme.typography.bodySmall
                            )
                        }

                        if (!passwordMode) {
                            Button(
                                onClick = { passwordMode = true },
                                modifier = Modifier.fillMaxWidth()
                            ) {
                                Text("Change Password")
                            }
                        } else {
                            TextField(
                                value = oldPassword,
                                onValueChange = { oldPassword = it },
                                label = { Text("Current Password") },
                                visualTransformation = PasswordVisualTransformation(),
                                modifier = Modifier.fillMaxWidth(),
                                enabled = !userState.isLoading
                            )

                            TextField(
                                value = newPassword,
                                onValueChange = { newPassword = it },
                                label = { Text("New Password") },
                                visualTransformation = PasswordVisualTransformation(),
                                modifier = Modifier.fillMaxWidth(),
                                enabled = !userState.isLoading
                            )

                            TextField(
                                value = confirmPassword,
                                onValueChange = { confirmPassword = it },
                                label = { Text("Confirm Password") },
                                visualTransformation = PasswordVisualTransformation(),
                                modifier = Modifier.fillMaxWidth(),
                                enabled = !userState.isLoading
                            )

                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.spacedBy(8.dp)
                            ) {
                                Button(
                                    onClick = {
                                        passwordMode = false
                                        oldPassword = ""
                                        newPassword = ""
                                        confirmPassword = ""
                                        userViewModel.clearMessages()
                                    },
                                    modifier = Modifier
                                        .weight(1f)
                                        .height(48.dp),
                                    enabled = !userState.isLoading
                                ) {
                                    Text("Cancel")
                                }
                                Button(
                                    onClick = {
                                        if (newPassword == confirmPassword && newPassword.isNotEmpty()) {
                                            userViewModel.changePassword(
                                                userId,
                                                oldPassword,
                                                newPassword
                                            )
                                            passwordMode = false
                                            oldPassword = ""
                                            newPassword = ""
                                            confirmPassword = ""
                                        }
                                    },
                                    modifier = Modifier
                                        .weight(1f)
                                        .height(48.dp),
                                    enabled = !userState.isLoading && 
                                            newPassword.isNotEmpty() && 
                                            newPassword == confirmPassword
                                ) {
                                    Text("Update")
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
