extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;
extern crate rand;

use std::env;

use futures::Stream;
use tokio_core::reactor::Core;
use telegram_bot::*;

use std::fs::File;
use std::io::prelude::*;

use rand::distributions::{IndependentSample, Range};

use std::collections::HashMap;

fn main() {
    let mut f = File::open("./lor_fortunes.txt").expect("file not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    let vec: Vec<&str> = contents.split("\n").collect();

    let mut currentFortune = "".to_string();
    let mut count = 0;
    let mut map: HashMap<u32, String> = HashMap::new();
    for fortunePart in vec {
        if (fortunePart != "%") {
            currentFortune += &("\n".to_string() + fortunePart);
        } else {
            map.insert(count, currentFortune);
            currentFortune = "".to_string();
            count += 1;
        }
    }

    let mut core = Core::new().unwrap();

    let token = env::var("TELEGRAM_TOKEN").unwrap();
    let api = Api::configure(token).build(core.handle()).unwrap();

    // Fetch new updates via long poll method
    let future = api.stream().for_each(|update| {

        // If the received update contains a new message...
        if let UpdateKind::Message(message) = update.kind {

            if let MessageKind::Text {ref data, ..} = message.kind {
                // Print received text message to stdout.
                println!("<{}>: {}", &message.from.first_name, data);
                let step =  Range::new(0, count);
                let mut rng = rand::thread_rng();
                let choice = step.ind_sample(&mut rng);
                let fortune = map.get(&choice).unwrap();

                if (data == "/fortune") {
                // Answer message with "Hi".
                    api.spawn(message.text_reply(fortune));
                } else if (data == "/help") {
                    api.spawn(message.text_reply("/fortune - постит случайную фортунку с лора"));
                }

            }
        }

        Ok(())
    });

    core.run(future).unwrap();
}