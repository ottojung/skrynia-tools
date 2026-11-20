pub fn echo() -> Vec<u8> {
    wat::parse_str(
        r#"(module (memory (export "memory") 1)
      (func (export "skrynia_main") (param i32 i32 i32 i32) (result i32)
        (memory.copy (local.get 2) (local.get 0) (local.get 1))
        (local.get 1)))"#,
    )
    .unwrap()
}
pub fn privileged() -> Vec<u8> {
    wat::parse_str(
        r#"(module
      (import "skrynia" "open_disk" (func $disk (param i32 i32 i32) (result i32)))
      (memory (export "memory") 1)
      (func (export "skrynia_main") (param i32 i32 i32 i32) (result i32)
        (call $disk (i32.const 0) (i32.const 0) (i32.const 1))))"#,
    )
    .unwrap()
}
