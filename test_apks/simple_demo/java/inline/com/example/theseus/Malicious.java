package com.example.theseus;
import android.app.Activity;

public class Malicious {
    public static String get_data(String data, Activity ac) {
        return "SecretData[" + data + "]";
    }

    public static String send_data(String data, Activity ac) {
        Utils.sink(ac, data);
        return null;
    }
}
