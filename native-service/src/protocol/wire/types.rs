#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppPrecond {
    Absent,
    Present(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u8,
    pub body: Vec<u8>,
}
