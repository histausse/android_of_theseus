package com.example.theseus.reflection;


public class Reflectee {

    String name;

    public Reflectee() {
        this.name = "";
    }

    public Reflectee(String name) {
        this.name = "[" + name + "] ";
    }

    public String transfer(String data) {
        return name + data;
    }

    public String transfer(
        boolean bool, 
        byte by, 
        short sh, 
        char ch, 
        int in, 
        long lo, 
        float fl, 
        double dou,
        String str
    ) {
        return name + " " + bool + " " + by + " " + sh + " " + ch + " " + in + " " + lo + " " + fl + " " + dou + " " + str;
    }
    public String transfer(
        String arg1,
        String... args
    ) {
        String val = name + " " + arg1;
        for (String v : args) {
            val += " " + v;
        }
        return val;
    }
}
