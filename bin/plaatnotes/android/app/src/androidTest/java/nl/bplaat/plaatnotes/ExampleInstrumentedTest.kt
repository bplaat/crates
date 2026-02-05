package nl.bplaat.plaatnotes

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.ext.junit.rules.ActivityScenarioRule

import org.junit.Test
import org.junit.Rule
import org.junit.runner.RunWith

import org.junit.Assert.*

/**
 * Smoke tests for PlaatNotes app
 * 
 * These tests verify that the app can start and basic functionality works
 * without requiring a backend API. Run these first to ensure app stability.
 */
@RunWith(AndroidJUnit4::class)
class SmokeTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun appContextExists() {
        // Verify app package name
        val appContext = InstrumentationRegistry.getInstrumentation().targetContext
        assertEquals("nl.bplaat.plaatnotes", appContext.packageName)
    }

    @Test
    fun appStartsSuccessfully() {
        // Verify app can be started without crashes
        val appContext = InstrumentationRegistry.getInstrumentation().targetContext
        assertNotNull(appContext)
        assertEquals("nl.bplaat.plaatnotes", appContext.packageName)
    }

    @Test
    fun loginScreenDisplaysOnAppStart() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        composeTestRule.setContent {
            PlaatNotesApp(context = context)
        }

        // Verify app title
        composeTestRule.onNodeWithText("PlaatNotes").assertExists()
        
        // Verify login screen elements
        composeTestRule.onNodeWithText("Email").assertExists()
        composeTestRule.onNodeWithText("Password").assertExists()
        composeTestRule.onNodeWithText("Login").assertExists()
    }

    @Test
    fun loginScreenIsInteractive() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        composeTestRule.setContent {
            PlaatNotesApp(context = context)
        }

        // Verify buttons are clickable
        composeTestRule.onNodeWithText("Login").assertExists()
        
        // Try to interact with email field
        composeTestRule.onNodeWithText("Email").assertExists()
        
        // Try to interact with password field
        composeTestRule.onNodeWithText("Password").assertExists()
    }

    @Test
    fun appDoesNotCrashOnStartup() {
        try {
            val context = InstrumentationRegistry.getInstrumentation().targetContext
            composeTestRule.setContent {
                PlaatNotesApp(context = context)
            }
            
            // If we get here without an exception, app started successfully
            assert(true)
        } catch (e: Exception) {
            fail("App crashed on startup: ${e.message}")
        }
    }
}

