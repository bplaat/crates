package com.example.animals;

public class Dog {
    private final String name;

    public Dog(String name) {
        this.name = name;
    }

    public void greet() {
        System.out.println("Woof said " + name + "!");
    }
}
