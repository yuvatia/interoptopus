#ifndef interoptopus_generated
#define interoptopus_generated

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>

typedef struct STRING
{
    uint8_t* ptr;
    uint64_t len;
    uint64_t capacity;
} STRING;

typedef enum OPTIONSTRING_TAG
{
    OPTIONSTRING_SOME = 0,
    OPTIONSTRING_NONE = 1,
} OPTIONSTRING_TAG;

typedef struct OPTIONSTRING
{
    OPTIONSTRING_TAG tag;
    union
    {
        STRING some;
    };
} OPTIONSTRING;

typedef struct VEC2
{
    float x;
    float y;
} VEC2;

typedef enum SHAPE_TAG
{
    SHAPE_CIRCLE = 0,
    SHAPE_RECTANGLE = 1,
} SHAPE_TAG;

typedef struct SHAPE
{
    SHAPE_TAG tag;
    union
    {
        float circle;
        VEC2 rectangle;
    };
} SHAPE;

typedef enum OPTIONVEC2_TAG
{
    OPTIONVEC2_SOME = 0,
    OPTIONVEC2_NONE = 1,
} OPTIONVEC2_TAG;

typedef struct OPTIONVEC2
{
    OPTIONVEC2_TAG tag;
    union
    {
        VEC2 some;
    };
} OPTIONVEC2;

typedef struct DRAWCOMMAND
{
    SHAPE shape;
    VEC2 position;
} DRAWCOMMAND;

typedef struct SLICEDRAWCOMMAND
{
    const DRAWCOMMAND* data;
    uint64_t len;
} SLICEDRAWCOMMAND;

typedef struct KITCHENSINK
{
    uint64_t id;
    bool enabled;
    double ratio;
    STRING label;
    SHAPE shape;
    OPTIONVEC2 position;
    SLICEDRAWCOMMAND tags;
    OPTIONSTRING name;
} KITCHENSINK;

typedef void (*KITCHENSINKCALLBACK_fn)(const KITCHENSINK*, const void*);

typedef struct KITCHENSINKCALLBACK
{
    KITCHENSINKCALLBACK_fn callback;
    const void* data;
    void (*destructor)(const void*);
} KITCHENSINKCALLBACK;

typedef float (*OPTIONCALLBACK_fn)(OPTIONVEC2, const void*);

typedef struct OPTIONCALLBACK
{
    OPTIONCALLBACK_fn callback;
    const void* data;
    void (*destructor)(const void*);
} OPTIONCALLBACK;

typedef struct VECDRAWCOMMAND
{
    DRAWCOMMAND* ptr;
    uint64_t len;
    uint64_t capacity;
} VECDRAWCOMMAND;

typedef float (*VECCALLBACK_fn)(VECDRAWCOMMAND, const void*);

typedef struct VECCALLBACK
{
    VECCALLBACK_fn callback;
    const void* data;
    void (*destructor)(const void*);
} VECCALLBACK;

typedef float (*SLICECALLBACK_fn)(SLICEDRAWCOMMAND, const void*);

typedef struct SLICECALLBACK
{
    SLICECALLBACK_fn callback;
    const void* data;
    void (*destructor)(const void*);
} SLICECALLBACK;

typedef struct SLICEMUTDRAWCOMMAND
{
    DRAWCOMMAND* data;
    uint64_t len;
} SLICEMUTDRAWCOMMAND;

typedef float (*SHAPECALLBACK_fn)(SHAPE, const void*);

typedef struct SHAPECALLBACK
{
    SHAPECALLBACK_fn callback;
    const void* data;
    void (*destructor)(const void*);
} SHAPECALLBACK;


VECDRAWCOMMAND create_default_commands(void);
void destroy_draw_commands(VECDRAWCOMMAND _COMMANDS);
OPTIONVEC2 find_largest_position(SLICEDRAWCOMMAND COMMANDS);
void invoke_callback_kitchen_sink(KITCHENSINKCALLBACK CALLBACK);
float invoke_callback_option(OPTIONVEC2 OPT, OPTIONCALLBACK CALLBACK);
float invoke_callback_shape(SHAPE SHAPE, SHAPECALLBACK CALLBACK);
float invoke_callback_slice(SLICEDRAWCOMMAND COMMANDS, SLICECALLBACK CALLBACK);
float invoke_callback_vec(VECCALLBACK CALLBACK);
void scale_commands(SLICEMUTDRAWCOMMAND COMMANDS, float FACTOR);
float shape_area(SHAPE SHAPE);
float total_area(SLICEDRAWCOMMAND COMMANDS);

typedef struct hello_world_c_api_t
{
    VECDRAWCOMMAND (*create_default_commands)(void);
    void (*destroy_draw_commands)(VECDRAWCOMMAND);
    OPTIONVEC2 (*find_largest_position)(SLICEDRAWCOMMAND);
    void (*invoke_callback_kitchen_sink)(KITCHENSINKCALLBACK);
    float (*invoke_callback_option)(OPTIONVEC2, OPTIONCALLBACK);
    float (*invoke_callback_shape)(SHAPE, SHAPECALLBACK);
    float (*invoke_callback_slice)(SLICEDRAWCOMMAND, SLICECALLBACK);
    float (*invoke_callback_vec)(VECCALLBACK);
    void (*scale_commands)(SLICEMUTDRAWCOMMAND, float);
    float (*shape_area)(SHAPE);
    float (*total_area)(SLICEDRAWCOMMAND);
} hello_world_c_api_t;

