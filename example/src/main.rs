use variable_size_enum::{variable_size_enum, CompactVec, VariableSizeEnum};

#[variable_size_enum(kind = "u64")]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri { a: f64, b: f64, c: f64 },
    Dot,
}

#[variable_size_enum(kind = u64, module = "prefix")]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ShapeWithPrefix {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri { a: f64, b: f64, c: f64 },
    Dot,
}

fn main() {
    // 1. Inspect the generated discriminants.
    let kind = ShapeKind::Circle;
    println!("Circle discriminant: {kind:?}");
    assert_eq!(kind, ShapeKind::Circle);

    // 2. Construct the generated variant structs.
    let circle = Circle::new(5.0);
    let rect = Rect::new(10.0, 20.0);
    let tri = Tri::new(3.0, 4.0, 5.0);
    let dot = Dot::new();

    assert_eq!(circle.kind, ShapeKind::Circle as u64);
    assert_eq!(rect.kind, ShapeKind::Rect as u64);
    assert_eq!(tri.kind, ShapeKind::Tri as u64);
    assert_eq!(dot.kind, ShapeKind::Dot as u64);

    println!("Circle radius: {}", circle.r);
    println!("Rectangle dimensions: {} x {}", rect.w, rect.h);
    println!("Triangle sides: {}, {}, {}", tri.a, tri.b, tri.c);

    // 3. Convert between the enum and its generated representation.
    let shape = Shape::Circle { r: 5.0 };
    let circle_struct: Circle = shape.try_into().expect("Correct enum variant");
    let shape_again: Shape = circle_struct.into();

    assert_eq!(shape_again, shape);

    // 4. Serialize a value into bytes and deserialize it.
    let shape = Shape::Rect { w: 10.0, h: 20.0 };
    let mut bytes = Vec::<u8>::with_capacity(Shape::MAX_SIZE);

    let written = shape.write_to_uninit(&mut bytes.spare_capacity_mut()[..Shape::MAX_SIZE]);

    // SAFETY: write_to_uninit guarantee that the first `written` bytes have been initialized.
    unsafe {
        bytes.set_len(written);
    }

    let (decoded, consumed) = Shape::from_bytes(&bytes);

    assert_eq!(decoded, shape);
    assert_eq!(consumed, written);

    println!("Packed rectangle: {bytes:?}");
    println!("Bytes written: {written}");

    // 5. Store mixed variants in CompactVec.
    let mut shapes = CompactVec::<Shape>::new();

    println!("Pushing Circle, Rect, Tri, Dot variants...");
    shapes.push(Shape::Circle { r: 5.0 });
    shapes.push(Shape::Rect { w: 10.0, h: 20.0 });
    shapes.push(Shape::Tri {
        a: 3.0,
        b: 4.0,
        c: 5.0,
    });
    shapes.push(Shape::Dot);

    assert_eq!(shapes.len(), 4);
    assert!(!shapes.is_empty());

    println!("Stored shapes: {}", shapes.len());
    println!("Bytes used: {}", shapes.used_bytes());
    println!(
        "Bytes saved over Vec<Shape> (would be {}): {}",
        4 * size_of::<Shape>(),
        4 * size_of::<Shape>() - shapes.used_bytes()
    );
    println!(
        "Maximum bytes per shape: {}. Default sizeof::<Shape>(): {}",
        Shape::MAX_SIZE,
        size_of::<Shape>()
    );

    // 6. Iterate over the compact collection.
    for (index, shape) in shapes.iter().enumerate() {
        println!("Shape {index}: {shape:?}");
    }

    // 7. The generated structs can be placed in a module.
    let prefixed_circle = prefix::Circle::new(8.0);
    let prefixed_dot = prefix::Dot::new();

    assert_eq!(prefixed_circle.kind, ShapeWithPrefixKind::Circle as u64);
    assert_eq!(prefixed_dot.kind, ShapeWithPrefixKind::Dot as u64);

    println!("Module-scoped circle radius: {}", prefixed_circle.r);

    // 8. Verify the prefixed enum's trait implementation.
    let prefixed_shape = ShapeWithPrefix::Circle { r: 8.0 };
    let mut prefixed_bytes = Vec::<u8>::with_capacity(ShapeWithPrefix::MAX_SIZE);

    let written = prefixed_shape
        .write_to_uninit(&mut prefixed_bytes.spare_capacity_mut()[..ShapeWithPrefix::MAX_SIZE]);

    // SAFETY: write_to_uninit guarantee that the first `written` bytes have been initialized.
    unsafe {
        prefixed_bytes.set_len(written);
    }

    let (decoded, consumed) = ShapeWithPrefix::from_bytes(&prefixed_bytes);

    assert_eq!(decoded, prefixed_shape);
    assert_eq!(consumed, written);

    println!("Prefixed enum round-trip: {decoded:?}");
}
