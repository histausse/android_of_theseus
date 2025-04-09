package com.example.theseus.dynandref;

import android.app.Activity;
import android.util.Log;
import java.lang.ClassLoader;
import dalvik.system.PathClassLoader;
import dalvik.system.InMemoryDexClassLoader;
import dalvik.system.DexClassLoader;
import dalvik.system.DelegateLastClassLoader;
import java.io.File;
import java.io.FileInputStream;
import java.nio.ByteBuffer;
import java.lang.reflect.Method;
import com.example.theseus.Utils;

public class Main {

    public static String getdexfile(Activity ac, String name) throws Exception {
        File dexfile = new File(ac.getCacheDir(), name);
        dexfile.setReadOnly();
        Log.e("DEBUG", dexfile.getPath());
        return dexfile.getPath();
    }

    public static ByteBuffer getdexbuffer(Activity ac, String name) throws Exception {
        File dexfile = new File(ac.getCacheDir(), name);
        FileInputStream in = new FileInputStream(dexfile);
        byte[] data = in.readAllBytes();
        return ByteBuffer.wrap(data);
    }

    public static void run(Activity ac, String clname, boolean isDirect, boolean hasParent, String methodType) {
        try {
        ClassLoader cl;
        ClassLoader parent;
        if (hasParent) {
            parent = Main.class.getClassLoader();
        } else {
            parent = null;
        }
        if (clname.equals("DelegateLastClassLoader")) {
            cl = new DelegateLastClassLoader(getdexfile(ac, "a.dex"), parent);
        } else if (clname.equals("DexClassLoader")) {
            cl = new DexClassLoader(getdexfile(ac, "a.dex"), null, null, parent);
        } else if (clname.equals("InMemoryDexClassLoader")) {
            cl = new InMemoryDexClassLoader(getdexbuffer(ac, "a.dex"), parent);
        } else if (clname.equals("PathClassLoader")) {
            cl = new PathClassLoader(getdexfile(ac, "a.dex"), parent);
        } else {
            cl = Main.class.getClassLoader();
        }

        Class clz = cl.loadClass("com.example.theseus.dynandref.Collider");
        Object[] args = {
            true, 
            (byte)42, 
            (short)666, 
            '*', 
            0xDEAD_BEEF, 
            0xD1AB011C_5EAF00DL, 
            0.99f, 
            3.1415926535897932384626433d, 
            "", 
            new String[] {"some", "strings"}
        };

        if (methodType.equals("Virtual")) {
            Method mth = clz.getMethod("virtTransfer", boolean.class, byte.class, short.class, char.class, int.class, long.class, float.class, double.class, String.class, String[].class);
            Object instance = clz.getDeclaredConstructor().newInstance();
            invoke(ac, instance, mth, args);
        } else if (methodType.equals("Static")) {
            Method mth = clz.getMethod("staticTransfer", boolean.class, byte.class, short.class, char.class, int.class, long.class, float.class, double.class, String.class, String[].class);
            invoke(ac, null, mth, args);
        } else if (methodType.equals("Extended")) {
            Method mth = clz.getMethod("extendedTransfer", boolean.class, byte.class, short.class, char.class, int.class, long.class, float.class, double.class, String.class, String[].class);
            Object instance = clz.getDeclaredConstructor().newInstance();
            invoke(ac, instance, mth, args);
        } else if (methodType.equals("Interface")) {
            Method mth = clz.getMethod("interTransfer", boolean.class, byte.class, short.class, char.class, int.class, long.class, float.class, double.class, String.class, String[].class);
            Object instance = clz.getDeclaredConstructor().newInstance();
            invoke(ac, instance, mth, args);
        } else if (methodType.equals("Interface Static")) {
            clz = cl.loadClass("com.example.theseus.dynandref.ICollider$-CC");
            Method mth = clz.getMethod("staticInterfaceTransfer", boolean.class, byte.class, short.class, char.class, int.class, long.class, float.class, double.class, String.class, String[].class);
            invoke(ac, null, mth, args);
        } else if (methodType.equals("Factory Pattern")) {
            return;
        } else {
            return;
        };
        } catch (Exception e) {
            Log.e("THESEUS", "error:", e);
        }
    }

    public static void invoke(Activity ac, Object instance, Method mth, Object[] args) throws Exception {
        args[8] = Utils.source();
        String res = (String)mth.invoke(instance, args);
        Utils.sink(ac, res);
    }
}
