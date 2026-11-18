#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Usage {
    pub bytes: u64,
    pub objects: u64,
}
impl Usage {
    pub fn object(size: u64) -> Option<Self> {
        Some(Self {
            bytes: size.checked_add(512)?,
            objects: 1,
        })
    }
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        Some(Self {
            bytes: self.bytes.checked_add(rhs.bytes)?,
            objects: self.objects.checked_add(rhs.objects)?,
        })
    }
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        Some(Self {
            bytes: self.bytes.checked_sub(rhs.bytes)?,
            objects: self.objects.checked_sub(rhs.objects)?,
        })
    }
}
