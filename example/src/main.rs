// Inspired by the goose programming language variable size enum 05_shapes.goose

use variable_size_enum::variable_enum;

#[variable_enum]
pub enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri { a: f64, b: f64, c: f64 },
    Dot,
}

fn main() {
    let d = ShapeDiscriminant::Circle;

    println!("{d:?}");

    assert_eq!(d, ShapeDiscriminant::Circle);
}
