package com.example.popup

import android.app.AlertDialog
import android.content.Context

class Popup(private val context: Context, private val text: String) {
    fun show() {
        AlertDialog.Builder(context)
            .setTitle(context.getString(R.string.popup_title))
            .setMessage(text)
            .setPositiveButton("OK") { dialog, _ -> dialog.dismiss() }
            .create()
            .show()
    }
}
