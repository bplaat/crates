// PlaatCraft - Log Header

#ifndef LOG_H
#define LOG_H

#include <pthread.h>

extern pthread_mutex_t log_lock;

void log_init();

void log_debug(char* format, ...);

void log_info(char* format, ...);

void log_warning(char* format, ...);

void log_error(char* format, ...);

#endif
