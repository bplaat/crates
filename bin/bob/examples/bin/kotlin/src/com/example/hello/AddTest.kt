package com.example.hello

import org.junit.Test
import org.junit.Assert.*

class AddTest {
    @Test
    fun testAddPositiveNumbers() {
        assertEquals(5, Add.add(2, 3))
    }

    @Test
    fun testAddNegativeNumbers() {
        assertEquals(-5, Add.add(-2, -3))
    }

    @Test
    fun testAddZero() {
        assertEquals(2, Add.add(2, 0))
        assertEquals(0, Add.add(0, 0))
    }

    @Test
    fun testAddPositiveAndNegative() {
        assertEquals(1, Add.add(3, -2))
        assertEquals(-1, Add.add(-3, 2))
    }
}
