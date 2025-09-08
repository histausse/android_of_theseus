package com.example.theseus;

import android.app.Activity;
import android.app.AlertDialog;


public class Utils {
    public static String source(String tag) {
        return "SecretData[" + tag + "]";
    }

    public static void popup(Activity ac, String title, String msg) {
        (new AlertDialog.Builder(ac))
            .setMessage(msg)
            .setTitle(title)
            .create()
            .show();
    }

    public static void sink(Activity ac, String data) {
        popup(ac, "Data leak:", data);
    }
}
