package com.example.theseus.reflection;


public interface IReflectee {
    public static String staticDefaultInterfaceTransfer(String data) {
        return "Static Interface Transfer IReflectee: " + data;
    }

    default public String defaultInterfaceTransfer(String data) {
        return "Default Interface Transfer IReflectee: " + data;
    }

    public String overridenInterfaceTransfer(String data);
}
