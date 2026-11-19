use super::{Durable, Pending, Queue};
impl<D: Durable> Queue<D> {
    pub(super) fn commit(&mut self, next: Vec<Pending>) -> Result<(), &'static str> {
        self.ready()?;
        if self.dur.save(&next).is_err() {
            self.uncertain = true;
            return Err("state_uncertain");
        }
        self.items = next;
        Ok(())
    }
    pub fn enqueue(&mut self, sub_id: String, endpoint: String) -> Result<(), &'static str> {
        self.ready()?;
        if self.items.iter().any(|p| p.sub_id == sub_id) {
            return Ok(());
        }
        let mut next = self.items.clone();
        next.push(Pending {
            sub_id,
            endpoint,
            attempts: 0,
            next_ms: 0,
        });
        self.commit(next)
    }
    pub fn remove(&mut self, sub_id: &str) -> Result<(), &'static str> {
        self.ready()?;
        let mut next = self.items.clone();
        next.retain(|p| p.sub_id != sub_id);
        if next.len() == self.items.len() {
            return Ok(());
        }
        self.commit(next)
    }
}
