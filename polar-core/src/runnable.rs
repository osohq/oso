use crate::counter::Counter;
use crate::error::{OperationalError, PolarError, PolarResult};
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
    fn run(&mut self, _counter: Option<&mut Counter>) -> PolarResult<QueryEvent>;

    fn external_question_result(&mut self, _call_id: u64, _answer: bool) -> PolarResult<()> {
        Err(OperationalError::InvalidState {
            msg: "Unexpected query answer".to_string(),
        }
        .into())
    }

    fn external_call_result(&mut self, _call_id: u64, _term: Option<Term>) -> PolarResult<()> {
        Err(OperationalError::InvalidState {
            msg: "Unexpected external call".to_string(),
        }
        .into())
    }

    fn debug_command(&mut self, _command: &str) -> PolarResult<()> {
        Err(OperationalError::InvalidState {
            msg: "Unexpected debug command".to_string(),
        }
        .into())
    }

    fn handle_error(&mut self, err: PolarError) -> PolarResult<QueryEvent> {
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
