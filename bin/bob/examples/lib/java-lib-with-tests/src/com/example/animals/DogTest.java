package com.example.animals;

import org.junit.Test;
import static org.junit.Assert.*;

public class DogTest {
    @Test
    public void testDogNew() {
        var dog = new Dog("Woof");
        assertEquals(dog.name(), "Woof");
    }
}
