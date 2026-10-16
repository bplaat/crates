// No include guard, the standard wants assert to follow NDEBUG at every include
#include <stdlib.h>

#undef assert
#ifdef NDEBUG
#define assert(x) ((void)0)
#else
#define assert(x) ((x) ? (void)0 : abort())
#endif
