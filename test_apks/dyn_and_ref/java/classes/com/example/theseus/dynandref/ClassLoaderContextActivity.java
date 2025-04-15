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

public class ClassLoaderContextActivity extends Activity {
    public String classLoaderName;

    public String getdexfile(String name) {
        File dexfile = new File(getCacheDir(), name);
        dexfile.setReadOnly();
        Log.e("DEBUG", dexfile.getPath());
        return dexfile.getPath();
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        Intent intent = getIntent();
        classLoaderName = intent.getStringExtra("classLoaderName");

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

        scrollView.addView(linLayout);
        relLayout.addView(scrollView);
        setContentView(relLayout);

        Activity ac = this;

        b1.setText("Collision With Parent");
        b1.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                nextActivity(classLoaderName, true, true);
            }
        });

        b2.setText("Collision Without Parent");
        b2.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                nextActivity(classLoaderName, true, false);
            }
        });

        b3.setText("No Collision With Parent");
        b3.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                nextActivity(classLoaderName, false, true);
            }
        });

        b4.setText("No Collision Without Parent");
        b4.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                nextActivity(classLoaderName, false, false);
            }
        });
    }

    public void nextActivity(String classLoaderName, boolean hasCollision, boolean hasParent) {
        Class cl = null;
        if  (classLoaderName.equals("DelegateLastClassLoader") && hasCollision && hasParent) {
            cl = CollisionWithParentDelegateLastClassLoaderActivity.class;
        } else if  (classLoaderName.equals("DelegateLastClassLoader") && hasCollision && !hasParent) {
            cl = CollisionWithParentDelegateLastClassLoaderActivity.class;
        } else if  (classLoaderName.equals("DelegateLastClassLoader") && !hasCollision && hasParent) {
            cl = NoCollisionWithoutParentDelegateLastClassLoaderActivity.class;
        } else if  (classLoaderName.equals("DelegateLastClassLoader") && !hasCollision && !hasParent) {
            cl = NoCollisionWithoutParentDelegateLastClassLoaderActivity.class;
        } else if  (classLoaderName.equals("DexClassLoader") && hasCollision && hasParent) {
            cl = CollisionWithParentDexClassLoaderActivity.class;
        } else if  (classLoaderName.equals("DexClassLoader") && hasCollision && !hasParent) {
            cl = CollisionWithParentDexClassLoaderActivity.class;
        } else if  (classLoaderName.equals("DexClassLoader") && !hasCollision && hasParent) {
            cl = NoCollisionWithoutParentDexClassLoaderActivity.class;
        } else if  (classLoaderName.equals("DexClassLoader") && !hasCollision && !hasParent) {
            cl = NoCollisionWithoutParentDexClassLoaderActivity.class;
        } else if  (classLoaderName.equals("InMemoryDexClassLoader") && hasCollision && hasParent) {
            cl = CollisionWithParentInMemoryDexClassLoaderActivity.class;
        } else if  (classLoaderName.equals("InMemoryDexClassLoader") && hasCollision && !hasParent) {
            cl = CollisionWithParentInMemoryDexClassLoaderActivity.class;
        } else if  (classLoaderName.equals("InMemoryDexClassLoader") && !hasCollision && hasParent) {
            cl = NoCollisionWithoutParentInMemoryDexClassLoaderActivity.class;
        } else if  (classLoaderName.equals("InMemoryDexClassLoader") && !hasCollision && !hasParent) {
            cl = NoCollisionWithoutParentInMemoryDexClassLoaderActivity.class;
        } else if  (classLoaderName.equals("PathClassLoader") && hasCollision && hasParent) {
            cl = CollisionWithParentPathClassLoaderActivity.class;
        } else if  (classLoaderName.equals("PathClassLoader") && hasCollision && !hasParent) {
            cl = CollisionWithParentPathClassLoaderActivity.class;
        } else if  (classLoaderName.equals("PathClassLoader") && !hasCollision && hasParent) {
            cl = NoCollisionWithoutParentPathClassLoaderActivity.class;
        } else if  (classLoaderName.equals("PathClassLoader") && !hasCollision && !hasParent) {
            cl = NoCollisionWithoutParentPathClassLoaderActivity.class;
        } else {
            Log.e("THESEUS", "error: unknown activity for " + classLoaderName + " with hasCollision = " + hasCollision + " and hasParent = " + hasParent);
            return;
        };
        Intent intent = new Intent(this, cl);
        intent.putExtra("classLoaderName", classLoaderName);
        intent.putExtra("collision", hasCollision);
        intent.putExtra("parent", hasParent);
        startActivity(intent);
    }
}
