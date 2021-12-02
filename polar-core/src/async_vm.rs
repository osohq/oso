use crate::counter::Counter;
use crate::events::QueryEvent;
use crate::terms::{Term, Symbol};
use crate::vm::PolarVirtualMachine;
use crate::messages::Message;
use crate::runtime::Host;
use crate::kb::KnowledgeBase;
use std::sync::{Mutex, Arc};
use std::cell::Cell;

pub struct AsyncVm {
    vm: Mutex<PolarVirtualMachine>,
    host: Arc<Host>,
    sync_result: Cell<Option<Result<QueryEvent, crate::error::RuntimeError>>>
}

impl AsyncVm {
    pub fn new(vm: PolarVirtualMachine, host: Arc<Host>) -> Self {
        Self { vm: Mutex::new(vm), host, sync_result: Cell::new(None) }
    }

    pub fn with_kb<F, R>(&self, f: F) -> R
    where F: FnOnce(&KnowledgeBase) -> R {
        f(&self.vm.lock().unwrap().kb())
    }

    pub fn bind(&self, var: &Symbol, val: Term) -> Result<(), crate::error::RuntimeError> {
        self.vm.lock().unwrap().bind(var, val)
    }

    pub fn next_msg(&self) -> Option<Message> {
        self.vm.lock().unwrap().messages.next()
    }

    pub fn term_source(&self, term: &Term, include_info: bool) -> String {
        self.vm.lock().unwrap().term_source(term, include_info)
    }

    pub async fn run(&self, _: Option<&mut Counter>) -> Result<(), crate::error::RuntimeError> {
        use crate::events::QueryEvent::*;

        loop {
            let ev = {
                match self.vm.lock().unwrap().run(std::option::Option::None) {
                    Ok(ev) => ev,
                    Err(e) => {
                        self.sync_result.set(Some(Err(e.clone())));
                    eprintln!("done");
                        return Err(e)
                    }
                }
            };
            eprintln!("async event {:?}", ev);
            match ev {
                None | Done { .. } | Result { .. } => {
                    self.sync_result.set(Some(Ok(ev)));
                    eprintln!("done");
                    return Ok(());
                },
                Debug { message } => self.host.debug(message).await,
                MakeExternal { instance_id, constructor } => self.host.make_external(instance_id, constructor).await,
                ExternalCall { call_id, instance, attribute, args, kwargs } => {
                    let result = self.host.external_call(call_id, instance, attribute, args, kwargs).await;
                    self.vm.lock().unwrap().external_call_result(call_id, result.unwrap()).unwrap();
                },
                ExternalIsa { call_id, instance, class_tag } => {
                    let result = self.host.external_isa(call_id, instance, class_tag).await;
                    self.vm.lock().unwrap().external_question_result(call_id, result)?;
                },
                ExternalIsaWithPath { call_id, base_tag, path, class_tag } => {
                    let result = self.host.external_isa_with_path(call_id, base_tag, path, class_tag).await;
                    self.vm.lock().unwrap().external_question_result(call_id, result.unwrap()).unwrap();
                },
                ExternalIsSubSpecializer { call_id, instance_id, left_class_tag, right_class_tag } => {
                    let result = self.host.external_is_sub_specializer(call_id, instance_id, left_class_tag, right_class_tag).await;
                    self.vm.lock().unwrap().external_question_result(call_id, result).unwrap();
                }
                ExternalIsSubclass { call_id, left_class_tag, right_class_tag } => {
                    let result = self.host.external_is_subclass(call_id, left_class_tag, right_class_tag).await;
                    self.vm.lock().unwrap().external_question_result(call_id, result).unwrap();
                },
                ExternalOp { call_id, operator, args } => {
                    let result = self.host.external_op(call_id, operator, args).await;
                    self.vm.lock().unwrap().external_question_result(call_id, result).unwrap();
                }
                NextExternal { .. }  => unimplemented!("Not impl")
            }
        }
        eprintln!("done");
    }

    pub fn try_take_ev(&self) -> Option<Result<QueryEvent, crate::error::RuntimeError>> {
        let val = self.sync_result.take();
        if val.is_some() {
            return val
        }

        self.sync_result.set(val);
        None
    }
}
