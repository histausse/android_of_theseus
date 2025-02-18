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
            callVirtualMethodCallAllScalar();
            callVirtualMethodReflectCallAllScalar();
            callVirtualMethodCallVarArg();
            callVirtualMethodReflectVarArg();
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
        Utils.testIsReflectee(this, r);
        Utils.sink(this, newData);
    }

    // A call to a virtual method with all scalar types.
    public void callVirtualMethodCallAllScalar()
    {
        String data = Utils.source("no reflect virt call all scalars");
        Reflectee r = new Reflectee("R3");
        String newData = r.transfer(true, (byte)42, (short)666, '*', 0xDEAD_BEEF, 0xD1AB011C_5EAF00DL, 0.99f, 3.1415926535897932384626433d, data);
        Utils.testIsReflectee(this, r);
        Utils.sink(this, newData);
    }

    // A call to a virtual method through reflection with all scalar types.
    public void callVirtualMethodReflectCallAllScalar() throws
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException
    {
        String data = Utils.source("reflect virt call all scalars");
        Reflectee r = new Reflectee("R4");
        ClassLoader cl = MainActivity.class.getClassLoader();
        Class clz = cl.loadClass("com.example.theseus.reflection.Reflectee");
        Method mth = clz.getMethod("transfer", boolean.class, byte.class, short.class, char.class, int.class, long.class, float.class, double.class, String.class);
        String newData = (String) mth.invoke(r, true, (byte)42, (short)666, '*', 0xDEAD_BEEF, 0xD1AB011C_5EAF00DL, 0.99f, 3.1415926535897932384626433d, data);
        Utils.testIsReflectee(this, r);
        Utils.sink(this, newData);
    }

    // A call to a virtual method with variable number of arg.
    public void callVirtualMethodCallVarArg()
    {
        String data = Utils.source("no reflect virt call variable arg numb");
        Reflectee r = new Reflectee("R5");
        String newData = r.transfer("aa", "bb", data, "cc");
        Utils.testIsReflectee(this, r);
        Utils.sink(this, newData);
    }

    // A call to a virtual method through reflection with variable number of arg.
    public void callVirtualMethodReflectVarArg() throws
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException
    {
        String data = Utils.source("reflect virt call variable arg numb");
        Reflectee r = new Reflectee("R6");
        ClassLoader cl = MainActivity.class.getClassLoader();
        Class clz = cl.loadClass("com.example.theseus.reflection.Reflectee");
        Method mth = clz.getMethod("transfer", String.class, String[].class);
        String newData = (String) mth.invoke(r, "aa", new String[] {"bb", data, "cc"});
        Utils.testIsReflectee(this, r);
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
        Utils.testIsObject(this, r);
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
        Utils.testIsObject(this, r);
        Utils.sink(this, newData);
    }

    // TODO: many argument methods
    // TODO: static
    // TODO: factory patern 
}
