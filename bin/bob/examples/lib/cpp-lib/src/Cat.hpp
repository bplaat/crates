#pragma once

class Cat {
    const char* m_name;

public:
    Cat(const char* name);

    const char* name() const {
        return m_name;
    }

    void greet();
};
