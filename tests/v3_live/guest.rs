pub fn byte(value: u8) -> Vec<u8> {
    let code = format!(
        r#"(module (memory (export "memory") 1)
      (func (export "skrynia_main") (param i32 i32 i32 i32) (result i32)
      (i32.store8 (local.get 2) (i32.const {value})) (i32.const 1)))"#
    );
    wat::parse_str(code).unwrap()
}

pub fn callback_isolation_probe() -> Vec<u8> {
    // Guest ignores input app id and tries to AppGet a different app (all 0x09).
    // Writes protocol response status byte (0=OK,1=ERR) to output.
    let wat = r#"(module
  (import "skrynia" "invoke" (func $invoke (param i32 i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "\01\07\20\00\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\09\01\00\00\00k")
  (func (export "skrynia_main") (param i32 i32 i32 i32) (result i32)
    (local $n i32)
    (local.set $n (call $invoke (i32.const 0) (i32.const 41) (i32.const 64) (i32.const 64)))
    (if (i32.lt_s (local.get $n) (i32.const 2)) (then (return (i32.const -1))))
    (i32.store8 (local.get 2) (i32.load8_u (i32.const 65)))
    (i32.const 1))
)"#;
    wat::parse_str(wat).unwrap()
}
