package com.example.theseus.tests;


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
}
