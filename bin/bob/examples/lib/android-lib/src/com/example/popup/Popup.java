package com.example.popup;

import android.app.AlertDialog;
import android.content.Context;

public class Popup {
    private final Context context;
    private final String text;

    public Popup(Context context, String text) {
        this.context = context;
        this.text = text;
    }

    public void show() {
        new AlertDialog.Builder(context)
            .setTitle(context.getString(R.string.popup_title))
            .setMessage(text)
            .setPositiveButton("OK", (dialog, id) -> dialog.dismiss())
            .create()
            .show();
    }
}
