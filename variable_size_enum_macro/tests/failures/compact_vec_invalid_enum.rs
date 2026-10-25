use variable_size_enum::CompactVec;

// Missing macro for marker trait VariableSizeEnum
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri { a: f64, b: f64, c: f64 },
    Dot,
}

fn main() {
    let shapes = CompactVec::<Shape>::new();
}
