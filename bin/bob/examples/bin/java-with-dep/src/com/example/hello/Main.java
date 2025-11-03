package com.example.hello;

import org.json.JSONObject;

public class Main {
    public static void main(String[] args) {
        var json = new JSONObject();
        json.put("name", "Alice");
        json.put("age", 30);
        json.put("isStudent", false);
        System.out.println(json.toString());
    }
}
