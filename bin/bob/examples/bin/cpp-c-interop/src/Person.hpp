#pragma once

class Person {
    const char* m_name;

public:
    Person(const char* name);

    const char* name() const {
        return m_name;
    }

    void greet();
};
