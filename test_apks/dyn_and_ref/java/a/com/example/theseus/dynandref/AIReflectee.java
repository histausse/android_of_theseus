package com.example.theseus.dynandref;

public interface AIReflectee {
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
    );

    default public String staticInterfaceTransfer(
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
        return "A:" + val + "(" + bool + " " + by + " " + sh + " " + ch + " " + in + " " + lo + " " + fl + " " + dou + " " + str + ")";
    }
}
