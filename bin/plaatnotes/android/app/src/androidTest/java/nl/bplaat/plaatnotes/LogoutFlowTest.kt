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
 * Integration tests for logout flow and settings access
 * 
 * Prerequisites:
 * - Backend API running on localhost:8080
 * - Admin user created with email: admin@example.com, password: admin123
 */
@RunWith(AndroidJUnit4::class)
class LogoutFlowTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun testAccessSettingsScreen() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        composeTestRule.setContent {
            PlaatNotesApp(context = context)
        }

        // Step 1: Login
        loginAsAdmin()

        // Step 2: Wait for main app to load
        composeTestRule.waitUntil(timeoutMillis = 10000) {
            composeTestRule.onAllNodes(hasText("Settings")).fetchSemanticsNodes().isNotEmpty()
        }

        // Step 3: Click Settings tab
        composeTestRule.onNodeWithText("Settings")
            .performClick()

        // Step 4: Verify settings screen appears
        composeTestRule.waitUntil(timeoutMillis = 5000) {
            composeTestRule.onAllNodes(hasText("Name:")).fetchSemanticsNodes().isNotEmpty()
        }

        // User info should be displayed
        composeTestRule.onNodeWithText("Email:").assertExists()
        composeTestRule.onNodeWithText("Role:").assertExists()
    }

    @Test
    fun testSettingsHasExitOption() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        composeTestRule.setContent {
            PlaatNotesApp(context = context)
        }

        // Login
        loginAsAdmin()

        // Wait for main app
        composeTestRule.waitUntil(timeoutMillis = 10000) {
            composeTestRule.onAllNodes(hasText("Settings")).fetchSemanticsNodes().isNotEmpty()
        }

        // Navigate to Settings
        composeTestRule.onNodeWithText("Settings")
            .performClick()

        // Wait for settings screen
        composeTestRule.waitUntil(timeoutMillis = 5000) {
            composeTestRule.onAllNodes(hasText("ExitToApp")).fetchSemanticsNodes().isNotEmpty()
        }

        // Logout button (ExitToApp icon) should be present
        val exitButtons = composeTestRule.onAllNodes(hasText("ExitToApp")).fetchSemanticsNodes()
        assert(exitButtons.isNotEmpty()) { "Logout button should be in settings" }
    }

    @Test
    fun testSwitchBetweenNotesAndSettings() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        composeTestRule.setContent {
            PlaatNotesApp(context = context)
        }

        // Login
        loginAsAdmin()

        // Wait for main app
        composeTestRule.waitUntil(timeoutMillis = 10000) {
            composeTestRule.onAllNodes(hasText("Notes")).fetchSemanticsNodes().isNotEmpty()
        }

        // Switch to Settings
        composeTestRule.onNodeWithText("Settings")
            .performClick()

        composeTestRule.waitUntil(timeoutMillis = 3000) {
            composeTestRule.onAllNodes(hasText("Name:")).fetchSemanticsNodes().isNotEmpty()
        }

        // Switch back to Notes
        composeTestRule.onNodeWithText("Notes")
            .performClick()

        // Notes screen should be visible
        composeTestRule.waitUntil(timeoutMillis = 3000) {
            composeTestRule.onAllNodes(hasText("Notes")).fetchSemanticsNodes().isNotEmpty()
        }
    }

    @Test
    fun testSettingsNavigationMultipleTimes() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        composeTestRule.setContent {
            PlaatNotesApp(context = context)
        }

        // Login
        loginAsAdmin()

        // Wait for main app
        composeTestRule.waitUntil(timeoutMillis = 10000) {
            composeTestRule.onAllNodes(hasText("Notes")).fetchSemanticsNodes().isNotEmpty()
        }

        // Navigate to Settings multiple times
        for (i in 1..3) {
            composeTestRule.onNodeWithText("Settings")
                .performClick()

            composeTestRule.waitUntil(timeoutMillis = 3000) {
                composeTestRule.onAllNodes(hasText("Name:")).fetchSemanticsNodes().isNotEmpty()
            }

            // Navigate back to Notes
            composeTestRule.onNodeWithText("Notes")
                .performClick()

            composeTestRule.waitUntil(timeoutMillis = 3000) {
                composeTestRule.onAllNodes(hasText("Notes")).fetchSemanticsNodes().isNotEmpty()
            }
        }
    }

    // Helper function to login as admin
    private fun loginAsAdmin() {
        // Verify login screen is displayed
        composeTestRule.onNodeWithText("PlaatNotes").assertExists()

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
    }
}
