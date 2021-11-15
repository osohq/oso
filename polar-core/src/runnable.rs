use crate::counter::Counter;
use crate::error::RuntimeError;
use crate::events::QueryEvent;
use crate::terms::Term;

/// Trait for something that produces query events and accepts answers.
///
/// Runnable must be clone so that the VM can re-execute runnables when
/// backtracking & retrying alternatives.
pub trait Runnable {
    /// Run the Runnable until an Error or QueryEvent is obtained.
    ///
    /// The optional Counter may be used to create monotonically increasing call IDs that will not
    /// conflict with the parent VM's call IDs.
    fn run(&mut self, _counter: Option<&mut Counter>) -> Result<QueryEvent, RuntimeError>;

    fn external_question_result(
        &mut self,
        _call_id: u64,
        _answer: bool,
    ) -> Result<(), RuntimeError> {
        Err(RuntimeError::InvalidState {
            msg: "Unexpected query answer".to_string(),
        })
    }

    fn external_call_result(
        &mut self,
        _call_id: u64,
        _term: Option<Term>,
    ) -> Result<(), RuntimeError> {
        Err(RuntimeError::InvalidState {
            msg: "Unexpected external call".to_string(),
        })
    }

    fn debug_command(&mut self, _command: &str) -> Result<(), RuntimeError> {
        Err(RuntimeError::InvalidState {
            msg: "Unexpected debug command".to_string(),
        })
    }

    fn handle_error(&mut self, err: RuntimeError) -> Result<QueryEvent, RuntimeError> {
        Err(err)
    }

    // TODO Alternative?: Goal::Run takes a Runnable constructor function.
    /// Create a new runnable that when run will perform the same operation as
    /// this one.
    fn clone_runnable(&self) -> Box<dyn Runnable>;
}

impl Clone for Box<dyn Runnable> {
    fn clone(&self) -> Self {
        (*self).clone_runnable()
    }
}

impl std::fmt::Debug for Box<dyn Runnable> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Box<dyn Runnable>")
    }
}
