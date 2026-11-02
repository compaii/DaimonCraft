//! Small, domain-independent cognition state machine. No identity authority.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;

pub const MAX_TEXT_BYTES: usize = 4096;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Awareness,
    Outcome,
    Hazard,
    Human,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub source: String,
    pub priority: Priority,
    pub text: String,
}

/// Saturation is reported to the caller; accepted human speech is never evicted.
#[derive(Debug)]
pub struct Attention {
    capacity: usize,
    events: VecDeque<Event>,
}

impl Attention {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            events: VecDeque::new(),
        }
    }

    pub fn push(&mut self, event: Event) -> Result<(), &'static str> {
        if event.text.len() > MAX_TEXT_BYTES || event.source.len() > 128 {
            return Err("event too large");
        }
        if event.priority == Priority::Awareness
            && let Some(old) = self
                .events
                .iter_mut()
                .find(|old| old.priority == Priority::Awareness && old.source == event.source)
        {
            *old = event;
            return Ok(());
        }
        if self.events.len() >= self.capacity {
            return Err("attention full; caller retains the event");
        }
        self.events.push_back(event);
        Ok(())
    }

    pub fn take(&mut self) -> Option<Event> {
        let highest = self.events.iter().map(|e| &e.priority).max()?;
        let index = self.events.iter().position(|e| &e.priority == highest)?;
        self.events.remove(index)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub name: String,
    pub args: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Action {
    pub id: u32,
    pub call: ToolCall,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Evidence {
    pub action_id: u32,
    pub success: bool,
    pub observation: Value,
}

#[derive(Clone, Debug)]
pub enum Decision {
    Act(ToolCall),
    Finish,
}

pub trait Model {
    /// Concrete network implementations must enforce their own call deadline.
    fn decide(&mut self, event: &Event, state: &Session) -> Result<Decision, String>;
}

pub trait Body {
    /// Err means outcome unknown, not proof the action did not happen.
    fn execute(&mut self, action: &Action) -> Result<Evidence, String>;
}

/// Bounded working memory, not a DMK replacement or a claim of Matrix admission.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    max_turns: u32,
    turns: u32,
    pending: Option<Action>,
    evidence: Vec<Evidence>,
    finished: bool,
}

impl Session {
    pub fn new(max_turns: u32) -> Result<Self, String> {
        if !(1..=100).contains(&max_turns) {
            return Err("turn budget must be in 1..=100".into());
        }
        Ok(Self {
            max_turns,
            turns: 0,
            pending: None,
            evidence: Vec::new(),
            finished: false,
        })
    }

    pub fn pending(&self) -> Option<&Action> {
        self.pending.as_ref()
    }
    pub fn evidence(&self) -> &[Evidence] {
        &self.evidence
    }
    pub fn turns(&self) -> u32 {
        self.turns
    }

    /// Caller persists returned pending state BEFORE executing the action.
    pub fn prepare(
        &mut self,
        model: &mut impl Model,
        event: &Event,
    ) -> Result<Option<Action>, String> {
        if self.pending.is_some() {
            return Err("pending outcome must be reconciled; no replay".into());
        }
        if self.finished {
            return Err("session finished".into());
        }
        if self.turns >= self.max_turns {
            return Err("turn budget exhausted".into());
        }
        if event.text.len() > MAX_TEXT_BYTES || event.source.len() > 128 {
            return Err("event too large".into());
        }
        // Count failed/uncertain model calls too; the attempt consumed resources.
        self.turns += 1;
        match model.decide(event, self)? {
            Decision::Finish => {
                self.finished = true;
                Ok(None)
            }
            Decision::Act(call) => {
                if call.name.is_empty()
                    || call.name.len() > 128
                    || call.args.to_string().len() > MAX_TEXT_BYTES
                {
                    return Err("invalid tool call size/name".into());
                }
                let action = Action {
                    id: self.turns,
                    call,
                };
                self.pending = Some(action.clone());
                Ok(Some(action))
            }
        }
    }

    /// A correlated observation resolves one pending action; narrative cannot.
    pub fn reconcile(&mut self, evidence: Evidence) -> Result<(), String> {
        let action = self.pending.as_ref().ok_or("no pending action")?;
        if evidence.action_id != action.id {
            return Err("evidence action mismatch".into());
        }
        if evidence.observation.to_string().len() > MAX_TEXT_BYTES {
            return Err("evidence too large".into());
        }
        self.evidence.push(evidence);
        self.pending = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    struct Once {
        calls: usize,
    }
    impl Model for Once {
        fn decide(&mut self, _: &Event, _: &Session) -> Result<Decision, String> {
            self.calls += 1;
            Ok(Decision::Act(ToolCall {
                name: "test".into(),
                args: json!({}),
            }))
        }
    }
    fn event(priority: Priority, text: &str) -> Event {
        Event {
            source: "world".into(),
            priority,
            text: text.into(),
        }
    }
    #[test]
    fn priority_fifo_coalescing_and_backpressure() {
        let mut q = Attention::new(3);
        q.push(event(Priority::Awareness, "old position")).unwrap();
        q.push(event(Priority::Awareness, "new position")).unwrap();
        q.push(event(Priority::Human, "first")).unwrap();
        q.push(event(Priority::Human, "second")).unwrap();
        assert!(
            q.push(event(Priority::Hazard, "retained by producer"))
                .is_err()
        );
        assert_eq!(q.take().unwrap().text, "first");
        assert_eq!(q.take().unwrap().text, "second");
        assert_eq!(q.take().unwrap().text, "new position");
    }
    #[test]
    fn restart_retains_uncertainty_without_new_model_call() {
        let mut state = Session::new(2).unwrap();
        let mut model = Once { calls: 0 };
        let input = event(Priority::Human, "move");
        let action = state.prepare(&mut model, &input).unwrap().unwrap();
        let mut restored: Session =
            serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        assert!(restored.prepare(&mut model, &input).is_err());
        assert_eq!(model.calls, 1);
        assert!(
            restored
                .reconcile(Evidence {
                    action_id: action.id + 1,
                    success: true,
                    observation: json!({})
                })
                .is_err()
        );
        assert!(restored.pending().is_some());
        restored
            .reconcile(Evidence {
                action_id: action.id,
                success: false,
                observation: json!({"blocked": true}),
            })
            .unwrap();
        assert!(restored.prepare(&mut model, &input).unwrap().is_some());
    }
    #[test]
    fn budget_and_evidence_bounds_hold_before_another_action() {
        let mut state = Session::new(1).unwrap();
        let mut model = Once { calls: 0 };
        let input = event(Priority::Human, "move");
        let action = state.prepare(&mut model, &input).unwrap().unwrap();
        assert!(
            state
                .reconcile(Evidence {
                    action_id: action.id,
                    success: true,
                    observation: json!("x".repeat(MAX_TEXT_BYTES))
                })
                .is_err()
        );
        state
            .reconcile(Evidence {
                action_id: action.id,
                success: true,
                observation: json!({}),
            })
            .unwrap();
        assert!(state.prepare(&mut model, &input).is_err());
        assert_eq!(model.calls, 1);
    }
}
