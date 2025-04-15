package com.example.theseus.dynandref;

public class MainReflectee extends MainPReflectee implements MainIReflectee {
    public static String getReflecteeId() {
        return "MainAPK";
    }
    public String virtTransfer(
        boolean bool, 
        byte by, 
        short sh, 
        char ch, 
        int in, 
        long lo, 
        float fl, 
        double dou,
        String str,
        String... args
    ) {
        String val = "";
        for (String v : args) {
            val += " " + v;
        }
        return getReflecteeId() + ":" + val + "(" + bool + " " + by + " " + sh + " " + ch + " " + in + " " + lo + " " + fl + " " + dou + " " + str + ")";
    }
    public static String staticTransfer(
        boolean bool, 
        byte by, 
        short sh, 
        char ch, 
        int in, 
        long lo, 
        float fl, 
        double dou,
        String str,
        String... args
    ) {
        String val = "";
        for (String v : args) {
            val += " " + v;
        }
        return getReflecteeId() + ":" + val + "(" + bool + " " + by + " " + sh + " " + ch + " " + in + " " + lo + " " + fl + " " + dou + " " + str + ")";
    }

    public String interTransfer(
        boolean bool, 
        byte by, 
        short sh, 
        char ch, 
        int in, 
        long lo, 
        float fl, 
        double dou,
        String str,
        String... args
    ) {
        String val = "";
        for (String v : args) {
            val += " " + v;
        }
        return getReflecteeId() + ":" + val + "(" + bool + " " + by + " " + sh + " " + ch + " " + in + " " + lo + " " + fl + " " + dou + " " + str + ")";
    }
}
