package com.example.hello;

import android.app.Activity;
import android.os.Bundle;
import android.widget.TextView;
import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;

public class MainActivity extends Activity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        // Read the name from assets/name.txt and display it in the label
        try (var inputStream = getAssets().open("name.txt");
             var reader = new BufferedReader(new InputStreamReader(inputStream))) {
            ((TextView)findViewById(R.id.label)).setText("Hello " + reader.readLine() + "!");
        } catch (IOException e) {
            e.printStackTrace();
        }
    }
}
