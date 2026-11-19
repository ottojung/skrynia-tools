use super::{Clock, Durable, Outcome, Queue, Transport};
impl<D: Durable> Queue<D> {
    pub fn run<C: Clock, T: Transport>(&mut self, clock: &C, tx: &mut T) {
        let now = clock.now_ms();
        let ids: Vec<String> = self
            .items
            .iter()
            .filter(|p| p.next_ms <= now)
            .map(|p| p.sub_id.clone())
            .collect();
        for id in ids {
            self.step(&id, now, tx);
        }
    }
    pub(super) fn step<T: Transport>(&mut self, id: &str, now: u64, tx: &mut T) {
        let Some(i) = self.items.iter().position(|p| p.sub_id == id) else {
            return;
        };
        let (endpoint, attempts) = (self.items[i].endpoint.clone(), self.items[i].attempts);
        match tx.send(id, &endpoint) {
            Outcome::Ack | Outcome::Gone => {
                self.items.remove(i);
            }
            Outcome::Retry => {
                let a = attempts.saturating_add(1);
                if a >= self.policy.max_failures {
                    self.items.remove(i);
                } else {
                    self.items[i].attempts = a;
                    self.items[i].next_ms = now.saturating_add(self.policy.backoff(a));
                }
            }
        }
        self.dur.save(&self.items);
    }
}
