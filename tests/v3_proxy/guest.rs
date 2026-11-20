pub fn echo() -> Vec<u8> {
    let wat = r#"(module
      (import "skrynia" "read" (func $r (param i32 i32 i32)(result i32)))
      (import "skrynia" "write" (func $w (param i32 i32 i32)(result i32)))
      (memory (export "memory") 1)
      (func (export "skrynia_main") (param $i i32)(param $o i32)(result i32)
        (local $n i32)(local $d i32)(local $x i32)
        (block $done (loop $read
          (local.set $n (call $r (local.get $i)(i32.const 0)(i32.const 4096)))
          (br_if $done (i32.eqz (local.get $n)))
          (if (i32.lt_s (local.get $n)(i32.const 0))
            (then (return (local.get $n))))
          (local.set $d (i32.const 0))
          (block $written (loop $write
            (br_if $written (i32.eq (local.get $d)(local.get $n)))
            (local.set $x (call $w (local.get $o)(local.get $d)
               (i32.sub (local.get $n)(local.get $d))))
            (if (i32.le_s (local.get $x)(i32.const 0))
              (then (return (i32.const -5))))
            (local.set $d (i32.add (local.get $d)(local.get $x)))
            (br $write)))
          (br $read)))
        (i32.const 0)))"#;
    wat::parse_str(wat).unwrap()
}
pub fn privileged() -> Vec<u8> {
    wat::parse_str(
        r#"(module
      (import "skrynia" "open_disk" (func $d (param i32 i32 i32)(result i32)))
      (memory(export "memory") 1)
      (func (export "skrynia_main") (param i32 i32)(result i32)
        (call $d (i32.const 0)(i32.const 0)(i32.const 1))))"#,
    )
    .unwrap()
}
