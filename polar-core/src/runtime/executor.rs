use std::sync::{Arc, MutexGuard};

use smol;

use crate::{
    vm::PolarVirtualMachine,
    error::PolarResult,
    events::QueryEvent,
    runtime::Host, async_vm::AsyncVm, kb::KnowledgeBase, terms::Term
};


/// Execute the VM, using host in the current thread.
pub struct LocalExecutor {
    vm: Arc<AsyncVm>,
    host: Arc<Host>,
    run_spawned: bool,
    runtime: smol::LocalExecutor<'static>,
    done: bool,
    last_event_vm: bool
}

impl LocalExecutor {
    pub fn new(vm: PolarVirtualMachine) -> Self {
        let host = vm.host().clone();
        let vm = Arc::new(AsyncVm::new(vm, host.clone()));
        Self { vm, host, run_spawned: false, runtime: smol::LocalExecutor::new(), done: false, last_event_vm: false }
    }

    pub fn host(&self) -> &Arc<Host> {
        &self.host
    }

    /// Get the next event from the VM or Host.
    /// Includes FFI Events (ExternalIsa) & VM Events (Done)
    pub fn next_event(&mut self) -> PolarResult<QueryEvent> {
        let mut _next_event = || {
            loop {
                if !self.run_spawned {
                    let vm = self.vm.clone();
                    self.runtime.spawn(async move {
                        let r = vm.run(None).await;
                        eprintln!("spawn done {:?}", r);
                    }).detach();
                    self.run_spawned = true;
                }

                eprintln!("tick");
                let more = self.runtime.try_tick();
                let host_event = self.host.next_event();
                if let Some(host_event) = host_event {
                    eprintln!("host event {:?}", host_event);
                    self.last_event_vm = false;
                    return host_event;
                }

                if let Some(vm_event) = self.vm.try_take_ev() {
                    self.run_spawned = false;
                    self.last_event_vm = true;
                    eprintln!("vm event {:?}", vm_event);
                    return vm_event.map_err(|e| self.vm.with_kb(|kb| e.with_context(kb)));
                }

                assert!(more);
            }
        };

        match _next_event() {
            e @ Ok(QueryEvent::Done { .. }) => {
                self.done = true;
                e
            }
            e => e
        }
    }

    pub fn vm(&self) -> MutexGuard<PolarVirtualMachine> {
        self.vm.vm()
    }

    pub fn with_kb<F, R>(&self, f: F) -> R
    where F: FnOnce(&KnowledgeBase) -> R 
    {
        self.vm.with_kb(f)
    }
}

impl std::iter::Iterator for LocalExecutor {
    type Item = PolarResult<QueryEvent>;
   
    fn next(&mut self) -> Option<Self::Item> {
        if !self.done {
            Some(self.next_event())
        } else {
            None
        }
    }
}