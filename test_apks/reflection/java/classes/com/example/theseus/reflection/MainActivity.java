package com.example.theseus.reflection;

import android.app.Activity;
import android.os.Bundle;
import android.util.Log;

import android.widget.RelativeLayout;
import android.widget.LinearLayout;
import android.view.ViewGroup;
import android.view.View;
import android.widget.Button;

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

        RelativeLayout relLayout = new RelativeLayout(this);
        relLayout.generateViewId();

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
        linLayout.addView(b5);

        Button b6 = new Button(this);
        b6.generateViewId();
        linLayout.addView(b6);

        Button b7 = new Button(this);
        b7.generateViewId();
        linLayout.addView(b7);

        Button b8 = new Button(this);
        b8.generateViewId();
        linLayout.addView(b8);

        Button b9 = new Button(this);
        b9.generateViewId();
        linLayout.addView(b9);

        relLayout.addView(linLayout);
        setContentView(relLayout);

        b1.setText("Virtual control");
        b1.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                try {
                    callVirtualMethod();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b2.setText("Virtual rflct");
        b2.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                try {
                    callVirtualMethodReflectCall();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b3.setText("Instanciation rflct");
        b3.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                try {
                    callConstructorVirtualMethodReflectConstr();
                    callVirtualMethodReflectOldConst();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b4.setText("Virtual with scalar control");
        b4.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                try {
                    callVirtualMethodCallAllScalar();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b5.setText("Virtual with scalar rflct");
        b5.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                try {
                    callVirtualMethodReflectCallAllScalar();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b6.setText("Virtual with variable args number control");
        b6.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                try {
                    callVirtualMethodCallVarArg();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b7.setText("Virtual with variable args number rflct");
        b7.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                try {
                    callVirtualMethodReflectVarArg();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b8.setText("Static control");
        b8.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                try {
                    callStaticMethod();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b9.setText("Static rflct");
        b9.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                try {
                    callStaticMethodReflectCall();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });
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

    // A normal virtual method call
    public void callStaticMethod() {
        String data = Utils.source("R7 no reflect");
        String newData = Reflectee.staticTransfer(data);
        Utils.sink(this, newData);
    }

    // A call to a virtual method through reflection
    public void callStaticMethodReflectCall() throws
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException
    {
        String data = Utils.source("R8 reflect");
        ClassLoader cl = MainActivity.class.getClassLoader();
        Class clz = cl.loadClass("com.example.theseus.reflection.Reflectee");
        Method mth = clz.getMethod("staticTransfer", String.class);
        String newData = (String) mth.invoke(null, data);
        Utils.sink(this, newData);
    }

    // TODO: Interface, Inheritance
    // TODO: many argument methods
    // TODO: factory patern
}
