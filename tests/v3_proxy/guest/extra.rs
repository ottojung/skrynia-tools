pub fn nested_router() -> Vec<u8> {
    wat::parse_str(include_str!("../wat/router.wat")).unwrap()
}
pub fn mutation_lock() -> Vec<u8> {
    wat::parse_str(include_str!("../wat/mutation_lock.wat")).unwrap()
}
pub fn mutation_trap() -> Vec<u8> {
    wat::parse_str(include_str!("../wat/mutation_trap.wat")).unwrap()
}
pub fn forbidden_lock_import() -> Vec<u8> {
    wat::parse_str(
        r#"(module
         (import "skrynia" "try_mutation_lock" (func $lock(result i32)))
         (memory(export "memory") 1)
         (func(export "skrynia_main")(param i32 i32)(result i32)
           (call $lock)))"#,
    )
    .unwrap()
}
