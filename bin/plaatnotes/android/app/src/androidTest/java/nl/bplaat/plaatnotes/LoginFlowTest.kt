package nl.bplaat.plaatnotes

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.hasText
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Integration tests for login flow
 * 
 * Prerequisites:
 * - Backend API running on localhost:8080
 * - Admin user created with email: admin@example.com, password: admin123
 */
@RunWith(AndroidJUnit4::class)
class LoginFlowTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun testSuccessfulLogin() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        composeTestRule.setContent {
            PlaatNotesApp(context = context)
        }

        // Verify login screen is displayed
        composeTestRule.onNodeWithText("PlaatNotes").assertExists()
        composeTestRule.onNodeWithText("Email").assertExists()
        composeTestRule.onNodeWithText("Password").assertExists()

        // Enter credentials
        composeTestRule.onNodeWithText("Email")
            .performClick()
            .performTextInput("admin@example.com")

        composeTestRule.onNodeWithText("Password")
            .performClick()
            .performTextInput("admin123")

        // Click login button
        composeTestRule.onNodeWithText("Login")
            .performClick()

        // Wait for navigation to main screen - should show bottom nav with Notes and Settings tabs
        composeTestRule.waitUntil(timeoutMillis = 10000) {
            composeTestRule.onAllNodes(hasText("Notes")).fetchSemanticsNodes().isNotEmpty()
        }

        // Verify main app is displayed (bottom nav tabs should be visible)
        composeTestRule.onNodeWithText("Notes").assertExists()
        composeTestRule.onNodeWithText("Settings").assertExists()
    }

    @Test
    fun testLoginWithInvalidCredentials() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        composeTestRule.setContent {
            PlaatNotesApp(context = context)
        }

        // Verify login screen is displayed
        composeTestRule.onNodeWithText("PlaatNotes").assertExists()

        // Enter invalid credentials
        composeTestRule.onNodeWithText("Email")
            .performClick()
            .performTextInput("invalid@example.com")

        composeTestRule.onNodeWithText("Password")
            .performClick()
            .performTextInput("wrongpassword")

        // Click login button
        composeTestRule.onNodeWithText("Login")
            .performClick()

        // Wait for error message to appear (or still on login screen)
        composeTestRule.waitUntil(timeoutMillis = 10000) {
            composeTestRule.onAllNodes(hasText("failed") or hasText("Error") or hasText("Login"))
                .fetchSemanticsNodes().size >= 1
        }

        // Verify still on login screen (Login button should still exist)
        composeTestRule.onNodeWithText("Login").assertExists()
    }

    @Test
    fun testLoginWithEmptyEmail() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        composeTestRule.setContent {
            PlaatNotesApp(context = context)
        }

        // Verify login screen
        composeTestRule.onNodeWithText("PlaatNotes").assertExists()

        // Enter only password
        composeTestRule.onNodeWithText("Password")
            .performClick()
            .performTextInput("admin123")

        // Try to click login
        composeTestRule.onNodeWithText("Login")
            .performClick()

        // Should still be on login screen
        composeTestRule.onNodeWithText("Email").assertExists()
    }

    @Test
    fun testLoginWithEmptyPassword() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        composeTestRule.setContent {
            PlaatNotesApp(context = context)
        }

        // Verify login screen
        composeTestRule.onNodeWithText("PlaatNotes").assertExists()

        // Enter only email
        composeTestRule.onNodeWithText("Email")
            .performClick()
            .performTextInput("admin@example.com")

        // Try to click login
        composeTestRule.onNodeWithText("Login")
            .performClick()

        // Should still be on login screen
        composeTestRule.onNodeWithText("Password").assertExists()
    }
}
