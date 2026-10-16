// EXIT: 0
// OUT: name=Bastiaan
// OUT: independent=true

class Label {
    @get @init(strdup) @deinit char* name;
};

int main(void) {
    char stack[] = "Bastiaan";
    Label* l = label_new(stack);
    stack[0] = 'X';
    printf("name=%s\n", label_get_name(l));
    printf("independent=%s\n", label_get_name(l)[0] == 'B' ? "true" : "false");
    label_free(l);
    return EXIT_SUCCESS;
}
