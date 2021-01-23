# Dynamic template loading

## Description

Specify the path to template file at runtime, compile it, and then render with supplied data.

This operation should be type-safe, and not raise any error after template compilation.

## `sailfish::dynamic::compile` function API

#### Signature

```rust
fn compile<Data: TemplateData, P: AsRef<Path>>(path: P) -> DynamicTemplate<Data>;
```

#### Behaviour

1. Generate Rust code to render templates
2. Compile it as a shared library by calling `cargo build` command.
3. Load the generated shared library.
4. returns the `DynamicTemplate<Data>` struct which contains the function pointer to call the template function.

## `DynamicTemplate::render` method API

#### Signature

```rust
impl<Data: TemplateData> DynamicTemplate<Data> {
    fn render(&self, data: &data) -> RenderResult;
}
```

#### Behaviour

1. Serialize the `data` to byte array
2. Create the vtable for memory allocation (See the below section)
3. Pass the those objects to the template function pointer.
4. Retrieve the result from function pointer, deserialize it to `Result<String>` and then return it.

Trait bound makes this code type-safe.

## Safety for memory allocation

Since compiler used for compiling templates at runtime is different from the one used for compiling renderer, we must export allocator functions as vtable and share it.

```rust
#[repr(C)]
pub struct AllocVtable {
    pub alloc: unsafe fn(Layout) -> *mut u8,
    pub realloc: unsafe fn(*mut u8, Layout, usize) -> *mut u8,
}

struct VBuffer {
    data: *mut u8,
    len: usize,
    capacity: usize,
    vtable: AllocVTable,
}
```

AllocVtable is passed to template function, and then VBuffer is constructed inside template function.

VBuffer should always use AllocVTable to allocate/reallocate a new memory. That cannot achieve with `std::string::String` struct only. We must re-implement the `RawVec` struct.

## Rust standard library conflict problem

Rarely, but not never, dynamically compiled templates may use different version of standard library.

This causes an Undefined behaviour, so we should add `#![no_std]` attribute inside generate Rust code.

However, since it is a corner case, It may be better if we provide `no_std=false` option to avoid this behaviour.

## `TempalteData` trait

We must ensure that all of the data passed to templates should satisfy the following restrictions.

- completely immutable
- does not allocate/deallocate memory
- can be serialized to/deserialized from byte array (All data is serialized to byte array, and then decoded inside templates)
- can be defined inside `#![no_std]` crate

Sailfish provide `TemplateData` trait which satisfies the above restrictions.

```rust
pub unsafe trait TemplateData {
    fn type_name() -> String;
    fn definition() -> String;
    fn fields() -> &'static [&'static str];
    fn deserialize() -> String;  // rust code to deserialize struct
    fn serialize(&self, v: &mut Vec<u8>);
}
```

This trait can be implemented to the following types

- String,
- Primitive integers (bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, isize, usize)
- [T; N] where T: TemplateData
- (T1, T2, T3, ...) where T1, T2, T3, ... : TemplateData
- Option\<T\> where T: TemplateData
- Vec\<T\> where T: TemplateData

### `#[derive(TemplateData)]` attribute

In order to pass the user-defined data, User must implement `TemplateData` manually. However, it is dangerous and should be avoided.

We must export the `derive(TemplateData)` procedural macro to automatically implement this trait.

This macro should cause error if any type of the fields does not implement `TemplateData`.

### How template file is transformed (current idea)

Template file contents is transformed into Rust code when `sailfish::dynamic::compile()` function is called.

For example, if we have a template

```html
<h1><%= msg %></h1>
```

and Rust code

```rust
struct Message {
    msg: String,
}

let template = compile::<Message>("templates/message.stpl").unwrap();
```

then, template will be transformed into the following code.

```rust
#![no_std]
use sailfish::dynamic::runtime as sfrt;
use sfrt::{VBuffer, AllocVtable, OutputData, SizeHint, RenderResult};

struct Message {
    msg: String,
}

fn deserialize(data: &mut &[u8]) -> Message {
    // Generated code from TemplateData::deserialize()
    let msg = sfrt::deserialize_string(data);

    Message { msg }
}

#[no_mangle]
pub extern fn sf_message(version: u64, data: *const [u8], data_len: usize, vtable: AllocVtable) -> OutputData {
    let inner = move || -> RenderResult {
        let mut data = unsafe { std::slice::from_raw_parts(data, data_len) };
        let Message { msg } = deserialize(&mut data);

        let mut buf = VBuffer::from_vtable(vtable);
        
        static SIZE_HINT = SizeHint::new();
        let size_hint = SIZE_HINT.get();
        buf.reserve(size_hint);

        {
            sfrt::render_text!(buf, "<h1>");
            sfrt::render_escaped!(buf, msg);
            sfrt::render_text!(buf, "</h1>");
        }

        SIZE_HINT.update(buf.len())
        Ok(buf.into_string())
    };

    OutputData::from_result(inner())
}
```

## Example usage

Template:

```html
<!DOCTYPE html>
<html>
  <body>
    <b><%= name %></b>: <%= score %>
  </body>
</html>
```

Rust code:

```rust
use sailfish::dynamic::compile;
use sailfish_macros::TemplateData;

#[derive(TemplateData)]
pub struct Team {
    name: String,
    score: u8
}

// compile the template as a callable shared library
let template: DynamicTemplate<Team> = compile::<Team>("templates/team.stpl").unwrap();
let data = Team {
    name: "Jiangsu".into(),
    score: 43
};
// render templates with given data
let result: String = unsafe { template.render(data).unwrap() };
println!("{}", result);
```
