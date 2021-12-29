use super::error::{PolarResult, RuntimeError};
use super::events::*;
use super::messages::*;
use super::runnable::Runnable;
use super::terms::*;
use super::vm::*;

pub struct Query {
    runnable_stack: Vec<(Box<dyn Runnable>, u64)>, // Tuple of Runnable + call_id.
    vm: PolarVirtualMachine,
    term: Term,
    done: bool,
}

impl Query {
    pub fn new(vm: PolarVirtualMachine, term: Term) -> Self {
        Self {
            runnable_stack: vec![],
            vm,
            term,
            done: false,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_logging_options(&mut self, rust_log: Option<String>, polar_log: Option<String>) {
        self.vm.set_logging_options(rust_log, polar_log);
    }

    /// Runnable lifecycle
    ///
    /// 1. Get Runnable A from the top of the Runnable stack, defaulting to the VM.
    /// 2. If Runnable A emits a Run event containing Runnable B, push Runnable B onto the stack.
    /// 3. Immediately request the next event, which will execute Runnable B.
    /// 4. When Runnable B emits a Done event, pop Runnable B off the stack and return its result as
    ///    an answer to Runnable A.
    pub fn next_event(&mut self) -> PolarResult<QueryEvent> {
        let mut counter = self.vm.id_counter();
        let qe = match self.top_runnable().run(Some(&mut counter)) {
            Ok(e) => e,
            Err(e) => self
                .top_runnable()
                .handle_error(e)
                .map_err(RuntimeError::with_context)?,
        };
        self.recv_event(qe)
    }

    fn recv_event(&mut self, qe: QueryEvent) -> PolarResult<QueryEvent> {
        match qe {
            QueryEvent::None => self.next_event(),
            QueryEvent::Run { runnable, call_id } => {
                self.push_runnable(runnable, call_id);
                self.next_event()
            }
            QueryEvent::Done { result } => {
                if let Some((_, result_call_id)) = self.pop_runnable() {
                    self.top_runnable()
                        .external_question_result(result_call_id, result)
                        .map_err(RuntimeError::with_context)?;
                    self.next_event()
                } else {
                    // VM is done.
                    assert!(self.runnable_stack.is_empty());
                    Ok(QueryEvent::Done { result })
                }
            }
            ev => Ok(ev),
        }
    }

    fn top_runnable(&mut self) -> &mut (dyn Runnable) {
        self.runnable_stack
            .last_mut()
            .map(|b| b.0.as_mut())
            .unwrap_or(&mut self.vm)
    }

    fn push_runnable(&mut self, runnable: Box<dyn Runnable>, call_id: u64) {
        self.runnable_stack.push((runnable, call_id));
    }

    fn pop_runnable(&mut self) -> Option<(Box<dyn Runnable>, u64)> {
        self.runnable_stack.pop()
    }

    pub fn call_result(&mut self, call_id: u64, value: Option<Term>) -> PolarResult<()> {
        self.top_runnable()
            .external_call_result(call_id, value)
            .map_err(RuntimeError::with_context)
    }

    pub fn question_result(&mut self, call_id: u64, result: bool) -> PolarResult<()> {
        self.top_runnable()
            .external_question_result(call_id, result)
            .map_err(RuntimeError::with_context)
    }

    pub fn application_error(&mut self, message: String) -> PolarResult<()> {
        self.vm
            .external_error(message)
            .map_err(RuntimeError::with_context)
    }

    pub fn debug_command(&mut self, command: &str) -> PolarResult<()> {
        self.top_runnable()
            .debug_command(command)
            .map_err(RuntimeError::with_context)
    }

    pub fn next_message(&self) -> Option<Message> {
        self.vm.messages.next()
    }

    pub fn source_info(&self) -> String {
        self.vm.term_source(&self.term, true)
    }

    pub fn bind(&mut self, name: Symbol, value: Term) -> PolarResult<()> {
        self.vm
            .bind(&name, value)
            .map_err(RuntimeError::with_context)
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
