#include <List.hh>

// List
void List::init() {
    Object::init();
    this->items = malloc(sizeof(Object*) * this->capacity);
}

void List::deinit() {
    for (usize i = 0; i < this->size; i++)
        object_free(this->items[i]);
    free(this->items);
    Object::deinit();
}

Object* List::get(usize index) {
    return this->items[index];
}

void List::set(usize index, Object* item) {
    if (index > this->capacity) {
        while (index > this->capacity)
            this->capacity <<= 1;
        this->items = realloc(this->items, sizeof(Object*) * this->capacity);
    }
    while (this->size <= index)
        this->items[this->size++] = NULL;
    this->items[index] = item;
}

void List::add(Object* item) {
    if (this->size == this->capacity) {
        this->capacity <<= 1;
        this->items = realloc(this->items, sizeof(Object*) * this->capacity);
    }
    this->items[this->size++] = item;
}

void List::insert(usize index, Object* item) {
    if (this->size == this->capacity) {
        this->capacity <<= 1;
        this->items = realloc(this->items, sizeof(Object*) * this->capacity);
    }
    for (usize i = this->size - 1; i >= index; i--)
        this->items[i + 1] = this->items[i];
    this->size++;
    this->items[index] = item;
}

void List::remove(usize index) {
    object_free(this->items[index]);
    for (usize i = index; i < this->size - 1; i++)
        this->items[i] = this->items[i + 1];
    this->size--;
}

IIterator List::iterator() {
    return cast<IIterator>(list_iterator_new(this));
}

// ListIterator
bool ListIterator::has_next() {
    return this->index < this->list->size;
}

Object* ListIterator::next() {
    return this->list->items[this->index++];
}
