package com.example.theseus;

import android.app.Activity;
import android.app.AlertDialog;

import java.io.InputStream;
import java.io.OutputStream;
import java.io.IOException;

public class Utils {
    public static String source() {
        return "Secret";
    }
    public static String source(String tag) {
        return "[" + tag + "] Secret";
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
    public static void copy(InputStream in, OutputStream out) throws IOException {
        byte[] buffer = new byte[1024];
        int read;
        while((read = in.read(buffer)) != -1){
            out.write(buffer, 0, read);
        }
    }
}
