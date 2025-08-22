package com.example.animals

class Dog(private val name: String) {
    fun name(): String {
        return name
    }

    fun greet() {
        println("Woof said $name!")
    }
}
