pub fn ops() -> Vec<u8> {
    vec![
        5, 8, 10, 4, // mkdir user/a, sync root
        5, 8, 10, 4, // mkdir user/a/b, sync user/a
        5, 8, 10, 4, // mkdir user/a/b/c, sync user/a/b
        11, 1, 3, 10, 10, 4, // random, open, write, file sync twice, close
        7, 8, 10, 4, // rename, leaf dir sync
    ]
}
