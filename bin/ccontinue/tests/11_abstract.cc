// EXIT: 0
// OUT: Rectangle: 12
// OUT: Square: 9
// OUT: Rectangle: 12

class Shape {
    virtual void print_area() = 0;
};

class Rectangle : Shape {
    @init i32 w;
    @init i32 h;
    virtual void print_area();
};
void Rectangle::print_area() {
    printf("Rectangle: %d\n", this->w * this->h);
}

class Square : Shape {
    @init i32 side;
    virtual void print_area();
};
void Square::print_area() {
    printf("Square: %d\n", this->side * this->side);
}

int main(void) {
    Rectangle* r = rectangle_new(3, 4);
    Square* s = square_new(3);
    shape_print_area(r);
    shape_print_area(s);
    rectangle_print_area(r);
    rectangle_free(r);
    square_free(s);
    return EXIT_SUCCESS;
}
