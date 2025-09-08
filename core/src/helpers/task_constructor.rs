use std::marker::PhantomData;

use iced::Task;

use crate::message::Message;

#[allow(clippy::type_complexity)]
/// Used to create a [Task] using a type [T] that is not in the scope. When the type [T] comes into
/// scope, the [TaskConstructor] can be build into a [Task].
pub struct TaskConstructor<T, M = Message> {
    constructors: Vec<Box<dyn FnOnce(&mut T) -> Task<M>>>,
    _phantom: PhantomData<(T, M)>,
}

impl<T> Default for TaskConstructor<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, M> TaskConstructor<T, M>
where
    M: 'static,
{
    pub fn new() -> Self {
        TaskConstructor {
            constructors: vec![],
            _phantom: PhantomData,
        }
    }

    pub fn chain<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut T) -> Task<M> + 'static,
    {
        self.constructors.push(Box::new(f));
        self
    }

    pub fn build(self, t: &mut T) -> Task<M> {
        self.constructors
            .into_iter()
            .fold(Task::none(), |task, constructor| task.chain(constructor(t)))
    }
}
