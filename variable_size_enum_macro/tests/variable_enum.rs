use variable_size_enum::{CompactVec, VariableSizeEnum, variable_size_enum};

#[variable_size_enum(kind = u64)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri { a: f64, b: f64, c: f64 },
    Dot,
}

#[test]
fn shape_generates_structs_and_kind() {
    let circle = Circle::new(5.0);
    assert_eq!(circle.kind, ShapeKind::Circle as u64);
    assert_eq!(circle.r, 5.0);
    assert!(size_of::<Circle>() > size_of::<f64>()); // + kind

    let rect = Rect::new(10.0, 20.0);
    assert_eq!(rect.kind, ShapeKind::Rect as u64);
    assert_eq!(rect.w, 10.0);
    assert_eq!(rect.h, 20.0);
    assert!(size_of::<Rect>() > 2 * size_of::<f64>()); // + kind

    let tri = Tri::new(3.0, 4.0, 5.0);
    assert_eq!(tri.kind, ShapeKind::Tri as u64);
    assert_eq!(tri.c, 5.0);
    assert!(size_of::<Tri>() > 3 * size_of::<f64>()); // + kind

    let dot = Dot::new();
    assert_eq!(dot.kind, ShapeKind::Dot as u64);
    assert!(size_of_val(&dot) > 0); // + kind
    assert!(size_of::<Dot>() > 0); // + kind
}

#[test]
fn shape_kinds_have_expected_values() {
    assert_eq!(ShapeKind::Circle as u8, 0);
    assert_eq!(ShapeKind::Rect as u8, 1);
    assert_eq!(ShapeKind::Tri as u8, 2);
    assert_eq!(ShapeKind::Dot as u8, 3);
}

#[test]
fn generated_structs_implement_debug() {
    let circle = Circle::new(5.0);
    let output = format!("{:?}", circle);

    assert!(output.contains("Circle"));
    assert!(output.contains("5.0"));
}

#[test]
fn converts_between_shape_and_tagged_structs() {
    // Enum -> Circle
    let shape = Shape::Circle { r: 5.0 };
    let circle = Circle::try_from(shape).unwrap();

    assert_eq!(circle.r, 5.0);
    assert_eq!(circle.kind, ShapeKind::Circle as u64);

    // Circle -> enum
    let shape_again: Shape = circle.into();

    match shape_again {
        Shape::Circle { r } => assert_eq!(r, 5.0),
        _ => panic!("expected Circle"),
    }

    let dot = Dot::new();

    let shape: Shape = dot.into();

    assert!(matches!(shape, Shape::Dot));
}

#[test]
fn conversion_rejects_wrong_variant() {
    let shape = Shape::Dot;

    let result = Circle::try_from(shape);

    assert!(result.is_err());

    // The original enum is preserved on failure.
    assert!(matches!(result.unwrap_err(), Shape::Dot));
}

#[test]
fn compact_vec_iter_returns_all_variants_in_order() {
    let mut shapes = CompactVec::<Shape>::new();
    assert!(shapes.is_empty());

    let expected = [
        Shape::Circle { r: 2.5 },
        Shape::Rect { w: 3.0, h: 4.0 },
        Shape::Tri {
            a: 3.0,
            b: 4.0,
            c: 5.0,
        },
        Shape::Dot,
    ];

    for shape in expected {
        shapes.push(shape);
    }
    assert_eq!(shapes.len(), 4);
    assert!(shapes.used_bytes() < size_of_val(&expected)); // Expected memory gain

    let actual: Vec<Shape> = shapes.iter().collect();

    assert_eq!(actual, expected);
}

#[test]
fn compact_vec_iter_handles_empty_vector() {
    let shapes = CompactVec::<Shape>::new();
    assert!(shapes.is_empty());

    assert_eq!(shapes.iter().count(), 0);
    assert_eq!(shapes.iter().next(), None);
}

#[test]
fn compact_vec_iter_can_be_reused() {
    let mut shapes = CompactVec::<Shape>::new();

    shapes.push(Shape::Circle { r: 1.0 });
    shapes.push(Shape::Dot);

    assert_eq!(shapes.iter().count(), 2);
    assert_eq!(shapes.iter().count(), 2);
    assert_eq!(shapes.len(), 2);
}

