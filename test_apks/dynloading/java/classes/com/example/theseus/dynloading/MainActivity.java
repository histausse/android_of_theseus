package com.example.theseus.dynloading;

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

import android.content.res.AssetManager;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.IOException;
import java.io.File;
import java.io.FileOutputStream;
import java.lang.reflect.InvocationTargetException;
import android.content.Context;
import dalvik.system.PathClassLoader;
import java.lang.reflect.Method;

import com.example.theseus.Utils;


import java.util.Arrays;

public class MainActivity extends Activity {

    public void setup() {
        AssetManager assetManager = getAssets();
        InputStream in = null;
        OutputStream out = null;
        File outFile = null;
        try {
            in = assetManager.open("a.dex");
            outFile = new File(getCacheDir(), "a.dex_"); // .dex_ because android does not like people writing .dex
            out = new FileOutputStream(outFile);
            Utils.copy(in, out);
            outFile.renameTo(new File(getCacheDir(), "a.dex")); // security?
        } catch (IOException e) {}
        try {
            in.close();
        } catch (IOException e) {}
        try {
            out.close();
        } catch (IOException e) {}
    }

    public String getdexfile(String name) {
        File dexfile = new File(getCacheDir(), name);
        dexfile.setReadOnly();
        Log.e("DEBUG", dexfile.getPath());
        return dexfile.getPath();
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        setup();

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

        b1.setText("Direct With Parent");
        b1.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                try {
                    directWithParent();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b2.setText("Direct Without Parent");
        b2.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                try {
                    directWithoutParent();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b3.setText("Indirect With Parent");
        b3.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                try {
                     indirectWithParent();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b4.setText("Indirect Without Parent");
        b4.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                try {
                    indirectWithoutParent();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });
    }

    public void directWithParent() {
        try {
            PathClassLoader cl = new PathClassLoader(getdexfile("a.dex"), MainActivity.class.getClassLoader());
            Class clz = cl.loadClass("com.example.theseus.dynloading.Collider");
            Method mth = clz.getMethod("getColliderId");
            String id = (String)mth.invoke(null);
            //Utils.popup(this, "Result", id);
            String expectedId = "MainAPK";
            if (id.equals(expectedId)) {
                Utils.popup(this, "OK", "The right class was loaded: " + id);
            } else {
                Utils.popup(this, "BAD", "The wrong class was loaded: id = " + id + " expected id = " + expectedId);
            }
        } catch (ClassNotFoundException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
        catch (NoSuchMethodException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
        catch (IllegalAccessException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
        catch (InvocationTargetException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
    }

    public void directWithoutParent() {
        try {
            PathClassLoader cl = new PathClassLoader(getdexfile("a.dex"), null);
            Class clz = cl.loadClass("com.example.theseus.dynloading.Collider");
            Method mth = clz.getMethod("getColliderId");
            String id = (String)mth.invoke(null);
            //Utils.popup(this, "Result", id);
            Utils.popup(this, "TEST", clz.descriptorString());
            String expectedId = "A";
            if (id.equals(expectedId)) {
                Utils.popup(this, "OK", "The right class was loaded: " + id);
            } else {
                Utils.popup(this, "BAD", "The wrong class was loaded: id = " + id + " expected id = " + expectedId);
            }
        } catch (ClassNotFoundException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
        catch (NoSuchMethodException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
        catch (IllegalAccessException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
        catch (InvocationTargetException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
    }

    public void indirectWithParent() {
        try {
            PathClassLoader cl = new PathClassLoader(getdexfile("a.dex"), MainActivity.class.getClassLoader());
            Class clz = cl.loadClass("com.example.theseus.dynloading.AMain");
            Method mth = clz.getMethod("getColliderId");
            String id = (String)mth.invoke(null);
            //Utils.popup(this, "Result", id);
            String expectedId = "MainAPK";
            if (id.equals(expectedId)) {
                Utils.popup(this, "OK", "The right class was loaded: " + id);
            } else {
                Utils.popup(this, "BAD", "The wrong class was loaded: id = " + id + " expected id = " + expectedId);
            }
        } catch (ClassNotFoundException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
        catch (NoSuchMethodException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
        catch (IllegalAccessException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
        catch (InvocationTargetException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
    }

    public void indirectWithoutParent() {
        try {
            PathClassLoader cl = new PathClassLoader(getdexfile("a.dex"), null);
            Class clz = cl.loadClass("com.example.theseus.dynloading.AMain");
            Method mth = clz.getMethod("getColliderId");
            String id = (String)mth.invoke(null);
            //Utils.popup(this, "Result", id);
            String expectedId = "A";
            if (id.equals(expectedId)) {
                Utils.popup(this, "OK", "The right class was loaded: " + id);
            } else {
                Utils.popup(this, "BAD", "The wrong class was loaded: id = " + id + " expected id = " + expectedId);
            }
        } catch (ClassNotFoundException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
        catch (NoSuchMethodException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
        catch (IllegalAccessException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
        catch (InvocationTargetException e) {
            Log.e("DEBUG", "ERROR: ", e);
        }
    }
}
