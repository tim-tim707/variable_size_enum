// Inspired by the goose programming language variable size enum 05_shapes.goose

use variable_size_enum::variable_size_enum;

#[variable_size_enum(kind = "u64")]
#[derive(Debug, Copy, Clone)]
pub enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri { a: f64, b: f64, c: f64 },
    Dot,
}

#[variable_size_enum(kind = u64, module = "prefix")]
#[derive(Debug, Copy, Clone)]
pub enum ShapeWithPrefix {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri { a: f64, b: f64, c: f64 },
    Dot,
}

fn main() {
    let d = ShapeKind::Circle;

    println!("{d:?}");

    assert_eq!(d, ShapeKind::Circle);

    let circle = Circle::new(5.0);
    let rect = Rect::new(10.0, 20.0);
    let dot = Dot::new();

    assert_eq!(circle.kind, ShapeKind::Circle as u64);
    assert_eq!(rect.kind, ShapeKind::Rect as u64);
    assert_eq!(dot.kind, ShapeKind::Dot as u64);

    println!("{:?}", circle.r);
}
