// EXIT: 0
// OUT: x=3
// OUT: y=7

class Point {
    @get @init i32 x;
    @get @init i32 y;
};

int main(void) {
    Point* p = point_new(3, 7);
    printf("x=%d\n", point_get_x(p));
    printf("y=%d\n", point_get_y(p));
    point_free(p);
    return EXIT_SUCCESS;
}
