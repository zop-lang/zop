use zop::{backend::mlir_text, frontend::analyze};

#[test]
fn source_reaches_verified_mlir_through_the_public_api() {
    let source = "fn answer -> i64\n    42\n";
    let hir = analyze(source).expect("source should type-check");
    let mlir = mlir_text(&hir).expect("MLIR should verify");

    assert!(mlir.contains("func.func @answer() -> i64"));
}
