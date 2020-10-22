use crate::counter::Counter;
use crate::error::{OperationalError, PolarResult};
use crate::events::QueryEvent;
use crate::terms::Term;
use crate::vm::BindingStack;

/// Trait for something that produces query events and accepts answers.
///
/// Runnable must be clone so that the VM can re-execute runnables when
/// backtracking & retrying alternatives.
pub trait Runnable {
    /// Run the Runnable until a Error or QueryEvent is obtained.
    ///
    /// Returns: The next query event or an error.
    fn run(
        &mut self,
        bindings: Option<&mut BindingStack>,
        bsp: Option<&mut usize>,
        counter: Option<&mut Counter>,
    ) -> PolarResult<QueryEvent>;

    fn external_question_result(&mut self, _call_id: u64, _answer: bool) -> PolarResult<()> {
        Err(OperationalError::InvalidState("Unexpected query answer".to_string()).into())
    }

    fn external_error(&mut self, _message: String) -> PolarResult<()> {
        Err(OperationalError::InvalidState("Unexpected external error".to_string()).into())
    }

    fn external_call_result(&mut self, _call_id: u64, _term: Option<Term>) -> PolarResult<()> {
        Err(OperationalError::InvalidState("Unexpected external call".to_string()).into())
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
