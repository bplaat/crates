// EXIT: 0
// OUT: visible=true
// OUT: count=0
// OUT: visible=false
// OUT: count=5

class Widget {
    @prop bool visible = true;
    @prop i32 count = 0;
};

int main(void) {
    Widget* w = widget_new();
    printf("visible=%s\n", widget_get_visible(w) ? "true" : "false");
    printf("count=%d\n", widget_get_count(w));
    widget_set_visible(w, false);
    widget_set_count(w, 5);
    printf("visible=%s\n", widget_get_visible(w) ? "true" : "false");
    printf("count=%d\n", widget_get_count(w));
    widget_free(w);
    return EXIT_SUCCESS;
}
