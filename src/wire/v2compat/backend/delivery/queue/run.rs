use super::{Clock, Durable, Outcome, Queue, Transport};
impl<D: Durable> Queue<D> {
    pub fn run<C: Clock, T: Transport>(
        &mut self,
        clock: &C,
        tx: &mut T,
    ) -> Result<(), &'static str> {
        let now = clock.now_ms();
        let ids: Vec<String> = self.due(now)?.into_iter().map(|p| p.sub_id).collect();
        for id in ids {
            self.step(&id, now, tx)?;
        }
        Ok(())
    }
    pub(super) fn step<T: Transport>(
        &mut self,
        id: &str,
        now: u64,
        tx: &mut T,
    ) -> Result<(), &'static str> {
        self.ready()?;
        let Some(i) = self.items.iter().position(|p| p.sub_id == id) else {
            return Ok(());
        };
        let (endpoint, attempts) = (self.items[i].endpoint.clone(), self.items[i].attempts);
        let outcome = tx.send(id, &endpoint);
        let mut next = self.items.clone();
        match outcome {
            Outcome::Ack | Outcome::Gone => {
                next.remove(i);
            }
            Outcome::Retry => {
                let a = attempts.saturating_add(1);
                if a >= self.policy.max_failures {
                    next.remove(i);
                } else {
                    next[i].attempts = a;
                    next[i].next_ms = now.saturating_add(self.policy.backoff(a));
                }
            }
        }
        self.commit(next)
    }
}
