const CAP: usize = 64;
#[derive(Clone, Copy)]
struct Slot {
    real: i32,
    age: u16,
    live: bool,
}
pub struct Table {
    slots: [Slot; CAP],
}
impl Table {
    pub fn new() -> Self {
        Self {
            slots: [Slot {
                real: -1,
                age: 0,
                live: false,
            }; CAP],
        }
    }
    pub fn insert(&mut self, real: i32) -> Option<i32> {
        for (i, s) in self.slots.iter_mut().enumerate() {
            if s.live {
                continue;
            }
            s.real = real;
            s.live = true;
            s.age = s.age.wrapping_add(1).max(1);
            return Some(((i as u32) << 16 | s.age as u32) as i32);
        }
        None
    }
    pub fn get(&self, h: i32) -> Option<i32> {
        let (i, g) = ((h as u32 >> 16) as usize, (h as u32 & 0xffff) as u16);
        let s = self.slots.get(i)?;
        (s.live && s.age == g).then_some(s.real)
    }
    pub fn take(&mut self, h: i32) -> Option<i32> {
        let (i, g) = ((h as u32 >> 16) as usize, (h as u32 & 0xffff) as u16);
        let s = self.slots.get_mut(i)?;
        if s.live && s.age == g {
            s.live = false;
            Some(s.real)
        } else {
            None
        }
    }
}
