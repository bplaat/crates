package com.example.hello

import java.util.ArrayList
import com.example.animals.Cat
import com.example.animals.Dog

fun main(args: Array<String>) {
    val people = ArrayList<Person>()
    people.add(Person("Alice", 25))
    people.add(Person("Bob", 31))
    for (person in people) {
        person.greet()
    }

    val cat = Cat("Mittens")
    cat.greet()
    val dog = Dog("Rover")
    dog.greet()
    val house = House("My House")
    house.greet()
}
