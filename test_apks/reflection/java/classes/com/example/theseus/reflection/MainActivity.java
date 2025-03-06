package com.example.theseus.reflection;

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

import com.example.theseus.Utils;


import java.util.Arrays;

public class MainActivity extends Activity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
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

        Button b10 = new Button(this);
        b10.generateViewId();
        linLayout.addView(b10);

        Button b11 = new Button(this);
        b11.generateViewId();
        linLayout.addView(b11);

        Button b12 = new Button(this);
        b12.generateViewId();
        linLayout.addView(b12);

        Button b13 = new Button(this);
        b13.generateViewId();
        linLayout.addView(b13);

        Button b14 = new Button(this);
        b14.generateViewId();
        linLayout.addView(b14);

        Button b15 = new Button(this);
        b15.generateViewId();
        linLayout.addView(b15);

        scrollView.addView(linLayout);
        relLayout.addView(scrollView);
        setContentView(relLayout);

        b1.setText("Virtual control");
        b1.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
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
                v.setBackgroundTintList(buttonColor);
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
                v.setBackgroundTintList(buttonColor);
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
                v.setBackgroundTintList(buttonColor);
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
                v.setBackgroundTintList(buttonColor);
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
                v.setBackgroundTintList(buttonColor);
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
                v.setBackgroundTintList(buttonColor);
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
                v.setBackgroundTintList(buttonColor);
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
                v.setBackgroundTintList(buttonColor);
                try {
                    callStaticMethodReflectCall();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b10.setText("Extends control");
        b10.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                try {
                    callVirtualInherited();
                    callVirtualOverriden();
                    callStaticInherited();
                    callStaticOverriden();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b11.setText("Extends rflct");
        b11.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                try {
                    callVirtualInheritedReflect();
                    callVirtualOverridenReflect();
                    callStaticInheritedReflect();
                    callStaticOverridenReflect();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b12.setText("Interface control");
        b12.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                try {
                    callVirtualInheritedInterface();
                    callVirtualOverridenInterface();
                    callStaticInheritedInterface();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b13.setText("Interface rflct");
        b13.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                try {
                    callVirtualInheritedInterfaceReflect();
                    callVirtualOverridenInterfaceReflect();
                    callStaticInheritedInterfaceReflect();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });

        b14.setText("Interface Factory control");
        b14.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                try {
                    callVirtualInheritedInterfaceFactory();
                    callVirtualOverridenInterfaceFactory();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });
        b15.setText("Interface Factory rflct");
        b15.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                v.setBackgroundTintList(buttonColor);
                try {
                    callVirtualInheritedInterfaceFactoryReflect();
                    callVirtualOverridenInterfaceFactoryReflect();
                } catch(Exception e) {
                    Log.e("THESEUS", "Error: ", e);
                }
            }
        });
    }

    // A normal virtual method call
    public void callVirtualMethod() {
        String data = Utils.source("no reflect virt call");
        Reflectee r = new Reflectee("T1");
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
        Reflectee r = new Reflectee("R1");
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
        Reflectee r = new Reflectee("T2");
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
        Reflectee r = new Reflectee("R2");
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
        Reflectee r = new Reflectee("T3");
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
        Reflectee r = new Reflectee("R3");
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
        String data = Utils.source("T4 control static");
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
        String data = Utils.source("R4 reflect static");
        ClassLoader cl = MainActivity.class.getClassLoader();
        Class clz = cl.loadClass("com.example.theseus.reflection.Reflectee");
        Method mth = clz.getMethod("staticTransfer", String.class);
        String newData = (String) mth.invoke(null, data);
        Utils.sink(this, newData);
    }

    public void callVirtualInherited() {
        String data = Utils.source("T5 control extend virt");
        String newData = new ChildReflectee().inheritedTransfer(data);
        Utils.sink(this, newData);
    }
    public void callVirtualOverriden() {
        String data = Utils.source("T6 control extend virt override");
        String newData = new ChildReflectee().overridenTransfer(data);
        Utils.sink(this, newData);
    }
    public void callStaticInherited() {
        String data = Utils.source("T7 control extend static");
        String newData = ChildReflectee.staticInheritedTransfer(data);
        Utils.sink(this, newData);
    }
    public void callStaticOverriden()  {
        String data = Utils.source("T8 control extend static override");
        String newData = ChildReflectee.staticOverridenTransfer(data);
        Utils.sink(this, newData);
    }

    public void callVirtualInheritedReflect() throws
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException
    {
        String data = Utils.source("R5 reflect extend virt");
        ChildReflectee obj = new ChildReflectee();
        ClassLoader cl = MainActivity.class.getClassLoader();
        Class clz = cl.loadClass("com.example.theseus.reflection.ChildReflectee");
        Method mth = clz.getMethod("inheritedTransfer", String.class);
        String newData = (String) mth.invoke(obj, data);
        Utils.testIsChildReflectee(this, obj);
        Utils.sink(this, newData);
    }

    public void callVirtualOverridenReflect() throws
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException
    {
        String data = Utils.source("R6 reflect extend virt override");
        ChildReflectee obj = new ChildReflectee();
        ClassLoader cl = MainActivity.class.getClassLoader();
        Class clz = cl.loadClass("com.example.theseus.reflection.ChildReflectee");
        Method mth = clz.getMethod("overridenTransfer", String.class);
        String newData = (String) mth.invoke(obj, data);
        Utils.testIsChildReflectee(this, obj);
        Utils.sink(this, newData);
    }

    public void callStaticInheritedReflect() throws
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException
    {
        String data = Utils.source("R7 reflect extend static");
        ClassLoader cl = MainActivity.class.getClassLoader();
        Class clz = cl.loadClass("com.example.theseus.reflection.ChildReflectee");
        Method mth = clz.getMethod("staticInheritedTransfer", String.class);
        String newData = (String) mth.invoke(null, data);
        Utils.sink(this, newData);
    }
    public void callStaticOverridenReflect() throws
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException
    {
        String data = Utils.source("R8 reflect extend static override");
        ClassLoader cl = MainActivity.class.getClassLoader();
        Class clz = cl.loadClass("com.example.theseus.reflection.ChildReflectee");
        Method mth = clz.getMethod("staticOverridenTransfer", String.class);
        String newData = (String) mth.invoke(null, data);
        Utils.sink(this, newData);
    }

    public void callVirtualInheritedInterface() {
        String data = Utils.source("T9 control virtual interface default");
        String newData = new ChildReflectee().defaultInterfaceTransfer(data);
        Utils.sink(this, newData);
    }
    public void callVirtualOverridenInterface() {
        String data = Utils.source("T10 control virtual interface overriden");
        String newData = new ChildReflectee().overridenInterfaceTransfer(data);
        Utils.sink(this, newData);
    }
    public void callStaticInheritedInterface() {
        String data = Utils.source("T11 control static interface");
        String newData = IReflectee.staticDefaultInterfaceTransfer(data);
        Utils.sink(this, newData);
    }
    public void callVirtualInheritedInterfaceReflect() throws 
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException
    {
        String data = Utils.source("R9 reflect virtual interface default");
        ChildReflectee obj = new ChildReflectee();
        ClassLoader cl = MainActivity.class.getClassLoader();
        Class clz = cl.loadClass("com.example.theseus.reflection.ChildReflectee");
        Method mth = clz.getMethod("defaultInterfaceTransfer", String.class);
        String newData = (String) mth.invoke(obj, data);
        Utils.testIsChildReflectee(this, obj);
        Utils.sink(this, newData);
    }
    public void callVirtualOverridenInterfaceReflect() throws 
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException
    {
        String data = Utils.source("R10 reflect virtual interface overriden");
        ChildReflectee obj = new ChildReflectee();
        ClassLoader cl = MainActivity.class.getClassLoader();
        Class clz = cl.loadClass("com.example.theseus.reflection.ChildReflectee");
        Method mth = clz.getMethod("overridenInterfaceTransfer", String.class);
        String newData = (String) mth.invoke(obj, data);
        Utils.testIsChildReflectee(this, obj);
        Utils.sink(this, newData);
    }

    /* Android is broken (what a surprise...), cannot get the Method representation 
     * of a static method implemented in an Interface.
     */
    public void callStaticInheritedInterfaceReflect() throws 
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException
    {
        Utils.popup(this, "DEBUG", "R11 not implemented due to Android beeing Android");
        /*
        String data = Utils.source("R11 reflect static interface");
        ClassLoader cl = MainActivity.class.getClassLoader();
        Class clz = cl.loadClass("com.example.theseus.reflection.IReflectee");
        Utils.popup(this, "DEBUG", Arrays.deepToString(clz.getDeclaredMethods()));
        Utils.popup(this, "DEBUG", ""+clz.getMethods().length);
        //Method mth = clz.getDeclaredMethod("staticDefaultInterfaceTransfer", String.class);
        Method mth = clz.getMethod("staticDefaultInterfaceTransfer", String.class);
        String newData = (String) mth.invoke(null, data);
        Utils.sink(this, newData);
        */
    }

    public void callVirtualInheritedInterfaceFactory() {
        String data = Utils.source("T12 control virtual interface default factory");
        IReflectee obj = (IReflectee) (new ChildReflectee());
        String newData = obj.defaultInterfaceTransfer(data);
        Utils.sink(this, newData);
    }
    public void callVirtualOverridenInterfaceFactory() {
        String data = Utils.source("T13 control virtual interface overriden factory");
        IReflectee obj = (IReflectee) (new ChildReflectee());
        String newData = obj.overridenInterfaceTransfer(data);
        Utils.sink(this, newData);
    }
    public void callVirtualInheritedInterfaceFactoryReflect() throws 
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException,
        InstantiationException
    {
        String data = Utils.source("R12 reflect virtual interface default factory");
        IReflectee obj = (IReflectee) ChildReflectee.class.getDeclaredConstructor().newInstance();
        String newData = obj.defaultInterfaceTransfer(data);
        Utils.sink(this, newData);
    }
    public void callVirtualOverridenInterfaceFactoryReflect() throws 
        ClassNotFoundException,
        NoSuchMethodException,
        IllegalAccessException,
        InvocationTargetException,
        InstantiationException
    {
        String data = Utils.source("R13 reflect virtual interface overriden factory");
        IReflectee obj = (IReflectee) ChildReflectee.class.getDeclaredConstructor().newInstance();
        String newData = obj.overridenInterfaceTransfer(data);
        Utils.sink(this, newData);
    }

    // TODO: many argument methods
    // TODO: call from static method
    // TODO: call different methods with the same invoke
    // TODO: several invoke in same method
}

