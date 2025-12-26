package com.example.hello;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;

import android.app.Activity;
import android.os.Bundle;
import android.util.Log;
import android.widget.TextView;

import com.example.popup.Popup;

public class MainActivity extends Activity {
    public static final String LOG_TAG = "hello";

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        Log.i(LOG_TAG, "Hello from Java!");

        // Read the name from assets/name.txt and display it in the label
        try (var reader = new BufferedReader(new InputStreamReader(getAssets().open("name.txt")))) {
            ((TextView)findViewById(R.id.label)).setText("Hello " + reader.readLine() + " from Java!");
        } catch (IOException e) {
            Log.e(LOG_TAG, "Error reading greeting message", e);
        }

        // Init button
        findViewById(R.id.button).setOnClickListener(v -> { Log.i(LOG_TAG, "Button clicked!"); });

        // Show popup
        new Popup(this, "Hello Android library!").show();
    }
}
