use skrynia_tools::wire::v2compat::{ObjectRead, Store};
use skrynia_v2_provider::{Callback, InvokeBackend, InvokeError};
struct FileError(i32);
impl Callback for FileError {
    fn invoke(&self, req: &[u8]) -> Result<Vec<u8>, InvokeError> {
        if req.first() != Some(&1) {
            return Err(InvokeError::Unavailable);
        }
        Ok(self.0.to_le_bytes().to_vec())
    }
}
#[test]
fn actual_backend_distinguishes_missing_file_from_disk_access_error() {
    let missing = FileError(-2);
    let backend = InvokeBackend { invoke: &missing };
    assert!(matches!(
        backend.read_object("team/new"),
        Ok(ObjectRead::Missing)
    ));
    for error in [-1, -3, -4, -7, -10] {
        let denied = FileError(error);
        let backend = InvokeBackend { invoke: &denied };
        assert!(backend.read_object("team/new").is_err());
    }
}
