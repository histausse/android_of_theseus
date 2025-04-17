package com.example.theseus.dynandref;

import android.app.Activity;
import android.os.Bundle;
import android.util.Log;

import android.widget.RelativeLayout;
import android.widget.ScrollView;
import android.widget.LinearLayout;
import android.view.ViewGroup;
import android.view.View;
import android.widget.Button;
import android.content.res.ColorStateList;

import java.lang.ClassLoader;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Constructor;
import java.lang.ClassNotFoundException;

import android.content.Intent;
import android.content.res.AssetManager;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.IOException;
import java.io.File;
import java.io.FileOutputStream;
import java.lang.reflect.InvocationTargetException;
import android.content.Context;
import dalvik.system.PathClassLoader;
import dalvik.system.DelegateLastClassLoader;
import java.lang.reflect.Method;

import com.example.theseus.Utils;


import java.util.Arrays;

public class MethodActivity extends Activity {
    public String classLoaderName;
    public boolean hasCollision;
    public boolean hasParent;


    public String getdexfile(String name) {
        File dexfile = new File(getCacheDir(), name);
        dexfile.setReadOnly();
        return dexfile.getPath();
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        Intent intent = getIntent();
        classLoaderName = intent.getStringExtra("classLoaderName");
        hasCollision = intent.getBooleanExtra("collision", false);
        hasParent = intent.getBooleanExtra("parent", false);

        ColorStateList buttonColor = ColorStateList.valueOf(0xff808080);

        RelativeLayout relLayout = new RelativeLayout(this);
        relLayout.generateViewId();

        ScrollView scrollView = new ScrollView(this);
        scrollView.generateViewId();

        RelativeLayout.LayoutParams lp = new RelativeLayout.LayoutParams(
            ViewGroup.LayoutParams.WRAP_CONTENT, 
            ViewGroup.LayoutParams.WRAP_CONTENT
        );
        lp.addRule(RelativeLayout.CENTER_IN_PARENT);

        LinearLayout linLayout = new LinearLayout(this);
        linLayout.generateViewId();
        linLayout.setLayoutParams(lp);
        linLayout.setOrientation(LinearLayout.VERTICAL);


        Button b1 = new Button(this);
        b1.generateViewId();
        linLayout.addView(b1);

        Button b2 = new Button(this);
        b2.generateViewId();
        linLayout.addView(b2);

        Button b3 = new Button(this);
        b3.generateViewId();
        linLayout.addView(b3);

        Button b4 = new Button(this);
        b4.generateViewId();
        linLayout.addView(b4);

        Button b5 = new Button(this);
        b5.generateViewId();
        //linLayout.addView(b5); // Static Interface Methods are a pain

        Button b6 = new Button(this);
        b6.generateViewId();
        if (!classLoaderName.equals("DelegateLastClassLoader") && hasParent) {
            // if no parent or use DelegateLastClassLoader, the type of the Interface is incompatible
            linLayout.addView(b6);
        }

        Button b7 = new Button(this);
        b7.generateViewId();
        if (!classLoaderName.equals("DelegateLastClassLoader") && hasParent) {
            // if no parent or use DelegateLastClassLoader, the type of the Interface is incompatible
            linLayout.addView(b7);
        }

        scrollView.addView(linLayout);
        relLayout.addView(scrollView);
        setContentView(relLayout);

        Activity ac = this;

        b1.setText("Virtual");
        b1.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                Main.run(ac, classLoaderName, hasCollision, hasParent, "Virtual");
            }
        });

        b2.setText("Static");
        b2.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                Main.run(ac, classLoaderName, hasCollision, hasParent, "Static");
            }
        });

        b3.setText("Extended");
        b3.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                Main.run(ac, classLoaderName, hasCollision, hasParent, "Extended");
            }
        });

        b4.setText("Interface");
        b4.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                Main.run(ac, classLoaderName, hasCollision, hasParent, "Interface");
            }
        });

        b5.setText("Interface Static");
        b5.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                Main.run(ac, classLoaderName, hasCollision, hasParent, "Interface Static");
            }
        });

        b6.setText("Factory Pattern Interface");
        b6.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                Main.run(ac, classLoaderName, hasCollision, hasParent, "Factory Pattern Interface");
            }
        });

        b7.setText("Factory Pattern Extend");
        b7.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                Main.run(ac, classLoaderName, hasCollision, hasParent, "Factory Pattern Extend");
            }
        });

        // Allow to start Main.run from intent for automation
        String methodType = intent.getStringExtra("methodType");
        if (methodType != null) {
            Main.run(ac, classLoaderName, hasCollision, hasParent, methodType);
        }
    }

    @Override
    protected void onNewIntent(Intent intent) {
        String classLoaderNameLocal = intent.getStringExtra("classLoaderName");
        boolean hasCollisionLocal = intent.getBooleanExtra("collision", false);
        boolean hasParentLocal = intent.getBooleanExtra("parent", false);
        String methodType = intent.getStringExtra("methodType");
        if (classLoaderNameLocal != null && methodType != null) {
            Main.run(this, classLoaderNameLocal, hasCollisionLocal, hasParentLocal, methodType);
        }
    }
}
