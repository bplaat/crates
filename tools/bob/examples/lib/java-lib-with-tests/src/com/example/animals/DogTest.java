package com.example.animals;

import static org.junit.Assert.*;

import org.junit.Test;

public class DogTest {
    @Test
    public void testDogNew() {
        var dog = new Dog("Woof");
        assertEquals(dog.name(), "Woof");
    }
}
