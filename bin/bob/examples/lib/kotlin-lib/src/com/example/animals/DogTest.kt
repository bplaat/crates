package com.example.animals

import org.junit.Test
import org.junit.Assert.*

public class DogTest {
    @Test
    fun testDogNew() {
        var dog = Dog("Woof");
        assertEquals(dog.name(), "Woof");
    }
}