#[test]
fn compact_vec_iter_handles_mixed_size_variants() {
    let mut shapes = CompactVec::<Shape>::new();

    shapes.push(Shape::Dot);
    shapes.push(Shape::Tri {
        a: 1.0,
        b: 2.0,
        c: 3.0,
    });
    shapes.push(Shape::Circle { r: 4.0 });
    shapes.push(Shape::Rect { w: 5.0, h: 6.0 });
    shapes.push(Shape::Dot);

    assert_eq!(shapes.len(), 5);
    let actual: Vec<Shape> = shapes.iter().collect();

    assert_eq!(
        actual,
        vec![
            Shape::Dot,
            Shape::Tri {
                a: 1.0,
                b: 2.0,
                c: 3.0,
            },
            Shape::Circle { r: 4.0 },
            Shape::Rect { w: 5.0, h: 6.0 },
            Shape::Dot,
        ]
    );
}

#[test]
fn compact_vec_iter_does_not_consume_the_vector() {
    let mut shapes = CompactVec::<Shape>::new();

    shapes.push(Shape::Circle { r: 7.0 });
    shapes.push(Shape::Rect { w: 8.0, h: 9.0 });

    assert_eq!(shapes.len(), 2);
    let first_pass: Vec<Shape> = shapes.iter().collect();
    let second_pass: Vec<Shape> = shapes.iter().collect();
    assert_eq!(shapes.len(), 2);

    assert_eq!(first_pass, second_pass);
    assert_eq!(shapes.iter().count(), 2);
}

#[test]
fn compact_vec_round_trips_each_shape_through_bytes() {
    let expected = [
        Shape::Circle { r: 2.5 },
        Shape::Rect { w: 3.0, h: 4.0 },
        Shape::Tri {
            a: 3.0,
            b: 4.0,
            c: 5.0,
        },
        Shape::Dot,
    ];

    for shape in expected {
        let mut bytes = Vec::<u8>::with_capacity(Shape::MAX_SIZE);

        let written = shape.write_to_uninit(&mut bytes.spare_capacity_mut()[..Shape::MAX_SIZE]);

        // SAFETY: write_to_uninit must guarantee that the first
        // `written` bytes have been initialized.
        unsafe {
            bytes.set_len(written);
        }

        let (decoded, consumed) = Shape::from_bytes(&bytes);

        assert_eq!(decoded, shape);
        assert_eq!(consumed, written);
    }
}

#[variable_size_enum(kind = u64, module = "prefix")]
#[derive(Debug, Copy, Clone)]
pub enum ShapeWithPrefix {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri { a: f64, b: f64, c: f64 },
    Dot,
}

#[variable_size_enum(kind = "u64", module = "prefix2")]
#[derive(Debug, Copy, Clone)]
pub enum ShapeWithPrefixNoClash {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri { a: f64, b: f64, c: f64 },
    Dot,
}

#[test]
fn shape_with_prefix_generates_structs_and_kind() {
    let circle = prefix::Circle::new(5.0);
    assert_eq!(circle.kind, ShapeWithPrefixKind::Circle as u64);
    assert_eq!(circle.r, 5.0);
    assert!(size_of::<prefix::Circle>() > size_of::<f64>()); // + kind

    let rect = prefix::Rect::new(10.0, 20.0);
    assert_eq!(rect.kind, ShapeWithPrefixKind::Rect as u64);
    assert_eq!(rect.w, 10.0);
    assert_eq!(rect.h, 20.0);
    assert!(size_of::<prefix::Rect>() > 2 * size_of::<f64>()); // + kind

    let tri = prefix::Tri::new(3.0, 4.0, 5.0);
    assert_eq!(tri.kind, ShapeWithPrefixKind::Tri as u64);
    assert_eq!(tri.c, 5.0);
    assert!(size_of::<prefix::Tri>() > 3 * size_of::<f64>()); // + kind

    let dot = prefix::Dot::new();
    assert_eq!(dot.kind, ShapeWithPrefixKind::Dot as u64);
    assert!(size_of_val(&dot) > 0); // + kind
    assert!(size_of::<prefix::Dot>() > 0); // + kind

    assert_eq!(ShapeWithPrefixKind::Circle as u8, 0);
    assert_eq!(ShapeWithPrefixKind::Rect as u8, 1);
    assert_eq!(ShapeWithPrefixKind::Tri as u8, 2);
    assert_eq!(ShapeWithPrefixKind::Dot as u8, 3);
}

