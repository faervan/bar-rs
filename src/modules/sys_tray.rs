use iced::{futures::Stream, stream};
//use system_tray::client::Client;

use crate::Message;


pub fn _system_tray() -> impl Stream<Item = Message> {
    stream::channel(1, |mut _sender| async move {
        /*let client = Client::new().await.unwrap();
        let mut tray_rx = client.subscribe();

        let initial_items = client.items();

        println!("initial_items: {initial_items:#?}\n\n");

        // do something with initial items...
        drop(initial_items);

        while let Ok(ev) = tray_rx.recv().await {
            println!("{ev:#?}"); // do something with event...
        }*/
    })
}
