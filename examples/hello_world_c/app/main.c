#include <stdio.h>
#include "../bindings/hello_world.h"

#if defined(_WIN32)
#define LIB_PATH "hello_world_c.dll"
#elif defined(__APPLE__)
#define LIB_PATH "libhello_world_c.dylib"
#else
#define LIB_PATH "libhello_world_c.so"
#endif

/* We store the API table globally so callbacks can call back into Rust. */
static hello_world_c_api_t api;

/* ── Callback functions ── */

static float cb_shape_area(SHAPE shape, const void* ctx) {
    (void)ctx;
    return api.shape_area(shape);
}

static float cb_slice_total_area(SLICEDRAWCOMMAND commands, const void* ctx) {
    (void)ctx;
    return api.total_area(commands);
}

static float cb_option_magnitude(OPTIONVEC2 opt, const void* ctx) {
    (void)ctx;
    if (opt.tag == OPTIONVEC2_SOME) {
        float x = opt.some.x;
        float y = opt.some.y;
        return x * x + y * y;
    }
    return -1.0f;
}

static float cb_vec_count(VECDRAWCOMMAND commands, const void* ctx) {
    (void)ctx;
    float count = (float)commands.len;
    /* We received ownership of this Vec — pass it back to Rust for cleanup. */
    api.destroy_draw_commands(commands);
    return count;
}

static void cb_kitchen_sink(const KITCHENSINK* sink, const void* ctx) {
    (void)ctx;
    printf("  KitchenSink {\n");
    printf("    id:       %llu\n", (unsigned long long)sink->id);
    printf("    enabled:  %s\n", sink->enabled ? "true" : "false");
    printf("    ratio:    %f\n", sink->ratio);
    printf("    label:    \"%.*s\"\n", (int)sink->label.len, (const char*)sink->label.ptr);

    if (sink->shape.tag == SHAPE_CIRCLE) {
        printf("    shape:    Circle(r=%f)\n", sink->shape.circle);
    } else {
        printf("    shape:    Rectangle(%f x %f)\n",
               sink->shape.rectangle.x, sink->shape.rectangle.y);
    }

    if (sink->position.tag == OPTIONVEC2_SOME) {
        printf("    position: Some(%f, %f)\n", sink->position.some.x, sink->position.some.y);
    } else {
        printf("    position: None\n");
    }

    printf("    tags:     [%llu commands]\n", (unsigned long long)sink->tags.len);

    if (sink->name.tag == OPTIONSTRING_SOME) {
        printf("    name:     Some(\"%.*s\")\n",
               (int)sink->name.some.len, (const char*)sink->name.some.ptr);
    } else {
        printf("    name:     None\n");
    }

    printf("  }\n");
}

int main(void) {
    if (hello_world_c_load(LIB_PATH, &api) != 0) {
        printf("Failed to load library\n");
        return 1;
    }

    /* ── Basic shapes ── */

    SHAPE circle;
    circle.tag = SHAPE_CIRCLE;
    circle.circle = 5.0f;
    printf("Circle area: %f\n", api.shape_area(circle));

    SHAPE rect;
    rect.tag = SHAPE_RECTANGLE;
    rect.rectangle.x = 3.0f;
    rect.rectangle.y = 4.0f;
    printf("Rectangle area: %f\n", api.shape_area(rect));

    /* ── Vec + Slice ── */

    VECDRAWCOMMAND cmds = api.create_default_commands();
    SLICEDRAWCOMMAND slice;
    slice.data = cmds.ptr;
    slice.len = cmds.len;
    printf("Total area of %llu commands: %f\n",
           (unsigned long long)slice.len, api.total_area(slice));

    /* ── Option ── */

    OPTIONVEC2 largest = api.find_largest_position(slice);
    if (largest.tag == OPTIONVEC2_SOME) {
        printf("Largest shape at: (%f, %f)\n", largest.some.x, largest.some.y);
    }

    /* ── Callback with Shape ── */

    SHAPECALLBACK shape_cb = { cb_shape_area, NULL, NULL };
    printf("Callback shape area: %f\n", api.invoke_callback_shape(circle, shape_cb));

    /* ── Callback with Slice ── */

    SLICECALLBACK slice_cb = { cb_slice_total_area, NULL, NULL };
    printf("Callback slice total: %f\n", api.invoke_callback_slice(slice, slice_cb));

    /* ── Callback with Option ── */

    OPTIONCALLBACK opt_cb = { cb_option_magnitude, NULL, NULL };
    OPTIONVEC2 some_vec;
    some_vec.tag = OPTIONVEC2_SOME;
    some_vec.some.x = 3.0f;
    some_vec.some.y = 4.0f;
    printf("Callback option magnitude: %f\n", api.invoke_callback_option(some_vec, opt_cb));

    OPTIONVEC2 none_vec;
    none_vec.tag = OPTIONVEC2_NONE;
    printf("Callback option none: %f\n", api.invoke_callback_option(none_vec, opt_cb));

    /* ── Callback with Vec (Rust creates + passes Vec, callback destroys) ── */

    VECCALLBACK vec_cb = { cb_vec_count, NULL, NULL };
    printf("Callback vec count: %f\n", api.invoke_callback_vec(vec_cb));

    /* ── Callback with KitchenSink (struct with all FFI types) ── */

    KITCHENSINKCALLBACK sink_cb = { cb_kitchen_sink, NULL, NULL };
    printf("Kitchen sink callback:\n");
    api.invoke_callback_kitchen_sink(sink_cb);

    /* ── Cleanup ── */

    api.destroy_draw_commands(cmds);

    return 0;
}