#if defined(_WIN32)
#include <windows.h>
static int hello_world_c_load(const char* path, hello_world_c_api_t* api)
{
    HMODULE lib = LoadLibraryA(path);
    if (!lib) return -1;
    api->create_default_commands = (VECDRAWCOMMAND (*)(void))(void*)GetProcAddress(lib, "create_default_commands");
    if (!api->create_default_commands) return -1;
    api->destroy_draw_commands = (void (*)(VECDRAWCOMMAND))(void*)GetProcAddress(lib, "destroy_draw_commands");
    if (!api->destroy_draw_commands) return -1;
    api->find_largest_position = (OPTIONVEC2 (*)(SLICEDRAWCOMMAND))(void*)GetProcAddress(lib, "find_largest_position");
    if (!api->find_largest_position) return -1;
    api->invoke_callback_kitchen_sink = (void (*)(KITCHENSINKCALLBACK))(void*)GetProcAddress(lib, "invoke_callback_kitchen_sink");
    if (!api->invoke_callback_kitchen_sink) return -1;
    api->invoke_callback_option = (float (*)(OPTIONVEC2, OPTIONCALLBACK))(void*)GetProcAddress(lib, "invoke_callback_option");
    if (!api->invoke_callback_option) return -1;
    api->invoke_callback_shape = (float (*)(SHAPE, SHAPECALLBACK))(void*)GetProcAddress(lib, "invoke_callback_shape");
    if (!api->invoke_callback_shape) return -1;
    api->invoke_callback_slice = (float (*)(SLICEDRAWCOMMAND, SLICECALLBACK))(void*)GetProcAddress(lib, "invoke_callback_slice");
    if (!api->invoke_callback_slice) return -1;
    api->invoke_callback_vec = (float (*)(VECCALLBACK))(void*)GetProcAddress(lib, "invoke_callback_vec");
    if (!api->invoke_callback_vec) return -1;
    api->scale_commands = (void (*)(SLICEMUTDRAWCOMMAND, float))(void*)GetProcAddress(lib, "scale_commands");
    if (!api->scale_commands) return -1;
    api->shape_area = (float (*)(SHAPE))(void*)GetProcAddress(lib, "shape_area");
    if (!api->shape_area) return -1;
    api->total_area = (float (*)(SLICEDRAWCOMMAND))(void*)GetProcAddress(lib, "total_area");
    if (!api->total_area) return -1;
    return 0;
}
#else
#include <dlfcn.h>
#include <string.h>
static int hello_world_c_load(const char* path, hello_world_c_api_t* api)
{
    void* lib = dlopen(path, RTLD_NOW);
    if (!lib) return -1;
    void* sym;
    sym = dlsym(lib, "create_default_commands");
    if (!sym) return -1;
    memcpy(&api->create_default_commands, &sym, sizeof(sym));
    sym = dlsym(lib, "destroy_draw_commands");
    if (!sym) return -1;
    memcpy(&api->destroy_draw_commands, &sym, sizeof(sym));
    sym = dlsym(lib, "find_largest_position");
    if (!sym) return -1;
    memcpy(&api->find_largest_position, &sym, sizeof(sym));
    sym = dlsym(lib, "invoke_callback_kitchen_sink");
    if (!sym) return -1;
    memcpy(&api->invoke_callback_kitchen_sink, &sym, sizeof(sym));
    sym = dlsym(lib, "invoke_callback_option");
    if (!sym) return -1;
    memcpy(&api->invoke_callback_option, &sym, sizeof(sym));
    sym = dlsym(lib, "invoke_callback_shape");
    if (!sym) return -1;
    memcpy(&api->invoke_callback_shape, &sym, sizeof(sym));
    sym = dlsym(lib, "invoke_callback_slice");
    if (!sym) return -1;
    memcpy(&api->invoke_callback_slice, &sym, sizeof(sym));
    sym = dlsym(lib, "invoke_callback_vec");
    if (!sym) return -1;
    memcpy(&api->invoke_callback_vec, &sym, sizeof(sym));
    sym = dlsym(lib, "scale_commands");
    if (!sym) return -1;
    memcpy(&api->scale_commands, &sym, sizeof(sym));
    sym = dlsym(lib, "shape_area");
    if (!sym) return -1;
    memcpy(&api->shape_area, &sym, sizeof(sym));
    sym = dlsym(lib, "total_area");
    if (!sym) return -1;
    memcpy(&api->total_area, &sym, sizeof(sym));
    return 0;
}
#endif

#ifdef HELLO_WORLD_C_STATIC
static int hello_world_c_load_static(hello_world_c_api_t* api)
{
    api->create_default_commands = create_default_commands;
    api->destroy_draw_commands = destroy_draw_commands;
    api->find_largest_position = find_largest_position;
    api->invoke_callback_kitchen_sink = invoke_callback_kitchen_sink;
    api->invoke_callback_option = invoke_callback_option;
    api->invoke_callback_shape = invoke_callback_shape;
    api->invoke_callback_slice = invoke_callback_slice;
    api->invoke_callback_vec = invoke_callback_vec;
    api->scale_commands = scale_commands;
    api->shape_area = shape_area;
    api->total_area = total_area;
    return 0;
}
#endif /* HELLO_WORLD_C_STATIC */

#ifdef __cplusplus
}
#endif

#endif /* interoptopus_generated */
