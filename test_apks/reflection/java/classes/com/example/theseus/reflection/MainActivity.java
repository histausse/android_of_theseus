package com.example.theseus.reflection;

import android.app.Activity;
import android.os.Bundle;
import android.util.Log;

import java.lang.ClassLoader;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Constructor;
import java.lang.ClassNotFoundException;

import com.example.theseus.Utils;

public class MainActivity extends Activity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        try {
            callVirtualMethod();
            callVirtualMethodReflectCall();
            callConstructorVirtualMethodReflectConstr();
            callVirtualMethodReflectOldConst();
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
        Class clz = cl.loadClass("com.example.theseus.reflection.Reflectee");
        Method mth = clz.getMethod("transfer", String.class);
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
        Object r = cst.newInstance(data);
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
        Object r = clz.newInstance();
        Method mth = clz.getMethod("transfer", String.class);
        String newData = (String) mth.invoke(r, data);
        Utils.sink(this, newData);
    }

    // TODO: many argument methods
    // TODO: static
    // TODO: factory patern 
}
