use timecat::*;

#[test]
fn test_go_infinite() {
    let mut engine = Engine::from_fen("8/3R4/p5kp/P1p3p1/6P1/8/6P1/Q6K w - - 0 47").unwrap();
    let _ = engine.go_verbose(&SearchConfig::new_infinite());
}
