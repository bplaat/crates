// EXIT: 0
// OUT: data=hello

class Buffer {
    @init @deinit char* data;
};

int main(void) {
    Buffer* buf = buffer_new(strdup("hello"));
    printf("data=%s\n", buf->data);
    buffer_free(buf);
    return EXIT_SUCCESS;
}
