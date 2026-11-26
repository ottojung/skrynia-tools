//! Real P2 probe for the ordinary P1-owned callback contract.
#[link(wasm_import_module = "p0")]
unsafe extern "C" {
    fn invoke(ctx: i32, op: i32, value: i32) -> i64;
}

fn call(op: i32, value: i32) -> i64 {
    unsafe { invoke(1, op, value) }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    match call(0, 0) {
        0 => pump(0, 1),
        1 => {
            pump(0, 4);
            assert_eq!(call(6, 0), 0);
            pump(3, 1);
        }
        2 => {
            assert_eq!(call(4, 1), 0);
            // Incomplete frame must fail when the interpreter finishes.
        }
        3 => {
            assert_eq!(call(4, 255), 0);
            let error = call(6, 0);
            assert!(error < 0);
            assert_eq!(call(4, 1), error);
            assert_eq!(call(6, 0), error);
        }
        4 => {
            let code = call(99, 0);
            assert_eq!(call(1, (code & 255) as i32), 0);
        }
        _ => panic!("unknown fixture mode"),
    }
}

fn pump(from: i32, to: i32) {
    loop {
        let byte = call(from, 0);
        if byte == -1 {
            return;
        }
        assert!((0..=255).contains(&byte));
        assert_eq!(call(to, byte as i32), 0);
    }
}
