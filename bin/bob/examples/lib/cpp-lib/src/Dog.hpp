#pragma once

class Dog {
    const char* m_name;
public:
    Dog(const char* name);

    const char *name() const { return m_name; }

    void greet();
};
