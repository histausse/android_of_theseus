package com.example.theseus.reflection;

public class ChildReflectee extends Reflectee {
    public static String staticOverridenTransfer(String data) {
        return "Static Overrided Transfer ChildReflectee: " + data;
    }
    public String overridenTransfer(String data) {
        return "Inherited Overrided Transfer ChildReflectee: " + data;
    }
}
