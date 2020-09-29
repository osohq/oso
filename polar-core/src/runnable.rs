use crate::error::PolarResult;
use crate::events::QueryEvent;
use crate::terms::Term;

/// Trait for something that produces query events and accepts answers.
pub trait Runnable {
    /// Run the Runnable until a Error or QueryEvent is obtained.
    ///
    /// Returns: The next query event or an error.
    fn run(&mut self) -> PolarResult<QueryEvent>;

    fn external_question_result(&mut self, call_id: u64, answer: bool);

    fn external_call_result(&mut self, call_id: u64, term: Option<Term>) -> PolarResult<()>;

    fn external_error(&mut self, message: String);
}
