mod commit;
mod run;
use super::{Clock, Delivery, Durable, Outcome, Pending, Policy, Transport};
pub struct Queue<D: Durable> {
    items: Vec<Pending>,
    dur: D,
    policy: Policy,
    pub(super) uncertain: bool,
}
impl<D: Durable> Queue<D> {
    pub fn open(dur: D, policy: Policy) -> Result<Self, &'static str> {
        let items = dur.load().map_err(|_| "load_failed")?;
        let mut seen = std::collections::HashSet::new();
        if items.iter().any(|p| !seen.insert(&p.sub_id)) {
            return Err("invalid_queue");
        }
        Ok(Self {
            items,
            dur,
            policy,
            uncertain: false,
        })
    }
    pub fn due(&self, now_ms: u64) -> Result<Vec<Delivery>, &'static str> {
        self.ready()?;
        Ok(self
            .items
            .iter()
            .filter(|p| p.next_ms <= now_ms)
            .map(|p| Delivery {
                sub_id: p.sub_id.clone(),
                endpoint: p.endpoint.clone(),
            })
            .collect())
    }
    pub fn pending(&self) -> Result<&[Pending], &'static str> {
        self.ready()?;
        Ok(&self.items)
    }
    pub(super) fn ready(&self) -> Result<(), &'static str> {
        if self.uncertain {
            Err("state_uncertain")
        } else {
            Ok(())
        }
    }
}
