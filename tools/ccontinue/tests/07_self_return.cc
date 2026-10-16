// EXIT: 0
// OUT: 5x8

class Builder {
    @prop @init i32 width;
    @prop @init i32 height;

    Self* with_width(i32 w);
    Self* with_height(i32 h);
    void print();
};
Self* Builder::with_width(i32 w) {
    this->width = w;
    return this;
}
Self* Builder::with_height(i32 h) {
    this->height = h;
    return this;
}
void Builder::print() {
    printf("%dx%d\n", this->width, this->height);
}

int main(void) {
    Builder* b = builder_new(10, 20);
    builder_print(builder_with_height(builder_with_width(b, 5), 8));
    builder_free(b);
    return EXIT_SUCCESS;
}
