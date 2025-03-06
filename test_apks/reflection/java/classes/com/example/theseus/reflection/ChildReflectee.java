package com.example.theseus.reflection;

public class ChildReflectee extends Reflectee implements IReflectee {
    public static String staticOverridenTransfer(String data) {
        return "Static Overrided Transfer ChildReflectee: " + data;
    }
    public String overridenTransfer(String data) {
        return "Inherited Overrided Transfer ChildReflectee: " + data;
    }

    public static String staticOverridenInterfaceTransfer(String data) {
        return "Static Overriden Interface Transfer ChildReflectee: " + data;
    }
    public String overridenInterfaceTransfer(String data) {
        return "Overriden Interface Transfer ChildReflectee: " + data;
    }
}
