// EXIT: 0
// OUT: speed=120
// OUT: Car Tesla speed=120

class Vehicle {
    @init i32 speed;
    void print_speed();
};
void Vehicle::print_speed() {
    printf("speed=%d\n", this->speed);
}

class Car : Vehicle {
    @init char* model;
    void print_info();
};
void Car::print_info() {
    printf("Car %s speed=%d\n", this->model, this->speed);
}

int main(void) {
    Car* car = car_new(120, "Tesla");
    vehicle_print_speed(car);
    car_print_info(car);
    car_free(car);
    return EXIT_SUCCESS;
}
