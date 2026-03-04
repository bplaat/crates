// EXIT: 0
// OUT: area=25
// OUT: color=red
// OUT: area=25

class IShape {
    i32 area();
};

class IColoredShape : IShape {
    char* color();
};

class Square : IColoredShape {
    @init i32 side;
    @init char* _color;
    virtual i32 area();
    virtual char* color();
};
i32 Square::area() {
    return this->side * this->side;
}
char* Square::color() {
    return this->_color;
}

int main(void) {
    Square* s = square_new(5, "red");

    IColoredShape ics = cast<IColoredShape>(s);
    printf("area=%d\n", i_colored_shape_area(ics));
    printf("color=%s\n", i_colored_shape_color(ics));

    IShape ishape = cast<IShape>(s);
    printf("area=%d\n", i_shape_area(ishape));

    square_free(s);
    return EXIT_SUCCESS;
}
