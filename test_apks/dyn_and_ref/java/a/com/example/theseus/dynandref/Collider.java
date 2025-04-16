package com.example.theseus.dynandref;

public class Collider extends PCollider implements ICollider, ICommonInterface {
    public static String getColliderId() {
        return "A";
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
        return getColliderId() + ":" + val + "(" + bool + " " + by + " " + sh + " " + ch + " " + in + " " + lo + " " + fl + " " + dou + " " + str + ")";
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
        return getColliderId() + ":" + val + "(" + bool + " " + by + " " + sh + " " + ch + " " + in + " " + lo + " " + fl + " " + dou + " " + str + ")";
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
        return getColliderId() + ":" + val + "(" + bool + " " + by + " " + sh + " " + ch + " " + in + " " + lo + " " + fl + " " + dou + " " + str + ")";
    }

    public String commonInterTransfer(
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
        return getColliderId() + ":" + val + "(" + bool + " " + by + " " + sh + " " + ch + " " + in + " " + lo + " " + fl + " " + dou + " " + str + ")";
    }
}
