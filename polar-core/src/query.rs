use std::sync::Arc;

use crate::runtime::Host;

use super::error::PolarResult;
use super::events::*;
use super::messages::*;
use super::terms::*;
use super::vm::*;
use crate::async_vm::AsyncVm;

use smol::LocalExecutor;

pub struct Query {
    vm: Arc<AsyncVm>,
    host: Arc<Host>,
    term: Term,
    done: bool,
    runtime: LocalExecutor<'static>,
    run_spawned: bool
}

impl Query {
    pub fn new(vm: PolarVirtualMachine, term: Term) -> Self {
        let host = Arc::new(Host::new());
        let async_vm = Arc::new(AsyncVm::new(vm, host.clone()));

        Self {
            vm: async_vm,
            term,
            host,
            done: false,
            runtime: LocalExecutor::new(),
            run_spawned: false
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_logging_options(&mut self, rust_log: Option<String>, polar_log: Option<String>) {
        self.vm.set_logging_options(rust_log, polar_log);
    }

    pub fn next_event(&mut self) -> PolarResult<QueryEvent> {
        return self.async_next_event();
    }

    fn async_next_event(&mut self) -> PolarResult<QueryEvent> {
        loop {
            if !self.run_spawned {
                let vm = self.vm.clone();
                eprintln!("spawn");
                self.runtime.spawn(async move {
                    let r = vm.run(None).await;
                    eprintln!("fut res: {:?}", r);
                }).detach();
                self.run_spawned = true;
            }

            eprintln!("tick");
            let more = self.runtime.try_tick();
            let ev = self.host.next_event();
            if let Some(ev) = ev {
                eprintln!("host event");
                return ev;
            }

            if let Some(ev) = self.vm.try_take_ev() {
                eprintln!("vm event {:?}", ev);
                self.run_spawned = false;
                return ev.map_err(|e| self.vm.with_kb(|kb| e.with_context(kb)))
            }

            assert!(more);
        }
    }

    pub fn call_result(&mut self, call_id: u64, value: Option<Term>) -> PolarResult<()> {
        self.host.external_call_result(call_id, value)
    }

    pub fn question_result(&mut self, call_id: u64, result: bool) -> PolarResult<()> {
        self.host.external_question_result(call_id, result)
    }

    pub fn application_error(&mut self, message: String) -> PolarResult<()> {
        self.host.application_error(message);
        Ok(())
    }

    pub fn debug_command(&mut self, command: &str) -> PolarResult<()> {
        unimplemented!("throw");
    }

    pub fn next_message(&self) -> Option<Message> {
        self.vm.next_msg()
    }

    pub fn source_info(&self) -> String {
        self.vm.term_source(&self.term, true)
    }

    pub fn bind(&mut self, name: Symbol, value: Term) -> PolarResult<()> {
        self.vm
            .bind(&name, value)
            .map_err(|e| self.vm.with_kb(|kb| e.with_context(kb)))
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
