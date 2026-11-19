mod run;
use super::{Clock, Delivery, Durable, Outcome, Pending, Policy, Transport};
pub struct Queue<D: Durable> {
    pub items: Vec<Pending>,
    pub dur: D,
    pub policy: Policy,
}
impl<D: Durable> Queue<D> {
    pub fn open(dur: D, policy: Policy) -> Self {
        Self {
            items: dur.load(),
            dur,
            policy,
        }
    }
    pub fn enqueue(&mut self, sub_id: String, endpoint: String) {
        if self.items.iter().any(|p| p.sub_id == sub_id) {
            return;
        }
        self.items.push(Pending {
            sub_id,
            endpoint,
            attempts: 0,
            next_ms: 0,
        });
        self.dur.save(&self.items);
    }
    pub fn due(&self, now_ms: u64) -> Vec<Delivery> {
        self.items
            .iter()
            .filter(|p| p.next_ms <= now_ms)
            .map(|p| Delivery {
                sub_id: p.sub_id.clone(),
                endpoint: p.endpoint.clone(),
            })
            .collect()
    }
    pub fn remove(&mut self, sub_id: &str) {
        let n = self.items.len();
        self.items.retain(|p| p.sub_id != sub_id);
        if self.items.len() != n {
            self.dur.save(&self.items);
        }
    }
}
