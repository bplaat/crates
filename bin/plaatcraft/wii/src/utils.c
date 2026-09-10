#include "utils.h"
#include <math.h>

double degrees(double radians) {
    return radians * (180 / M_PI);
}

double radians(double degrees) {
    return degrees * (M_PI / 180);
}
