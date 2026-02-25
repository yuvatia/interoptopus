use interoptopus::{callback, ffi, ffi_function, ffi_type};

// ── Types ──

#[ffi_type]
#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[ffi_type]
#[derive(Clone, Copy)]
pub enum Shape {
    Circle(f32),
    Rectangle(Vec2),
}

#[ffi_type]
#[derive(Clone, Copy)]
pub struct DrawCommand {
    pub shape: Shape,
    pub position: Vec2,
}

/// A struct exercising all major FFI types at once.
#[ffi_type]
pub struct KitchenSink {
    pub id: u64,
    pub enabled: bool,
    pub ratio: f64,
    pub label: ffi::String,
    pub shape: Shape,
    pub position: ffi::Option<Vec2>,
    pub tags: ffi::Slice<'static, DrawCommand>,
    pub name: ffi::Option<ffi::String>,
}

// ── Callbacks ──

callback!(ShapeCallback(shape: Shape) -> f32);
callback!(SliceCallback(commands: ffi::Slice<DrawCommand>) -> f32);
callback!(OptionCallback(opt: ffi::Option<Vec2>) -> f32);
callback!(VecCallback(commands: ffi::Vec<DrawCommand>) -> f32);
callback!(KitchenSinkCallback(sink: &KitchenSink));

// ── Functions ──

#[ffi_function]
pub fn shape_area(shape: Shape) -> f32 {
    match shape {
        Shape::Circle(r) => std::f32::consts::PI * r * r,
        Shape::Rectangle(v) => v.x * v.y,
    }
}

#[ffi_function]
pub fn total_area(commands: ffi::Slice<DrawCommand>) -> f32 {
    commands.as_slice().iter().map(|c| shape_area(c.shape)).sum()
}

#[ffi_function]
pub fn scale_commands(mut commands: ffi::SliceMut<DrawCommand>, factor: f32) {
    for cmd in commands.as_slice_mut() {
        match &mut cmd.shape {
            Shape::Circle(r) => *r *= factor,
            Shape::Rectangle(v) => {
                v.x *= factor;
                v.y *= factor;
            }
        }
    }
}

#[ffi_function]
pub fn create_default_commands() -> ffi::Vec<DrawCommand> {
    let commands = vec![
        DrawCommand { shape: Shape::Circle(5.0), position: Vec2 { x: 0.0, y: 0.0 } },
        DrawCommand { shape: Shape::Rectangle(Vec2 { x: 3.0, y: 4.0 }), position: Vec2 { x: 10.0, y: 10.0 } },
    ];
    ffi::Vec::from(commands)
}

#[ffi_function]
pub fn destroy_draw_commands(_commands: ffi::Vec<DrawCommand>) {
    // Vec is dropped here, freeing the backing allocation.
}

#[ffi_function]
pub fn find_largest_position(commands: ffi::Slice<DrawCommand>) -> ffi::Option<Vec2> {
    let mut max_area = 0.0f32;
    let mut max_pos = None;
    for cmd in commands.as_slice() {
        let area = shape_area(cmd.shape);
        if area > max_area {
            max_area = area;
            max_pos = Some(cmd.position);
        }
    }
    match max_pos {
        Some(p) => ffi::Some(p),
        None => ffi::None,
    }
}

#[ffi_function]
pub fn invoke_callback_shape(shape: Shape, callback: ShapeCallback) -> f32 {
    callback.call(shape)
}

#[ffi_function]
pub fn invoke_callback_slice(commands: ffi::Slice<DrawCommand>, callback: SliceCallback) -> f32 {
    callback.call(commands)
}

#[ffi_function]
pub fn invoke_callback_option(opt: ffi::Option<Vec2>, callback: OptionCallback) -> f32 {
    callback.call(opt)
}

#[ffi_function]
pub fn invoke_callback_vec(callback: VecCallback) -> f32 {
    let commands = vec![
        DrawCommand { shape: Shape::Circle(3.0), position: Vec2 { x: 1.0, y: 1.0 } },
        DrawCommand { shape: Shape::Rectangle(Vec2 { x: 2.0, y: 5.0 }), position: Vec2 { x: 2.0, y: 2.0 } },
    ];
    callback.call(ffi::Vec::from(commands))
}

static SINK_COMMANDS: [DrawCommand; 2] = [
    DrawCommand { shape: Shape::Circle(5.0), position: Vec2 { x: 0.0, y: 0.0 } },
    DrawCommand { shape: Shape::Rectangle(Vec2 { x: 3.0, y: 4.0 }), position: Vec2 { x: 10.0, y: 10.0 } },
];

#[ffi_function]
pub fn invoke_callback_kitchen_sink(callback: KitchenSinkCallback) {
    let sink = KitchenSink {
        id: 42,
        enabled: true,
        ratio: 3.14159265,
        label: ffi::String::from_string("hello from rust".to_string()),
        shape: Shape::Circle(7.5),
        position: ffi::Some(Vec2 { x: 1.0, y: 2.0 }),
        tags: ffi::Slice::from_slice(&SINK_COMMANDS),
        name: ffi::Some(ffi::String::from_string("kitchen sink".to_string())),
    };
    callback.call(&sink);
}

#[test]
#[rustfmt::skip]
fn generate_bindings() {
    use interoptopus::{extra_type, function};
    use interoptopus::inventory::Inventory;
    use interoptopus_backend_c::Interop;

    let inventory = Inventory::builder()
        .register(function!(shape_area))
        .register(function!(total_area))
        .register(function!(scale_commands))
        .register(function!(create_default_commands))
        .register(function!(destroy_draw_commands))
        .register(function!(find_largest_position))
        .register(function!(invoke_callback_shape))
        .register(function!(invoke_callback_slice))
        .register(function!(invoke_callback_option))
        .register(function!(invoke_callback_vec))
        .register(function!(invoke_callback_kitchen_sink))
        .register(extra_type!(DrawCommand))
        .validate()
        .build();

    Interop::builder()
        .inventory(inventory)
        .loader("hello_world_c".to_string())
        .build().unwrap()
        .write_file("bindings/hello_world.h").unwrap()
}
