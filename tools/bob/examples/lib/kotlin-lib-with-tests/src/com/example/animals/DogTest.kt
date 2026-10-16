package com.example.animals

import org.junit.Test
import org.junit.Assert.*

public class DogTest {
    @Test
    fun testDogNew() {
        val dog = Dog("Woof")
        assertEquals(dog.name(), "Woof")
    }
}
