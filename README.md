# Variable size enum for Rust

cargo build
cargo run -p example
cargo test

Inspired by the goose programming language variable size enum 05_shapes.goose

# `variable_size_enum`

**Compact storage for Rust enums with variants of different sizes.**

`variable_size_enum` is a Rust library that generates compact representations of enum variants and stores them as a contiguous sequence of bytes. Instead of reserving enough space for the largest variant for every element, it stores each value using the size of its active variant.

This reduces memory usage when working with large collections of enums whose variants contain different amounts of data.

## Why?

Consider an enum with three data-carrying variants and one empty variant:

```rust
#[derive(Debug, Copy, Clone)]
pub enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri { a: f64, b: f64, c: f64 },
    Dot,
}
```

The enum's size is determined by its largest variant, along with the space required for its discriminant and any layout padding. In the example below, `size_of::<Shape>()` is 32 bytes.

A conventional `Vec<Shape>` reserves a fixed 32 bytes per element, even when an element is just `Dot`.

With `variable_size_enum`, the variants can be encoded using their own generated representations. `CompactVec` concatenates these encodings in a byte buffer, avoiding the unused space that a fixed-size enum representation would otherwise reserve for smaller variants.

## Example

Add the library to your project:

```toml
[dependencies]
variable_size_enum = "0.1"
```

### 1. Annotate your enum

The `#[variable_size_enum(kind = ...)]` attribute macro is **mandatory** for enums used with `CompactVec`. It generates the variant representations and implements the `VariableSizeEnum` trait.

```rust
use variable_size_enum::variable_size_enum;

#[variable_size_enum(kind = "u64")]
#[derive(Debug, Copy, Clone)]
pub enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri { a: f64, b: f64, c: f64 },
    Dot,
}
```

The `kind` argument specifies the integer type used for variant discriminants.

### 2. Use the generated variant representations

The macro generates a representation struct for each variant, along with a discriminant type.

```rust
let circle = Circle::new(5.0);
let rect = Rect::new(10.0, 20.0);
let tri = Tri::new(3.0, 4.0, 5.0);
let dot = Dot::new();

assert_eq!(circle.kind, ShapeKind::Circle as u64);
assert_eq!(rect.kind, ShapeKind::Rect as u64);
assert_eq!(tri.kind, ShapeKind::Tri as u64);
assert_eq!(dot.kind, ShapeKind::Dot as u64);
```

These generated types provide a representation for the fields and discriminant of each variant. The macro also generates the conversion support used by the library.

### 4. Store values in `CompactVec`

`CompactVec<T>` stores each encoded value back-to-back in a single byte buffer.

```rust
use variable_size_enum::CompactVec;

let mut shapes = CompactVec::<Shape>::new();

shapes.push(Shape::Circle { r: 5.0 });
shapes.push(Shape::Rect { w: 10.0, h: 20.0 });
shapes.push(Shape::Tri { a: 3.0, b: 4.0, c: 5.0 });
shapes.push(Shape::Dot);

assert_eq!(shapes.len(), 4);

for (index, shape) in shapes.iter().enumerate() {
    println!("Shape {index}: {shape:?}");
}

println!("Bytes used: {}", shapes.used_bytes());
println!("Maximum bytes per shape: {}", Shape::MAX_SIZE);
```

`CompactVec` provides a familiar collection interface while keeping the stored representation compact.

## Memory savings

Here is the output from the example program:

```text
Pushing Circle, Rect, Tri, Dot variants...
Stored shapes: 4
Bytes used: 80
Bytes saved over Vec<Shape> (would be 128): 48
Maximum bytes per shape: 32. Default sizeof::<Shape>(): 32
```

For this particular collection:

| Metric | Bytes |
|---|---:|
| Number of values | 4 |
| Fixed size per `Shape` | 32 |
| Equivalent `Vec<Shape>` element storage | 128 |
| `CompactVec` encoded storage | 80 |
| Bytes saved | 48 |
| Reduction in element storage | 37.5% |

The calculation compares the element storage only. It excludes vector allocation metadata, spare capacity, and allocator overhead. Actual memory consumption depends on allocation capacity and the collection's contents.

The result illustrates the main benefit: smaller variants do not have to occupy the space needed by the largest enum variant.

## Putting generated types in a module

If you prefer to keep generated representation types out of the surrounding namespace, specify a module:

```rust
#[variable_size_enum(kind = u64, module = "prefix")]
#[derive(Debug, Copy, Clone)]
pub enum ShapeWithPrefix {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri { a: f64, b: f64, c: f64 },
    Dot,
}
```

The generated types can then be accessed through the module:

```rust
let circle = prefix::Circle::new(8.0);
let dot = prefix::Dot::new();

println!("Circle radius: {}", circle.r);
```

The module argument controls where generated types are placed; it does not change the name of the original enum.

## API overview

### `CompactVec<T>`

A collection that stores encoded enum values in a contiguous byte buffer.

- `new()` — creates an empty collection.
- `push(item)` — appends a value.
- `len()` — returns the number of values.
- `is_empty()` — checks whether the collection is empty.
- `used_bytes()` — reports the number of bytes currently used by the encoded values.
- `iter()` — iterates over decoded values.

## When should I use it?

`variable_size_enum` is particularly useful when:

- You have an enum with variants of substantially different sizes.
- You store many values in a collection.
- You want contiguous storage with less wasted space for small variants.
- You can work with the library's encoding and decoding model.

It may be less beneficial when variants are already similar in size, when the collection is small, or when encoding and decoding overhead outweighs the memory savings.

## License

This project is licensed under the MIT License.
