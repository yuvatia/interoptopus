#ifndef interoptopus_generated
#define interoptopus_generated

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>

typedef struct VEC2
{
    float x;
    float y;
} VEC2;


VEC2 my_function(VEC2 INPUT);

typedef struct hello_world_api_t
{
    VEC2 (*my_function)(VEC2);
} hello_world_api_t;

#if defined(_WIN32)
#include <windows.h>
static int hello_world_load(const char* path, hello_world_api_t* api)
{
    HMODULE lib = LoadLibraryA(path);
    if (!lib) return -1;
    api->my_function = (VEC2 (*)(VEC2))(void*)GetProcAddress(lib, "my_function");
    if (!api->my_function) return -1;
    return 0;
}
#else
#include <dlfcn.h>
#include <string.h>
static int hello_world_load(const char* path, hello_world_api_t* api)
{
    void* lib = dlopen(path, RTLD_NOW);
    if (!lib) return -1;
    void* sym;
    sym = dlsym(lib, "my_function");
    if (!sym) return -1;
    memcpy(&api->my_function, &sym, sizeof(sym));
    return 0;
}
#endif

#ifdef HELLO_WORLD_STATIC
static int hello_world_load_static(hello_world_api_t* api)
{
    api->my_function = my_function;
    return 0;
}
#endif /* HELLO_WORLD_STATIC */

#ifdef __cplusplus
}
#endif

#endif /* interoptopus_generated */
