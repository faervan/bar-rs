use std::{fmt::Debug, sync::Arc};

use config::Config;
use iced::{window::Id, Element, Task};
use message::Message;

mod config;
mod message;
mod modules;
mod registry;
mod subscription;

fn main() -> iced::Result {
    env_logger::init();
    iced::daemon("Crabbar", State::update, State::view)
        .subscription(State::subscribe)
        .run_with(State::new)
}

#[derive(Debug)]
struct State {
    config: Arc<Config>,
}

impl State {
    fn new() -> (Self, Task<Message>) {
        (
            State {
                config: Arc::new(Config::default()),
            },
            Task::none(),
        )
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        use Message::*;
        match msg {
            FetchSubscriptions(sx) => {
                sx.send(vec![]).unwrap();
            }
            FetchConfig(sx) => {
                sx.send(self.config.clone()).unwrap();
            }
            Update(updates) => {
                for updatefn in updates {
                    Arc::into_inner(updatefn.0).unwrap()()
                }
            }
        }
        Task::none()
    }

    fn view(&self, _: Id) -> Element<Message> {
        "hello".into()
    }
}
