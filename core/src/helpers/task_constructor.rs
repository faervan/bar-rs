use std::marker::PhantomData;

use iced::Task;

use crate::message::Message;

pub struct TaskConstructor<T, M = Message> {
    constructors: Vec<Box<dyn FnOnce(&T) -> Task<M>>>,
    _phantom: PhantomData<(T, M)>,
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
        F: FnOnce(&T) -> Task<M> + 'static,
    {
        self.constructors.push(Box::new(f));
        self
    }

    pub fn build(self, t: &T) -> Task<M> {
        self.constructors
            .into_iter()
            .fold(Task::none(), |task, constructor| task.chain(constructor(t)))
    }
}
