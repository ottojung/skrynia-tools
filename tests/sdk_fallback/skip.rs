use skrynia_tools::sdk::{CopyFailure, ScalarError, skip_input};

use super::support::FakeIo;

#[test]
fn relative_skip_returns_absolute_position_and_clamps() {
    let mut io = FakeIo {
        input: vec![0; 4],
        ..FakeIo::default()
    };
    assert_eq!(skip_input(&mut io, 0, 2), Ok(2));
    assert_eq!(skip_input(&mut io, 0, 9), Ok(4));
    for shift in [-1, -9] {
        let result = skip_input(&mut io, 0, shift);
        assert_eq!(result, Err(CopyFailure::Seek(ScalarError(-4))));
    }
}
