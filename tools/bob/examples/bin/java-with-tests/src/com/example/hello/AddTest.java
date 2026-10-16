package com.example.hello;

import static org.junit.Assert.*;

import org.junit.Test;

public class AddTest {
    @Test
    public void testAddPositiveNumbers() {
        assertEquals(5, Add.add(2, 3));
    }

    @Test
    public void testAddNegativeNumbers() {
        assertEquals(-5, Add.add(-2, -3));
    }

    @Test
    public void testAddZero() {
        assertEquals(2, Add.add(2, 0));
        assertEquals(0, Add.add(0, 0));
    }

    @Test
    public void testAddPositiveAndNegative() {
        assertEquals(1, Add.add(3, -2));
        assertEquals(-1, Add.add(-3, 2));
    }
}
