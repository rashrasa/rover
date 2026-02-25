use std::sync::{Arc, Mutex};

/// A value that will be resolved at a later time.
/// Simple mechanism for returning a placeholder value which gets resolved elsewhere.
///
/// Can be cloned and consumed as many times as needed.
///
/// Consuming this data returns a cloned version of the resolved value.
#[derive(Debug, Clone)]
pub struct Completer<T: Clone + Copy> {
    precondition: Option<&'static str>,
    inner: Arc<Mutex<Option<T>>>,
}

impl<T: Clone + Copy> Completer<T> {
    pub fn new(precondition: Option<&'static str>) -> Self {
        Self {
            precondition,
            inner: Arc::new(Mutex::new(None)),
        }
    }

    pub fn from_value(value: T) -> Self {
        Self {
            precondition: None,
            inner: Arc::new(Mutex::new(Some(value))),
        }
    }

    pub fn complete(&mut self, value: T) -> Result<(), CompleterError<T>> {
        let mut current = self.inner.lock().unwrap();
        if let Some(v) = *current {
            return Err(CompleterError::Completed(v));
        }
        *current = Some(value);
        Ok(())
    }

    pub fn consume(self) -> Result<T, CompleterError<T>> {
        let inner = self.inner.lock().unwrap();
        match *inner {
            None => {
                return Err(CompleterError::PreconditionFailed(
                    self.precondition.unwrap_or("Unspecified Precondition"),
                ));
            }
            Some(v) => return Ok(v.clone()),
        }
    }
}

#[derive(Debug)]
pub enum CompleterError<T: Clone + Copy> {
    Completed(T),
    PreconditionFailed(&'static str),
}