#[test]
fn converts_between_prefix_shape_and_tagged_structs() {
    // Enum -> Circle
    let shape = ShapeWithPrefix::Circle { r: 5.0 };
    let circle = prefix::Circle::try_from(shape).unwrap();

    assert_eq!(circle.r, 5.0);
    assert_eq!(circle.kind, ShapeWithPrefixKind::Circle as u64);

    // Circle -> enum
    let shape_again: ShapeWithPrefix = circle.into();

    match shape_again {
        ShapeWithPrefix::Circle { r } => assert_eq!(r, 5.0),
        _ => panic!("expected Circle"),
    }

    let dot = prefix::Dot::new();

    let shape: ShapeWithPrefix = dot.into();

    assert!(matches!(shape, ShapeWithPrefix::Dot));
}

#[variable_size_enum(kind = u64, module = "prefix3::prefix")]
#[derive(Debug, Copy, Clone)]
pub enum ShapeWithPathPrefix {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri { a: f64, b: f64, c: f64 },
    Dot,
}

#[test]
fn shape_with_path_prefix_generates_structs_and_kind() {
    let circle = prefix3::prefix::Circle::new(5.0);
    assert_eq!(circle.kind, ShapeWithPathPrefixKind::Circle as u64);
    assert_eq!(circle.r, 5.0);
    assert!(size_of::<prefix3::prefix::Circle>() > size_of::<f64>()); // + kind

    let rect = prefix3::prefix::Rect::new(10.0, 20.0);
    assert_eq!(rect.kind, ShapeWithPathPrefixKind::Rect as u64);
    assert_eq!(rect.w, 10.0);
    assert_eq!(rect.h, 20.0);
    assert!(size_of::<prefix3::prefix::Rect>() > 2 * size_of::<f64>()); // + kind

    let tri = prefix3::prefix::Tri::new(3.0, 4.0, 5.0);
    assert_eq!(tri.kind, ShapeWithPathPrefixKind::Tri as u64);
    assert_eq!(tri.c, 5.0);
    assert!(size_of::<prefix3::prefix::Tri>() > 3 * size_of::<f64>()); // + kind

    let dot = prefix3::prefix::Dot::new();
    assert_eq!(dot.kind, ShapeWithPathPrefixKind::Dot as u64);
    assert!(size_of_val(&dot) > 0); // + kind
    assert!(size_of::<prefix3::prefix::Dot>() > 0); // + kind

    assert_eq!(ShapeWithPathPrefixKind::Circle as u8, 0);
    assert_eq!(ShapeWithPathPrefixKind::Rect as u8, 1);
    assert_eq!(ShapeWithPathPrefixKind::Tri as u8, 2);
    assert_eq!(ShapeWithPathPrefixKind::Dot as u8, 3);
}

#[variable_size_enum(kind = u8, module = "data_prefix")]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MyData {
    A { r: u8 },
    B { w: u8, h: u8 },
    C { a: u8, b: u16 },
    D,
}

#[test]
fn enum_with_tagged_struct_to_bytes() {
    let a = data_prefix::A::new(5);
    assert_eq!(a.kind, MyDataKind::A as u8);
    assert_eq!(a.r, 5);
    assert!(size_of::<data_prefix::A>() == 2 * size_of::<u8>()); // + kind

    let b = data_prefix::B::new(10, 20);
    assert_eq!(b.kind, MyDataKind::B as u8);
    assert_eq!(b.w, 10);
    assert_eq!(b.h, 20);
    assert!(size_of::<data_prefix::B>() == 3 * size_of::<u8>()); // + kind

    let c = data_prefix::C::new(3, 270);
    assert_eq!(c.kind, MyDataKind::C as u8);
    assert_eq!(c.b, 270);
    assert!(size_of::<data_prefix::C>() == 4 * size_of::<u8>()); // + kind

    let d = data_prefix::D::new();
    assert_eq!(d.kind, MyDataKind::D as u8);
    assert!(size_of_val(&d) > 0); // + kind
    assert!(size_of::<data_prefix::D>() > 0); // + kind

    assert_eq!(MyDataKind::A as u8, 0);
    assert_eq!(MyDataKind::B as u8, 1);
    assert_eq!(MyDataKind::C as u8, 2);
    assert_eq!(MyDataKind::D as u8, 3);

    let a_bytes = bytemuck::bytes_of(&a);
    assert_eq!(a_bytes, [0, 5]); // tag, r
    let b_bytes = bytemuck::bytes_of(&b);
    assert_eq!(b_bytes, [1, 10, 20]); // tag w h
    let c_bytes = bytemuck::bytes_of(&c);
    assert_eq!(c_bytes, [2, 3, 14, 1]); // tag a, b & 0xFF, b >> 0xFF
    let d_bytes = bytemuck::bytes_of(&d);
    assert_eq!(d_bytes, [3]); // tag

    let new_a = bytemuck::from_bytes::<data_prefix::A>(a_bytes);
    assert_eq!(a, *new_a);
    let new_b = bytemuck::from_bytes::<data_prefix::B>(b_bytes);
    assert_eq!(b, *new_b);
    let new_c = bytemuck::from_bytes::<data_prefix::C>(c_bytes);
    assert_eq!(c, *new_c);
    let new_d = bytemuck::from_bytes::<data_prefix::D>(d_bytes);
    assert_eq!(d, *new_d);
}

