package com.example.theseus;

import android.app.Activity;
import android.os.Bundle;
import android.util.Log;

public class MainActivity extends Activity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        try {
            Main main = new Main(this);
            main.main();
        } catch (Exception e) {
            Log.i("THESEUS", "Error", e);
        }
    }
}
