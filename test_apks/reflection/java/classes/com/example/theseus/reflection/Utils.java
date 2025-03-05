package com.example.theseus;

import android.app.Activity;
import android.app.AlertDialog;
import com.example.theseus.reflection.Reflectee;
import com.example.theseus.reflection.ChildReflectee;


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

    public static void testIsObject(Activity ac, Object obj) {
        //popup(ac, "Object", "Object was expected and found");
    }
    public static void testIsReflectee(Activity ac, Reflectee ref) {
        //popup(ac, "Reflectee", "Reflectee was expected and found");
    }
    public static void testIsChildReflectee(Activity ac, ChildReflectee ref) {
        //popup(ac, "ChildReflectee", "ChildReflectee was expected and found");
    }
}
