use variable_size_enum::variable_enum;

#[variable_enum(struct_with_kind)]
#[derive(Debug)]
pub enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri { a: f64, b: f64, c: f64 },
    Dot,
}

#[variable_enum(plain_struct)]
#[derive(Debug)]
pub enum VanillaShape {
    Circle2 { r: f64 },
    Rect2 { w: f64, h: f64 },
    Tri2 { a: f64, b: f64, c: f64 },
    Dot2,
}

#[variable_enum(kind, plain_struct)]
#[derive(Debug)]
pub enum ShapeKindlessStruct {
    Circle3 { r: f64 },
    Rect3 { w: f64, h: f64 },
    Tri3 { a: f64, b: f64, c: f64 },
    Dot3,
}

#[variable_enum(struct_with_kind)]
#[derive(Debug)]
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
fn shape_generates_structs_and_kind() {
    let circle = Circle::new(5.0);
    assert_eq!(circle.kind, ShapeKind::Circle);
    assert_eq!(circle.r, 5.0);
    assert!(size_of::<Circle>() > size_of::<f64>()); // + kind

    let rect = Rect::new(10.0, 20.0);
    assert_eq!(rect.kind, ShapeKind::Rect);
    assert_eq!(rect.w, 10.0);
    assert_eq!(rect.h, 20.0);
    assert!(size_of::<Rect>() > 2 * size_of::<f64>()); // + kind

    let tri = Tri::new(3.0, 4.0, 5.0);
    assert_eq!(tri.kind, ShapeKind::Tri);
    assert_eq!(tri.c, 5.0);
    assert!(size_of::<Tri>() > 3 * size_of::<f64>()); // + kind

    let dot = Dot::new();
    assert_eq!(dot.kind, ShapeKind::Dot);
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
    assert_eq!(value.kind, ManyKind::V279);
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
    assert_eq!(circle.kind, ShapeKind::Circle);

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
fn vanilla_shape_generates_structs() {
    let circle = Circle2::new(5.0);
    assert_eq!(circle.r, 5.0);
    assert!(size_of::<Circle2>() == size_of::<f64>());

    let rect = Rect2::new(10.0, 20.0);
    assert_eq!(rect.w, 10.0);
    assert_eq!(rect.h, 20.0);
    assert!(size_of::<Rect2>() == 2 * size_of::<f64>());

    let tri = Tri2::new(3.0, 4.0, 5.0);
    assert_eq!(tri.c, 5.0);
    assert!(size_of::<Tri2>() == 3 * size_of::<f64>());

    let dot = Dot2::new();
    assert!(size_of_val(&dot) == 0);
    assert!(size_of::<Dot2>() == 0);
}

#[test]
fn converts_between_vanilla_shape_and_structs() {
    // Enum -> Circle
    let shape = VanillaShape::Circle2 { r: 5.0 };
    let circle = Circle2::try_from(shape).unwrap();

    // Circle -> enum
    let shape_again: VanillaShape = circle.into();

    match shape_again {
        VanillaShape::Circle2 { r } => assert_eq!(r, 5.0),
        _ => panic!("expected Circle"),
    }

    let dot = Dot2::new();

    let shape: VanillaShape = dot.into();
    assert!(matches!(shape, VanillaShape::Dot2));
}

#[test]
fn shape_generates_kindless_structs_and_kind() {
    let circle = Circle3::new(5.0);
    assert_eq!(circle.r, 5.0);
    assert!(size_of::<Circle3>() == size_of::<f64>());

    let rect = Rect3::new(10.0, 20.0);
    assert_eq!(rect.w, 10.0);
    assert_eq!(rect.h, 20.0);
    assert!(size_of::<Rect3>() == 2 * size_of::<f64>());

    let tri = Tri3::new(3.0, 4.0, 5.0);
    assert_eq!(tri.c, 5.0);
    assert!(size_of::<Tri3>() == 3 * size_of::<f64>());

    let dot = Dot3::new();
    assert!(size_of_val(&dot) == 0);
    assert!(size_of::<Dot3>() == 0);

    assert_eq!(ShapeKindlessStructKind::Circle3 as u8, 0);
    assert_eq!(ShapeKindlessStructKind::Rect3 as u8, 1);
    assert_eq!(ShapeKindlessStructKind::Tri3 as u8, 2);
    assert_eq!(ShapeKindlessStructKind::Dot3 as u8, 3);
}

#[test]
fn converts_between_shape_and_kindless_structs() {
    // Enum -> Circle
    let shape = ShapeKindlessStruct::Circle3 { r: 5.0 };
    let circle = Circle3::try_from(shape).unwrap();

    // Circle -> enum
    let shape_again: ShapeKindlessStruct = circle.into();

    match shape_again {
        ShapeKindlessStruct::Circle3 { r } => assert_eq!(r, 5.0),
        _ => panic!("expected Circle"),
    }

    let dot = Dot3::new();

    let shape: ShapeKindlessStruct = dot.into();
    assert!(matches!(shape, ShapeKindlessStruct::Dot3));
}
