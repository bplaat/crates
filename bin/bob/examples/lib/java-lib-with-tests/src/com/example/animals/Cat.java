package com.example.animals;

public class Cat {
    private final String name;

    public Cat(String name) {
        this.name = name;
    }

    public void greet() {
        System.out.println("Miauw said " + name + "!");
    }
}
