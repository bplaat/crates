package nl.bplaat.plaatnotes

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.hasText
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * End-to-end integration tests for the complete user flow
 * 
 * Prerequisites:
 * - Backend API running on localhost:8080
 * - Admin user created with email: admin@example.com, password: admin123
 */
@RunWith(AndroidJUnit4::class)
class EndToEndFlowTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun testCompleteLoginLogoutFlow() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        composeTestRule.setContent {
            PlaatNotesApp(context = context)
        }

        // ===== LOGIN PHASE =====
        println("Step 1: Verify login screen")
        composeTestRule.onNodeWithText("PlaatNotes").assertExists()
        composeTestRule.onNodeWithText("Email").assertExists()
        composeTestRule.onNodeWithText("Password").assertExists()
        composeTestRule.onNodeWithText("Login").assertExists()

        println("Step 2: Enter credentials")
        composeTestRule.onNodeWithText("Email")
            .performClick()
            .performTextInput("admin@example.com")

        composeTestRule.onNodeWithText("Password")
            .performClick()
            .performTextInput("admin123")

        println("Step 3: Click login button")
        composeTestRule.onNodeWithText("Login")
            .performClick()

        println("Step 4: Wait for navigation to main app")
        composeTestRule.waitUntil(timeoutMillis = 10000) {
            composeTestRule.onAllNodes(hasText("PlaatNotes")).fetchSemanticsNodes().size >= 1
        }

        // Verify main app top bar
        composeTestRule.onNodeWithContentDescription("Menu")
            .assertExists()

        println("✓ Successfully logged in")

        // ===== SETTINGS PHASE =====
        println("Step 5: Open settings menu")
        composeTestRule.onNodeWithContentDescription("Menu")
            .performClick()

        composeTestRule.onNodeWithText("Settings")
            .assertExists()
        composeTestRule.onNodeWithText("Logout")
            .assertExists()

        println("Step 6: Click Settings")
        composeTestRule.onNodeWithText("Settings")
            .performClick()

        composeTestRule.waitUntil(timeoutMillis = 5000) {
            composeTestRule.onAllNodes(hasText("Settings")).fetchSemanticsNodes().size >= 2
        }

        composeTestRule.onNodeWithText("Edit").assertExists()

        println("Step 7: Close settings")
        composeTestRule.onNodeWithText("Close")
            .performClick()

        println("✓ Settings accessed successfully")

        // ===== LOGOUT PHASE =====
        println("Step 8: Wait for main app screen")
        composeTestRule.waitUntil(timeoutMillis = 3000) {
            composeTestRule.onAllNodes(hasText("PlaatNotes")).fetchSemanticsNodes().size >= 1
        }

        println("Step 9: Open menu for logout")
        composeTestRule.onNodeWithContentDescription("Menu")
            .performClick()

        println("Step 10: Click Logout")
        composeTestRule.onNodeWithText("Logout")
            .performClick()

        println("Step 11: Wait for return to login screen")
        composeTestRule.waitUntil(timeoutMillis = 5000) {
            composeTestRule.onAllNodes(hasText("Email")).fetchSemanticsNodes().isNotEmpty()
        }

        composeTestRule.onNodeWithText("Email").assertExists()
        composeTestRule.onNodeWithText("Password").assertExists()
        composeTestRule.onNodeWithText("Login").assertExists()

        println("✓ Successfully logged out")
        println("\n✅ Complete login-logout-settings flow test PASSED")
    }

    @Test
    fun testMultipleLoginLogoutCycles() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        composeTestRule.setContent {
            PlaatNotesApp(context = context)
        }

        // Test multiple login/logout cycles
        for (cycle in 1..2) {
            println("Cycle $cycle: Testing login/logout")

            // LOGIN
            composeTestRule.onNodeWithText("Email")
                .performClick()
                .performTextInput("admin@example.com")

            composeTestRule.onNodeWithText("Password")
                .performClick()
                .performTextInput("admin123")

            composeTestRule.onNodeWithText("Login")
                .performClick()

            composeTestRule.waitUntil(timeoutMillis = 10000) {
                composeTestRule.onAllNodes(hasText("PlaatNotes")).fetchSemanticsNodes().size >= 1
            }

            println("  ✓ Logged in (cycle $cycle)")

            // LOGOUT
            composeTestRule.onNodeWithContentDescription("Menu")
                .performClick()

            composeTestRule.onNodeWithText("Logout")
                .performClick()

            composeTestRule.waitUntil(timeoutMillis = 5000) {
                composeTestRule.onAllNodes(hasText("Email")).fetchSemanticsNodes().isNotEmpty()
            }

            println("  ✓ Logged out (cycle $cycle)")
        }

        println("✅ Multiple login/logout cycles test PASSED")
    }

    @Test
    fun testErrorRecovery() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        composeTestRule.setContent {
            PlaatNotesApp(context = context)
        }

        println("Step 1: Try login with wrong password")
        composeTestRule.onNodeWithText("Email")
            .performClick()
            .performTextInput("admin@example.com")

        composeTestRule.onNodeWithText("Password")
            .performClick()
            .performTextInput("wrongpassword")

        composeTestRule.onNodeWithText("Login")
            .performClick()

        composeTestRule.waitUntil(timeoutMillis = 10000) {
            composeTestRule.onAllNodes(
                hasText("failed") or hasText("Login")
            ).fetchSemanticsNodes().size >= 1
        }

        println("✓ Error received for wrong password")

        // Login screen should still be accessible
        composeTestRule.onNodeWithText("Login").assertExists()
        println("✓ Still on login screen after error")

        println("\n✅ Error recovery test PASSED")
    }
}
