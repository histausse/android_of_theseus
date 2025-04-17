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

    public static void run(Activity ac, String clname, boolean hasCollision, boolean hasParent, String methodType) {
        try {
        Log.i("THESEUS", "clname: " + clname + ", hasCollision: " + hasCollision + ", hasParent: " + hasParent + ", methodType: " + methodType);
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

        Class clz = null;
        if (hasCollision) {
            clz = cl.loadClass("com.example.theseus.dynandref.Collider");
        } else {
            clz = cl.loadClass("com.example.theseus.dynandref.AReflectee");
        }

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
            invoke(ac,
                true,
                clz,
                mth,
                args,
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
            );
        } else if (methodType.equals("Static")) {
            Method mth = clz.getMethod("staticTransfer", boolean.class, byte.class, short.class, char.class, int.class, long.class, float.class, double.class, String.class, String[].class);
            invoke(ac,
                false,
                clz,
                mth,
                args,
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
            );
        } else if (methodType.equals("Extended")) {
            Method mth = clz.getMethod("extendedTransfer", boolean.class, byte.class, short.class, char.class, int.class, long.class, float.class, double.class, String.class, String[].class);
            invoke(ac,
                true,
                clz,
                mth,
                args,
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
            );
        } else if (methodType.equals("Interface")) {
            Method mth = clz.getMethod("interTransfer", boolean.class, byte.class, short.class, char.class, int.class, long.class, float.class, double.class, String.class, String[].class);
            invoke(ac,
                true,
                clz,
                mth,
                args,
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
            );
        } else if (methodType.equals("Interface Static")) {
            //clz = cl.loadClass("com.example.theseus.dynandref.ICollider$-CC");
            //Method mth = clz.getMethod("$default$staticInterfaceTransfer", boolean.class, byte.class, short.class, char.class, int.class, long.class, float.class, double.class, String.class, String[].class);
            clz = cl.loadClass("com.example.theseus.dynandref.ICollider");
            Method mth = clz.getMethod("staticInterfaceTransfer", boolean.class, byte.class, short.class, char.class, int.class, long.class, float.class, double.class, String.class, String[].class);
            invoke(ac,
                false,
                clz,
                mth,
                args,
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
            );
        } else if (methodType.equals("Factory Pattern Interface")) {
            factoryInterface(
                ac, clz, 
                true, 
                (byte)42, 
                (short)666,
                '*',
                0xDEAD_BEEF,
                0xD1AB011C_5EAF00DL,
                0.99f,
                3.1415926535897932384626433d,
                new String[] {"some", "strings"}
            );
        } else if (methodType.equals("Factory Pattern Extend")) {
            factoryExtend(
                ac, clz, 
                true, 
                (byte)42, 
                (short)666,
                '*',
                0xDEAD_BEEF,
                0xD1AB011C_5EAF00DL,
                0.99f,
                3.1415926535897932384626433d,
                new String[] {"some", "strings"}
            );
        } else {
            return;
        };
        } catch (Exception e) {
            Log.e("THESEUS", "error:", e);
        }
    }

    public static void factoryInterface(
        Activity ac, Class clz,
        boolean bool,
        byte by,
        short sh,
        char ch,
        int in,
        long lo,
        float fl,
        double dou,
        String... args
    ) throws Exception {
        ICommonInterface instance = (ICommonInterface)clz.getDeclaredConstructor().newInstance();
        String res = instance.commonInterTransfer(
            bool,
            by,
            sh,
            ch,
            in,
            lo,
            fl,
            dou,
            Utils.source(),
            args
        );
        Utils.sink(ac, res);
    }
    public static void factoryExtend(
        Activity ac, Class clz,
        boolean bool,
        byte by,
        short sh,
        char ch,
        int in,
        long lo,
        float fl,
        double dou,
        String... args
    ) throws Exception {
        PCommonParent instance = (PCommonParent)clz.getDeclaredConstructor().newInstance();
        String res = instance.commonParentTransfer(
            bool,
            by,
            sh,
            ch,
            in,
            lo,
            fl,
            dou,
            Utils.source(),
            args
        );
        Utils.sink(ac, res);
    }

    public static void invoke(
        Activity ac, boolean instanciate, Class clz, Method mth, Object[] args, 
        // Additionnal args to check the register reservation
        boolean bool, 
        byte by, 
        short sh, 
        char ch, 
        int in, 
        long lo, 
        float fl, 
        double dou,
        String str,
        String... strArgs
    ) throws Exception {
        Object instance = null;
        if (instanciate) {
            instance = clz.getDeclaredConstructor().newInstance();
        }
        args[8] = Utils.source();
        String res = (String)mth.invoke(instance, args);
        Utils.sink(ac, res);
        if (!(
            (bool == true) &&
            (by == (byte)42) &&
            (sh == (short)666) &&
            (ch == '*') &&
            (in == 0xDEAD_BEEF) &&
            (lo == 0xD1AB011C_5EAF00DL) &&
            (fl == 0.99f) &&
            (dou == 3.1415926535897932384626433d) &&
            str.equals("") && 
            (strArgs.length == 2) &&
            strArgs[0].equals("some") &&
            strArgs[1].equals("strings")
        )) {
            Log.e("THESEUS", "Main.invoke additionnal arguments don't match");
            Log.e("THESEUS", "bool: " + (bool == true));
            Log.e("THESEUS", "by: " + (by == (byte)42));
            Log.e("THESEUS", "sh: " + (sh == (short)666));
            Log.e("THESEUS", "ch: " + (ch == '*'));
            Log.e("THESEUS", "in: " + (in == 0xDEAD_BEEF));
            Log.e("THESEUS", "lo: " + (lo == 0xD1AB011C_5EAF00DL));
            Log.e("THESEUS", "fl: " + (fl == 0.99f));
            Log.e("THESEUS", "dou: " + (dou == 3.1415926535897932384626433d));
            Log.e("THESEUS", "str: " + str.equals(""));
            Log.e("THESEUS", "strArgs.length: " + (strArgs.length == 2));
            Log.e("THESEUS", "strArgs[0]: " + strArgs[0].equals("some"));
            Log.e("THESEUS", "strArgs[1]: " + strArgs[1].equals("strings"));
        }
    }
}
