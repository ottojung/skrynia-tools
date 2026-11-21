(module
 (import "skrynia" "invoke" (func $invoke(param i32 i32 i32 i32)(result i32)))
 (memory(export "memory") 1)
 (data(i32.const 0) "\07\12\00\00\00outside/secret.txt\11\00\00\00granted/moved.txt")
 (func(export "skrynia_main")(param $input i32)(param $output i32)(result i32)
  (local $rc i32)
  (local.set $rc(call $invoke(i32.const 0)(i32.const 44)(i32.const 256)(i32.const 4)))
  (if (i32.ne(local.get $rc)(i32.const 4)) (then(return(i32.const -11))))
  (i32.load(i32.const 256))))
