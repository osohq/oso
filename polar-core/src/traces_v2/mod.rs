use std::convert::TryFrom;
/// Traces v2. Better traces.
use std::option::Option;

use serde::Serialize;

use crate::bindings::Bindings;
use crate::formatting::ToPolarString;
use crate::sources::Source;
use crate::terms::Term;
use crate::vm;

/// Top level event for trace containing common fields.
#[derive(Clone, Serialize)]
pub struct Event {
    /// Trace timestamp.
    timestamp_ms: u128,
    id: u64,
    parent_id: u64,

    #[serde(flatten)]
    event_type: EventDetail,
}

impl Event {
    pub fn execute_goal(goal: vm::Goal, source: Option<Source>) -> Self {
        let goal = Goal::try_from(goal).unwrap();
        Event {
            timestamp_ms: _timestamp_ms(),
            id: 0,
            parent_id: 0,
            event_type: EventDetail::ExecuteGoal { goal, source },
        }
    }
    pub fn evaluate_rule(rule: String, source: Option<Source>) -> Self {
        Event {
            timestamp_ms: _timestamp_ms(),
            id: 0,
            parent_id: 0,
            event_type: EventDetail::EvaluateRule { rule, source },
        }
    }

    pub fn backtrack(reason: String) -> Self {
        Event {
            timestamp_ms: _timestamp_ms(),
            id: 0,
            parent_id: 0,
            event_type: EventDetail::Backtrack { reason },
        }
    }

    pub fn execute_choice() -> Self {
        Event {
            timestamp_ms: _timestamp_ms(),
            id: 0,
            parent_id: 0,
            event_type: EventDetail::ExecuteChoice {},
        }
    }

    pub fn choice_push() -> Self {
        Event {
            timestamp_ms: _timestamp_ms(),
            id: 0,
            parent_id: 0,
            event_type: EventDetail::ChoicePush {},
        }
    }

    pub fn bindings(bindings: Bindings) -> Self {
        Event {
            timestamp_ms: _timestamp_ms(),
            id: 0,
            parent_id: 0,
            event_type: EventDetail::Bindings { bindings },
        }
    }

    pub fn result(bindings: Bindings) -> Self {
        Event {
            timestamp_ms: _timestamp_ms(),
            id: 0,
            parent_id: 0,
            event_type: EventDetail::Result { bindings },
        }
    }

    pub fn done() -> Self {
        Event {
            timestamp_ms: _timestamp_ms(),
            id: 0,
            parent_id: 0,
            event_type: EventDetail::Done,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(tag = "goal_type")]
pub enum Goal {
    Query {
        term: Term,
        polar: String,
    },
    CheckApplicableRule {
        rule: crate::rules::Rule,
        args: Vec<Term>,
        polar: String,
    },
}

impl TryFrom<vm::Goal> for Goal {
    type Error = ();

    fn try_from(other: vm::Goal) -> Result<Self, ()> {
        match other {
            vm::Goal::Query { term } => {
                let polar = term.to_polar();
                Ok(Goal::Query { term, polar })
            }
            vm::Goal::FilterRules {
                args,
                unfiltered_rules,
                ..
            } => {
                let rule = unfiltered_rules.last().unwrap();
                let polar = rule.to_polar();
                Ok(Goal::CheckApplicableRule {
                    args,
                    rule: rule.as_ref().clone(),
                    polar,
                })
            }
            _ => Err(()),
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(tag = "event_type")]
pub enum EventDetail {
    /// Emitted when choices are pushed by the VM.
    ChoicePush {},

    /// Emitted when a goal is executed.
    ExecuteGoal { goal: Goal, source: Option<Source> },

    /// Emitted when evaluation of a rule begins
    EvaluateRule {
        rule: String,
        source: Option<Source>,
    },

    /// Emitted when a choice starts executing.
    ExecuteChoice {},

    /// Emitted when bindings are changed.
    Bindings { bindings: Bindings },

    /// Emitted on a backtrack.
    Backtrack { reason: String },

    /// Emitted on a result.
    Result { bindings: Bindings },

    /// Emitted on VM done.
    Done,
}

/// Use to record traces.
#[derive(Clone, Default)]
pub struct Recorder {
    events: Vec<Event>,
    next_id: u64,
}

impl Recorder {
    fn push(&mut self, mut event: Event) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        event.id = id;
        self.events.push(event);

        id
    }

    fn events(&self) -> &Vec<Event> {
        &self.events
    }

    fn into_events(self) -> Vec<Event> {
        self.events
    }
}

#[derive(Clone, Default)]
/// A recorder that writes traces to a parent, with a particular id.
pub struct ScopedRecorder {
    recorder: Recorder,
    parent_id: Vec<u64>,
}

impl ScopedRecorder {
    pub fn new(recorder: Recorder) -> Self {
        ScopedRecorder {
            recorder,
            parent_id: vec![],
        }
    }

    fn parent_id(&self) -> u64 {
        self.parent_id.last().cloned().unwrap_or_default()
    }

    pub fn push_parent_id(&mut self, id: u64) {
        self.parent_id.push(id);
    }

    pub fn push_parent(&mut self, mut event: Event) -> u64 {
        let id = self.push(event);
        dbg!("push_parent", id, self.parent_id());
        self.parent_id.push(id);
        id
    }

    pub fn push(&mut self, mut event: Event) -> u64 {
        event.parent_id = self.parent_id();
        dbg!("push", event.id, event.parent_id);
        self.recorder.push(event)
    }

    pub fn pop(&mut self) {
        self.parent_id.pop();
    }

    pub fn pop_to(&mut self, target: u64) {
        loop {
            dbg!("pop_to", &self.parent_id, target);
            let id = self.parent_id.pop().unwrap();
            if id == target {
                return;
            }
        }
    }

    pub fn pop_up_to(&mut self, target: u64) {
        loop {
            dbg!("pop_up_to", &self.parent_id, target);
            let id = self.parent_id.last().unwrap();
            if id == &target {
                return;
            }

            self.parent_id.pop();
        }
    }

    pub fn into_recorder(self) -> Recorder {
        self.recorder
    }

    pub fn events(&self) -> &Vec<Event> {
        self.recorder.events()
    }
}

fn _timestamp_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
