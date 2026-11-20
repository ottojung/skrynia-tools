pub fn echo() -> Vec<u8> {
    wat::parse_str(
        r#"(module
(import "skrynia" "read" (func $read(param i32 i32 i32)(result i32)))
(import "skrynia" "write" (func $write(param i32 i32 i32)(result i32)))
(memory(export "memory") 1)
(func(export "skrynia_main")(param $input i32)(param $output i32)(result i32)
 (local $n i32)(local $sent i32)(local $wrote i32)
 (block $eof (loop $read_more
  (local.set $n (call $read(local.get $input)(i32.const 0)(i32.const 65536)))
  (if (i32.lt_s (local.get $n) (i32.const 0))
   (then(return(local.get $n))))
  (br_if $eof (i32.eqz(local.get $n)))
  (local.set $sent (i32.const 0))
  (block $done (loop $write_more
   (br_if $done (i32.ge_s(local.get $sent)(local.get $n)))
   (local.set $wrote(call $write(local.get $output)(local.get $sent)
    (i32.sub(local.get $n)(local.get $sent))))
   (if (i32.le_s(local.get $wrote)(i32.const 0))
    (then(return(i32.const -5))))
   (local.set $sent(i32.add(local.get $sent)(local.get $wrote)))
   (br $write_more)))
  (br $read_more)))
 (i32.const 0)))"#,
    )
    .unwrap()
}
pub fn privileged() -> Vec<u8> {
    wat::parse_str(
        r#"(module
      (import "skrynia" "open_disk" (func $disk (param i32 i32 i32) (result i32)))
      (memory (export "memory") 1)
      (func (export "skrynia_main") (param i32 i32) (result i32)
        (call $disk (i32.const 0) (i32.const 0) (i32.const 1))))"#,
    )
    .unwrap()
}
pub fn read_disk() -> Vec<u8> {
    wat::parse_str(include_str!("wat/read_disk.wat")).unwrap()
}
pub fn write_disk() -> Vec<u8> {
    wat::parse_str(include_str!("wat/write_disk.wat")).unwrap()
}
pub fn durable() -> Vec<u8> {
    wat::parse_str(include_str!("wat/durable.wat")).unwrap()
}
pub fn nested_router() -> Vec<u8> {
    wat::parse_str(include_str!("wat/router.wat")).unwrap()
}
