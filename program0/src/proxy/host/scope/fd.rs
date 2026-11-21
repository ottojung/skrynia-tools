const CAP: usize = 64;
#[derive(Clone, Copy)]
struct Slot(i32, u16, bool, bool); // real, age, live, dead
pub struct Table {
    slots: [Slot; CAP],
}
impl Table {
    pub fn new() -> Self {
        Self {
            slots: [Slot(-1, 0, false, false); CAP],
        }
    }
    pub fn insert(&mut self, real: i32) -> Option<i32> {
        for (i, s) in self.slots.iter_mut().enumerate() {
            if s.2 || s.3 {
                continue;
            }
            let next = s.1.wrapping_add(1).max(1);
            if next == 1 && s.1 != 0 {
                s.3 = true;
                continue;
            }
            *s = Slot(real, next, true, false);
            return Some(((i as u32) << 16 | next as u32) as i32);
        }
        None
    }
    pub fn get(&self, h: i32) -> Option<i32> {
        let (i, g) = ((h as u32 >> 16) as usize, (h as u32 & 0xffff) as u16);
        let s = self.slots.get(i)?;
        (s.2 && s.1 == g).then_some(s.0)
    }
    pub fn take(&mut self, h: i32) -> Option<i32> {
        let (i, g) = ((h as u32 >> 16) as usize, (h as u32 & 0xffff) as u16);
        let s = self.slots.get_mut(i)?;
        if s.2 && s.1 == g {
            s.2 = false;
            Some(s.0)
        } else {
            None
        }
    }
}
