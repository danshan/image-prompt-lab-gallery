use crate::DomainResult;

pub trait Clock {
    fn now_millis(&self) -> DomainResult<String>;
}

pub trait IdGenerator {
    fn new_id(&self) -> String;
}

pub trait TransactionManager {
    fn run_in_transaction<T>(&self, operation: impl FnOnce() -> DomainResult<T>)
        -> DomainResult<T>;
}
