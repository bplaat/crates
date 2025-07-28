package com.example.hello

class Person(private val name: String, private val age: Int) {
    fun greet() {
        println("Hello, my name is $name and I am $age years old.")
    }
}
