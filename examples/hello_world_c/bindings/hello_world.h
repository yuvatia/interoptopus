

#ifndef interoptopus_generated
#define interoptopus_generated

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include <sys/types.h>




///  UTF-8 string marshalling helper.
///  A highly dangerous 'use once type' that has ownership semantics!
///  Once passed over an FFI boundary 'the other side' is meant to own
///  (and free) it. Rust handles that fine, but if in C# you put this
///  in a struct and then call Rust multiple times with that struct 
///  you'll free the same pointer multiple times, and get UB!
typedef struct UTF8STRING
    {
    uint8_t* ptr;
    uint64_t len;
    uint64_t capacity;
    } UTF8STRING;

typedef struct VEC2
    {
    float x;
    float y;
    } VEC2;

/// Option that contains Some(value) or None.
typedef enum OPTIONUTF8STRING_TAG
    {
    /// Element if Some().
    OPTIONUTF8STRING_SOME = 0,
    OPTIONUTF8STRING_NONE = 1,
    } OPTIONUTF8STRING_TAG;

typedef struct OPTIONUTF8STRING
    {
    OPTIONUTF8STRING_TAG tag;
    union
        {
        UTF8STRING some;
        };
    } OPTIONUTF8STRING;

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

/// Option that contains Some(value) or None.
typedef enum OPTIONVEC2_TAG
    {
    /// Element if Some().
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

typedef float (*OPTIONCALLBACK_fn)(OPTIONVEC2 OPT, const void* CALLBACK_DATA);

typedef struct OPTIONCALLBACK
    {
    OPTIONCALLBACK_fn call;
    const void* context;
    } OPTIONCALLBACK;

typedef float (*SHAPECALLBACK_fn)(SHAPE SHAPE, const void* CALLBACK_DATA);

typedef struct SHAPECALLBACK
    {
    SHAPECALLBACK_fn call;
    const void* context;
    } SHAPECALLBACK;

/// A pointer to an array of data someone else owns which may not be modified.
typedef struct SLICEDRAWCOMMAND
    {
    /// Pointer to start of immutable data.
    const DRAWCOMMAND* data;
    /// Number of elements.
    uint64_t len;
    } SLICEDRAWCOMMAND;

/// A pointer to an array of data someone else owns which may be modified.
typedef struct SLICEMUTDRAWCOMMAND
    {
    /// Pointer to start of mutable data.
    const DRAWCOMMAND* data;
    /// Number of elements.
    uint64_t len;
    } SLICEMUTDRAWCOMMAND;

///  Vec marshalling helper.
///  A highly dangerous 'use once type' that has ownership semantics!
///  Once passed over an FFI boundary 'the other side' is meant to own
///  (and free) it. Rust handles that fine, but if in C# you put this
///  in a struct and then call Rust multiple times with that struct 
///  you'll free the same pointer multiple times, and get UB!
typedef struct VECDRAWCOMMAND
    {
    DRAWCOMMAND* ptr;
    uint64_t len;
    uint64_t capacity;
    } VECDRAWCOMMAND;

///  A struct exercising all major FFI types at once.
typedef struct KITCHENSINK
    {
    uint64_t id;
    bool enabled;
    double ratio;
    UTF8STRING label;
    SHAPE shape;
    OPTIONVEC2 position;
    SLICEDRAWCOMMAND tags;
    OPTIONUTF8STRING name;
    } KITCHENSINK;

typedef float (*SLICECALLBACK_fn)(SLICEDRAWCOMMAND COMMANDS, const void* CALLBACK_DATA);

typedef struct SLICECALLBACK
    {
    SLICECALLBACK_fn call;
    const void* context;
    } SLICECALLBACK;

typedef float (*VECCALLBACK_fn)(VECDRAWCOMMAND COMMANDS, const void* CALLBACK_DATA);

typedef struct VECCALLBACK
    {
    VECCALLBACK_fn call;
    const void* context;
    } VECCALLBACK;

typedef void (*KITCHENSINKCALLBACK_fn)(const KITCHENSINK* SINK, const void* CALLBACK_DATA);

typedef struct KITCHENSINKCALLBACK
    {
    KITCHENSINKCALLBACK_fn call;
    const void* context;
    } KITCHENSINKCALLBACK;


float shape_area(SHAPE SHAPE);

float total_area(SLICEDRAWCOMMAND COMMANDS);

void scale_commands(SLICEMUTDRAWCOMMAND COMMANDS, float FACTOR);

VECDRAWCOMMAND create_default_commands();

void destroy_draw_commands(VECDRAWCOMMAND COMMANDS);

OPTIONVEC2 find_largest_position(SLICEDRAWCOMMAND COMMANDS);

float invoke_callback_shape(SHAPE SHAPE, SHAPECALLBACK CALLBACK);

float invoke_callback_slice(SLICEDRAWCOMMAND COMMANDS, SLICECALLBACK CALLBACK);

float invoke_callback_option(OPTIONVEC2 OPT, OPTIONCALLBACK CALLBACK);

float invoke_callback_vec(VECCALLBACK CALLBACK);

void invoke_callback_kitchen_sink(KITCHENSINKCALLBACK CALLBACK);


typedef struct hello_world_c_api_t
    {
        float (*shape_area)(SHAPE);
        float (*total_area)(SLICEDRAWCOMMAND);
        void (*scale_commands)(SLICEMUTDRAWCOMMAND, float);
        VECDRAWCOMMAND (*create_default_commands)();
        void (*destroy_draw_commands)(VECDRAWCOMMAND);
        OPTIONVEC2 (*find_largest_position)(SLICEDRAWCOMMAND);
        float (*invoke_callback_shape)(SHAPE, SHAPECALLBACK);
        float (*invoke_callback_slice)(SLICEDRAWCOMMAND, SLICECALLBACK);
        float (*invoke_callback_option)(OPTIONVEC2, OPTIONCALLBACK);
        float (*invoke_callback_vec)(VECCALLBACK);
        void (*invoke_callback_kitchen_sink)(KITCHENSINKCALLBACK);
    } hello_world_c_api_t;

#if defined(_WIN32)
#include <windows.h>
static int hello_world_c_load(const char* path, hello_world_c_api_t* api)
    {
        HMODULE lib = LoadLibraryA(path);
        if (!lib) return -1;
        api->shape_area = (float (*)(SHAPE))(void*)GetProcAddress(lib, "shape_area");
        if (!api->shape_area) return -1;
        api->total_area = (float (*)(SLICEDRAWCOMMAND))(void*)GetProcAddress(lib, "total_area");
        if (!api->total_area) return -1;
        api->scale_commands = (void (*)(SLICEMUTDRAWCOMMAND, float))(void*)GetProcAddress(lib, "scale_commands");
        if (!api->scale_commands) return -1;
        api->create_default_commands = (VECDRAWCOMMAND (*)())(void*)GetProcAddress(lib, "create_default_commands");
        if (!api->create_default_commands) return -1;
        api->destroy_draw_commands = (void (*)(VECDRAWCOMMAND))(void*)GetProcAddress(lib, "destroy_draw_commands");
        if (!api->destroy_draw_commands) return -1;
        api->find_largest_position = (OPTIONVEC2 (*)(SLICEDRAWCOMMAND))(void*)GetProcAddress(lib, "find_largest_position");
        if (!api->find_largest_position) return -1;
        api->invoke_callback_shape = (float (*)(SHAPE, SHAPECALLBACK))(void*)GetProcAddress(lib, "invoke_callback_shape");
        if (!api->invoke_callback_shape) return -1;
        api->invoke_callback_slice = (float (*)(SLICEDRAWCOMMAND, SLICECALLBACK))(void*)GetProcAddress(lib, "invoke_callback_slice");
        if (!api->invoke_callback_slice) return -1;
        api->invoke_callback_option = (float (*)(OPTIONVEC2, OPTIONCALLBACK))(void*)GetProcAddress(lib, "invoke_callback_option");
        if (!api->invoke_callback_option) return -1;
        api->invoke_callback_vec = (float (*)(VECCALLBACK))(void*)GetProcAddress(lib, "invoke_callback_vec");
        if (!api->invoke_callback_vec) return -1;
        api->invoke_callback_kitchen_sink = (void (*)(KITCHENSINKCALLBACK))(void*)GetProcAddress(lib, "invoke_callback_kitchen_sink");
        if (!api->invoke_callback_kitchen_sink) return -1;
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
        sym = dlsym(lib, "shape_area");
        if (!sym) return -1;
        memcpy(&api->shape_area, &sym, sizeof(sym));
        sym = dlsym(lib, "total_area");
        if (!sym) return -1;
        memcpy(&api->total_area, &sym, sizeof(sym));
        sym = dlsym(lib, "scale_commands");
        if (!sym) return -1;
        memcpy(&api->scale_commands, &sym, sizeof(sym));
        sym = dlsym(lib, "create_default_commands");
        if (!sym) return -1;
        memcpy(&api->create_default_commands, &sym, sizeof(sym));
        sym = dlsym(lib, "destroy_draw_commands");
        if (!sym) return -1;
        memcpy(&api->destroy_draw_commands, &sym, sizeof(sym));
        sym = dlsym(lib, "find_largest_position");
        if (!sym) return -1;
        memcpy(&api->find_largest_position, &sym, sizeof(sym));
        sym = dlsym(lib, "invoke_callback_shape");
        if (!sym) return -1;
        memcpy(&api->invoke_callback_shape, &sym, sizeof(sym));
        sym = dlsym(lib, "invoke_callback_slice");
        if (!sym) return -1;
        memcpy(&api->invoke_callback_slice, &sym, sizeof(sym));
        sym = dlsym(lib, "invoke_callback_option");
        if (!sym) return -1;
        memcpy(&api->invoke_callback_option, &sym, sizeof(sym));
        sym = dlsym(lib, "invoke_callback_vec");
        if (!sym) return -1;
        memcpy(&api->invoke_callback_vec, &sym, sizeof(sym));
        sym = dlsym(lib, "invoke_callback_kitchen_sink");
        if (!sym) return -1;
        memcpy(&api->invoke_callback_kitchen_sink, &sym, sizeof(sym));
        return 0;
    }
#endif

#ifdef __cplusplus
}
#endif

#endif /* interoptopus_generated */
