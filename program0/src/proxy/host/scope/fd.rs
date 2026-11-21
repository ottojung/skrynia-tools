const CAP: usize = 64;
#[derive(Clone, Copy)]
struct Slot {
    real: i32,
    age: u16,
    live: bool,
    dead: bool,
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
                dead: false,
            }; CAP],
        }
    }
    pub fn insert(&mut self, real: i32) -> Option<i32> {
        for (i, s) in self.slots.iter_mut().enumerate() {
            if s.live || s.dead {
                continue;
            }
            let next = s.age.wrapping_add(1).max(1);
            if next == 1 && s.age != 0 {
                s.dead = true;
                continue;
            }
            s.real = real;
            s.live = true;
            s.age = next;
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
#[cfg(test)]
mod tests {
    use super::Table;
    #[test]
    fn recycle_65k_without_stale_alias() {
        let mut t = Table::new();
        let h1 = t.insert(10).unwrap();
        for _ in 0..65_535u32 {
            let h = t.insert(20).unwrap();
            assert_eq!(t.take(h), Some(20));
        }
        assert_eq!(t.get(h1), Some(10));
        assert_eq!(t.take(h1), Some(10));
        assert_eq!(t.get(h1), None);
    }
}
