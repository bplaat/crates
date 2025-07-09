package com.example.hello.models;

public class Person {
    private final String name;

    public Person(String name) {
        this.name = name;
    }

    public void greet() {
        System.out.println("Hello " + name + "!");
    }
}
