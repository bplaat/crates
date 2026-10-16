package com.example.hello;

public class House {
    private final String name;

    public House(String name) {
        this.name = name;
    }

    public void greet() {
        System.out.println("House name is " + name + "!");
    }
}
