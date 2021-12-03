use std::sync::Arc;

use crate::error::RuntimeError;
use crate::runtime::Host;
use crate::traces::Node;

use super::error::PolarResult;
use super::events::*;
use super::messages::*;
use super::terms::*;
use super::vm::*;
use crate::runtime::executor::LocalExecutor;

pub struct Query {
    term: Term,
    done: bool,
    runtime: LocalExecutor,
}

impl Query {
    pub fn new(vm: PolarVirtualMachine, term: Term) -> Self {
        let runtime = LocalExecutor::new(vm);

        Self {
            runtime,
            term,
            done: false,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_logging_options(&mut self, rust_log: Option<String>, polar_log: Option<String>) {
        self.vm.set_logging_options(rust_log, polar_log);
    }

    pub fn next_event(&mut self) -> PolarResult<QueryEvent> {
        self.async_next_event()
    }

    fn async_next_event(&mut self) -> PolarResult<QueryEvent> {
        self.runtime.next_event()
    }

    pub fn call_result(&mut self, call_id: u64, value: Option<Term>) -> PolarResult<()> {
        self.runtime.host().external_call_result(call_id, value)
    }

    pub fn question_result(&mut self, call_id: u64, result: bool) -> PolarResult<()> {
        self.runtime.host().external_question_result(call_id, result)
    }

    pub fn application_error(&mut self, call_id: u64, msg: String) -> PolarResult<()> {
        self.runtime.host().application_error(
            call_id,
            RuntimeError::Application {
                msg,
                stack_trace: "".into(),
                term: None,
            },
        )
    }

    pub fn debug_command(&mut self, _command: &str) -> PolarResult<()> {
        unimplemented!("throw");
    }

    pub fn next_message(&self) -> Option<Message> {
        self.runtime.next_msg()
    }

    pub fn source_info(&self) -> String {
        self.runtime.vm().term_source(&self.term, true)
    }

    pub fn bind(&mut self, name: Symbol, value: Term) -> PolarResult<()> {
        self.runtime.vm()
            .bind(&name, value)
            .map_err(|e| self.runtime.with_kb(|kb| e.with_context(kb)))
    }
}

// Query as an iterator returns `None` after the first time `Done` is seen
impl Iterator for Query {
    type Item = PolarResult<QueryEvent>;

    fn next(&mut self) -> Option<PolarResult<QueryEvent>> {
        if self.done {
            return None;
        }
        let event = self.next_event();
        if let Ok(QueryEvent::Done { .. }) = event {
            self.done = true;
        }
        Some(event)
    }
}
