/// The abstract trait to represent the status of a task
/// usually use a enum
/// The status like a simple state machine result
pub trait Status: Send + Sync + Default + PartialEq {
    // the task is failed
    fn failed() -> Self;
}
