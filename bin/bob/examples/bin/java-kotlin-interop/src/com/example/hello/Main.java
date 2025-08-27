package com.example.hello;

import com.example.animals.Cat;
import com.example.animals.Dog;
import com.example.hello.models.Person;

public class Main {
    public static void main(String[] args) {
        var person = new Person("Bastiaan");
        person.greet();
        var cat = new Cat("Mittens");
        cat.greet();
        var dog = new Dog("Rover");
        dog.greet();
    }
}
