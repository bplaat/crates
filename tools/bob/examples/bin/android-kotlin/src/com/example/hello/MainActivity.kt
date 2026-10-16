package com.example.hello

import android.app.Activity
import android.os.Bundle
import android.util.Log
import android.widget.Button
import android.widget.TextView
import java.io.BufferedReader
import java.io.IOException
import java.io.InputStreamReader
import com.example.popup.Popup

class MainActivity : Activity() {
    private val LOG_TAG = "hello"

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        Log.i(LOG_TAG, "Hello from Kotlin!")

        // Read the name from assets/name.txt and display it in the label
        try {
            BufferedReader(InputStreamReader(assets.open("name.txt"))).use { reader ->
                findViewById<TextView>(R.id.label).text = "Hello ${reader.readLine()} from Kotlin!"
            }
        } catch (e: IOException) {
            e.printStackTrace()
        }

        // Init button
        findViewById<Button>(R.id.button).setOnClickListener { Log.i(LOG_TAG, "Button clicked!") }

        // Show popup
        Popup(this, "Hello Android library!").show()
    }
}
