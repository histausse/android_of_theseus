package com.example.theseus.tests;

import android.app.Activity;
import android.os.Bundle;
import android.util.Log;

import java.lang.ClassLoader;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Constructor;
import java.lang.ClassNotFoundException;
import java.util.Arrays;

import android.util.Log;

import java.util.Random;

import com.example.theseus.Utils;

public class MainActivity extends Activity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {

        boolean t = (new Random().nextInt(1000)) < 500;
        if (t) {
            Log.i("THESEUS", "OK");
        } else { 
            Log.i("THESEUS", "NOP");
        }
        boolean t0 = (new Random().nextInt(1000)) < 500;
        boolean t1 = (new Random().nextInt(1000)) < 500;
        boolean t2 = (new Random().nextInt(1000)) < 500;
        boolean t3 = (new Random().nextInt(1000)) < 500;
        boolean t4 = (new Random().nextInt(1000)) < 500;
        boolean t5 = (new Random().nextInt(1000)) < 500;
        boolean t6 = (new Random().nextInt(1000)) < 500;
        boolean t7 = (new Random().nextInt(1000)) < 500;
        boolean t8 = (new Random().nextInt(1000)) < 500;
        boolean t9 = (new Random().nextInt(1000)) < 500;
        boolean t10 = (new Random().nextInt(1000)) < 500;
        boolean t11 = (new Random().nextInt(1000)) < 500;
        boolean t12 = (new Random().nextInt(1000)) < 500;
        boolean t13 = (new Random().nextInt(1000)) < 500;
        boolean t14 = (new Random().nextInt(1000)) < 500;
        boolean t15 = (new Random().nextInt(1000)) < 500;
        Log.i("THESEUS", "" + t0+t1+t2+t3+t4+t5+t6+t7+t8+t9+t10+t11+t12+t13+t14+t15);

        super.onCreate(savedInstanceState);
        try {
            //callVirtualMethod();
            callVirtualMethodReflectCall();
            //callConstructorVirtualMethodReflectConstr();
            //callVirtualMethodReflectOldConst();
        } catch(Exception e) {
            Log.e("THESEUS", "Error: ", e);
        }
    }

    // A normal virtual method call
    public void callVirtualMethod() {
        String data = Utils.source("no reflect virt call");
        Reflectee r = new Reflectee("R1");
        String newData = r.transfer(data);
        Utils.sink(this, newData);
    }

    // A call to a virtual method through reflection
    public void callVirtualMethodReflectCall() throws
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException
    {
        String data = Utils.source("reflect virt call");
        Reflectee r = new Reflectee("R2");
        ClassLoader cl = MainActivity.class.getClassLoader();
        Class clz = cl.loadClass("com.example.theseus.tests.Reflectee");
        Method mth = clz.getMethod("transfer", String.class);
        /*
        String name = mth.getName();
        Class[] params = mth.getParameterTypes();
        Class ret = mth.getReturnType();
        Class dec = mth.getDeclaringClass();
        Log.e("[TEST]", "---------------------------------");
        Log.e("[TEST]", name);
        Log.e("[TEST]", params.toString());
        Log.e("[TEST]", ret.toString());
        Log.e("[TEST]", dec.toString());
        Log.e("[TEST]", "---------------------------------");
        */
        Class[] params = mth.getParameterTypes();
        if (
            mth.getName().equals("transfer") && 
            mth.getReturnType() == String.class && 
            mth.getDeclaringClass() == Reflectee.class &&
            params.length == 1 && 
            params[0] == String.class
        ) { 
            Log.e("[TEST]", "OK");
        }
        String newData = (String) mth.invoke(r, data);
        Utils.sink(this, newData);
    }

    // A call to a virtual method through reflection using an object instanciated 
    // through reflection. The sensitive data is passed to the constructor.
    public void callConstructorVirtualMethodReflectConstr() throws 
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException,
        InstantiationException
    {
        String data = Utils.source("no reflect constr");
        ClassLoader cl = MainActivity.class.getClassLoader();
        Class clz = cl.loadClass("com.example.theseus.reflection.Reflectee");
        Constructor cst = clz.getDeclaredConstructor(String.class);
        Object r;
        Class[] args = cst.getParameterTypes();
        if (args.length == 1 && args[0] == String.class && cst.getDeclaringClass() == Reflectee.class) {
            r = new Reflectee(data);
        } else {
            r = cst.newInstance(data);
        }
        Method mth = clz.getMethod("transfer", String.class);
        String newData = (String) mth.invoke(r, "");
        Utils.sink(this, newData);
    }

    // A call to a virtual method through reflection using an object instanciated 
    // through reflection using a deprecated method.
    public void callVirtualMethodReflectOldConst() throws
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException,
        InstantiationException
    {
        String data = Utils.source("no reflect constr");
        ClassLoader cl = MainActivity.class.getClassLoader();
        Class clz = cl.loadClass("com.example.theseus.reflection.Reflectee");
        Object r;
        if (
            clz == Reflectee.class
        ) {
            r = new Reflectee();
        } else {
            r = clz.newInstance();
        }
        Method mth = clz.getMethod("transfer", String.class);
        String newData = (String) mth.invoke(r, data);
        Utils.sink(this, newData);
    }

    // TODO: many argument methods
    // TODO: static
    // TODO: factory patern 
}
