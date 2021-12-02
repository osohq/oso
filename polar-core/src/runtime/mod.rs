use std::{
    collections::{
        BTreeMap,
        HashMap,
        VecDeque
    },
    option::Option,
    sync::{
        Arc,
        Mutex, MutexGuard
    }
};

use tokio::sync::{oneshot::{self, Receiver}, mpsc};

use crate::{
    terms::{Term, Symbol, TermList, Operator},
    messages::Message,
    events::QueryEvent,
    error::{PolarResult, RuntimeError}
};

type ExternalResult<T> = std::result::Result<T, RuntimeError>;
type CallId = u64;

/// Represent host language.
#[derive(Clone, Debug)]
pub struct Host {
    // mutable state
    state: Arc<Mutex<HostState>>
}

#[derive(Debug)]
enum ExternalValue {
    Term(ExternalResult<Option<Term>>),
    Boolean(ExternalResult<bool>)
}

impl ExternalValue {
    fn to_term(self) -> ExternalResult<Option<Term>> {
        if let ExternalValue::Term(e) = self {
            e
        } else {
            panic!("Not term.")
        }
    }

    fn to_bool(self) -> ExternalResult<bool> {
        if let ExternalValue::Boolean(e) = self {
            e
        } else {
            panic!("Not bool.")
        }
    }
}

#[derive(Debug)]
struct HostState {
    results: HashMap<CallId, oneshot::Sender<ExternalValue>>,
    application_error: Option<String>,
    messages: VecDeque<Message>,
    event_rx: mpsc::Receiver<QueryEvent>,
    event_tx: mpsc::Sender<QueryEvent>
}

impl HostState {
    async fn send_event(&mut self, event: QueryEvent) {
        self.event_tx.send(event).await.unwrap()
    }
}

/// Interface the VM uses.
impl Host {
    pub fn new() -> Host {
        let (tx, rx) = mpsc::channel(10);

        let state = HostState {
            results: HashMap::default(),
            application_error: Default::default(),
            messages: Default::default(),
            event_rx: rx,
            event_tx: tx,
        };
        Host { state: Arc::new(Mutex::new(state)) }
    }

    fn state(&self) -> MutexGuard<HostState> {
        self.state.lock().unwrap()
    }

    pub async fn debug(&self, message: String) {
        self.state().send_event(QueryEvent::Debug {
            message
        }).await
    }

    pub async fn make_external(&self, instance_id: u64, constructor: Term) {
        self.state().send_event(QueryEvent::MakeExternal {
            instance_id,
            constructor
        }).await
    }

    pub async fn send_event_with_result(&self, call_id: u64, event: QueryEvent) -> Receiver<ExternalValue> {
        let (tx, rx) = oneshot::channel();
        let mut state = self.state();
        state.results.insert(call_id, tx).unwrap();
        state.send_event(event).await;
        rx
    }

    pub async fn external_call(&self, call_id: u64, instance: Term, attribute: Symbol, args: Option<Vec<Term>>, kwargs: Option<BTreeMap<Symbol, Term>>) -> ExternalResult<Option<Term>> {
        let rx = self.send_event_with_result(call_id, QueryEvent::ExternalCall {
            call_id,
            instance,
            attribute,
            args,
            kwargs
        }).await;

        let external_value = rx.await.unwrap();
        external_value.to_term()
    }


    pub async fn external_isa(&self, call_id: u64, instance: Term, class_tag: Symbol) -> bool {
        let rx = self.send_event_with_result(call_id, QueryEvent::ExternalIsa {
            call_id,
            instance,
            class_tag
        }).await;

        let external_value = rx.await.unwrap();
        external_value.to_bool().unwrap()
    }

    pub async fn external_isa_with_path(&self, call_id: u64, base_tag: Symbol, path: TermList, class_tag: Symbol) -> ExternalResult<bool> {
        let rx = self.send_event_with_result(call_id, QueryEvent::ExternalIsaWithPath {
            call_id,
            base_tag,
            path,
            class_tag
        }).await;

        let external_value = rx.await.unwrap();
        external_value.to_bool()
    }

    pub async fn external_is_sub_specializer(&self, call_id: u64, instance_id: u64, left_class_tag: Symbol, right_class_tag: Symbol) -> bool {
        let rx = self.send_event_with_result(call_id, QueryEvent::ExternalIsSubSpecializer {
            call_id,
            instance_id,
            left_class_tag,
            right_class_tag
        }).await;

        let external_value = rx.await.unwrap();
        external_value.to_bool().unwrap()

    }

    pub async fn external_is_subclass(&self, call_id: u64, left_class_tag: Symbol, right_class_tag: Symbol) -> bool {
        let rx = self.send_event_with_result(call_id, QueryEvent::ExternalIsSubclass {
            call_id,
            left_class_tag,
            right_class_tag
        }).await;

        let external_value = rx.await.unwrap();
        external_value.to_bool().unwrap()
    }

    pub async fn external_op(&self, call_id: u64, operator: Operator, args: TermList) -> bool {
        let rx = self.send_event_with_result(call_id, QueryEvent::ExternalOp {
            call_id,
            operator,
            args
        }).await;

        let external_value = rx.await.unwrap();
        external_value.to_bool().unwrap()
    }
}


/// Interface that Polar object uses.
impl Host {
    pub fn external_call_result(&self, call_id: u64, term: Option<Term>) -> PolarResult<()> {
        let sender = self.state().results.remove(&call_id).unwrap();
        sender.send(ExternalValue::Term(Ok(term))).unwrap();
        Ok(())
    }

    pub fn external_question_result(&self, call_id: u64, b: bool) -> PolarResult<()> {
        let sender = self.state().results.remove(&call_id).unwrap();
        sender.send(ExternalValue::Boolean(Ok(b))).unwrap();
        Ok(())
    }

    pub fn application_error(&self, _message: String) {
        unimplemented!("Not handling these gracefully right now.")
    }

    pub fn next_event(&self) -> Option<PolarResult<QueryEvent>> {
        let mut state = self.state();
        match state.event_rx.try_recv() {
            Ok(r) => Some(Ok(r)),
            Err(mpsc::error::TryRecvError::Empty) => None,
            e @ Err(mpsc::error::TryRecvError::Disconnected) => unimplemented!("Handle this error."),
        }
    }
}