#[variable_size_enum(kind = u16)]
#[derive(Debug, Copy, Clone)]
pub enum Many {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
    V11,
    V12,
    V13,
    V14,
    V15,
    V16,
    V17,
    V18,
    V19,
    V20,
    V21,
    V22,
    V23,
    V24,
    V25,
    V26,
    V27,
    V28,
    V29,
    V30,
    V31,
    V32,
    V33,
    V34,
    V35,
    V36,
    V37,
    V38,
    V39,
    V40,
    V41,
    V42,
    V43,
    V44,
    V45,
    V46,
    V47,
    V48,
    V49,
    V50,
    V51,
    V52,
    V53,
    V54,
    V55,
    V56,
    V57,
    V58,
    V59,
    V60,
    V61,
    V62,
    V63,
    V64,
    V65,
    V66,
    V67,
    V68,
    V69,
    V70,
    V71,
    V72,
    V73,
    V74,
    V75,
    V76,
    V77,
    V78,
    V79,
    V80,
    V81,
    V82,
    V83,
    V84,
    V85,
    V86,
    V87,
    V88,
    V89,
    V90,
    V91,
    V92,
    V93,
    V94,
    V95,
    V96,
    V97,
    V98,
    V99,
    V100,
    V101,
    V102,
    V103,
    V104,
    V105,
    V106,
    V107,
    V108,
    V109,
    V110,
    V111,
    V112,
    V113,
    V114,
    V115,
    V116,
    V117,
    V118,
    V119,
    V120,
    V121,
    V122,
    V123,
    V124,
    V125,
    V126,
    V127,
    V128,
    V129,
    V130,
    V131,
    V132,
    V133,
    V134,
    V135,
    V136,
    V137,
    V138,
    V139,
    V140,
    V141,
    V142,
    V143,
    V144,
    V145,
    V146,
    V147,
    V148,
    V149,
    V150,
    V151,
    V152,
    V153,
    V154,
    V155,
    V156,
    V157,
    V158,
    V159,
    V160,
    V161,
    V162,
    V163,
    V164,
    V165,
    V166,
    V167,
    V168,
    V169,
    V170,
    V171,
    V172,
    V173,
    V174,
    V175,
    V176,
    V177,
    V178,
    V179,
    V180,
    V181,
    V182,
    V183,
    V184,
    V185,
    V186,
    V187,
    V188,
    V189,
    V190,
    V191,
    V192,
    V193,
    V194,
    V195,
    V196,
    V197,
    V198,
    V199,
    V200,
    V201,
    V202,
    V203,
    V204,
    V205,
    V206,
    V207,
    V208,
    V209,
    V210,
    V211,
    V212,
    V213,
    V214,
    V215,
    V216,
    V217,
    V218,
    V219,
    V220,
    V221,
    V222,
    V223,
    V224,
    V225,
    V226,
    V227,
    V228,
    V229,
    V230,
    V231,
    V232,
    V233,
    V234,
    V235,
    V236,
    V237,
    V238,
    V239,
    V240,
    V241,
    V242,
    V243,
    V244,
    V245,
    V246,
    V247,
    V248,
    V249,
    V250,
    V251,
    V252,
    V253,
    V254,
    V255,
    V256,
    V257,
    V258,
    V259,
    V260,
    V261,
    V262,
    V263,
    V264,
    V265,
    V266,
    V267,
    V268,
    V269,
    V270,
    V271,
    V272,
    V273,
    V274,
    V275,
    V276,
    V277,
    V278,
    V279,
}

#[test]
fn variants_use_u16_representation() {
    assert_eq!(std::mem::size_of::<ManyKind>(), 2);
    assert_eq!(ManyKind::V0 as u16, 0);
    assert_eq!(ManyKind::V255 as u16, 255);
    assert_eq!(ManyKind::V256 as u16, 256);
    assert_eq!(ManyKind::V279 as u16, 279);
}

#[test]
fn generated_unit_struct_has_correct_kind() {
    let value = V279::new();
    assert_eq!(value.kind, ManyKind::V279 as u16);
}
