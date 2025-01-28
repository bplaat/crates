package com.example.hello;

import com.example.hello.models.Person;

public class Main {
    public static void main(String[] args) {
        var person = new Person("Bastiaan");
        person.greet();
    }
}
