extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;
extern crate rand;

use std::env;

use futures::Stream;
use tokio_core::reactor::Core;
use telegram_bot::*;

use std::fs::File;
use std::io::{self, BufRead, BufReader};

use rand::distributions::{IndependentSample, Range};

use rand::ThreadRng;

struct Fortunes {
    data: Vec<String>,    
    step: Range<u32>,
    rng: ThreadRng,
}

impl Fortunes {
    pub fn random_fortune(&mut self) -> &str {
        let choice: u32 = self.step.ind_sample(&mut self.rng);
        let fortune = &self.data[choice as usize];
        fortune
    }
}

fn main() {
    let path = "./lor_fortunes.txt";
    println!("Loading fortunes from file {}", path);
    let data: Vec<String> = load_fortunes(path);    
    let mut fortunes = init_fortunes(data);
    println!("{} Fortunes loaded successfully", fortunes.data.len());

    let mut core = Core::new().unwrap();
    let token = env::var("TELEGRAM_TOKEN").unwrap();
    let api = Api::configure(token).build(core.handle()).unwrap();
    
    let future = api.stream().for_each(|update| {
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                if data == "/fortune" {
                    api.spawn(message.text_reply(fortunes.random_fortune()));
                } else if data == "/help" {
                    api.spawn(message.text_reply("/fortune - постит случайную фортунку с лора"));
                }
            }
        }

        Ok(())
    });

    core.run(future).unwrap();
}

fn init_fortunes<'a>(data: Vec<String>) -> Fortunes {
    let size = data.len();
    Fortunes {
        data: data,        
        step: Range::new(0, size as u32),
        rng: rand::thread_rng()
    }
}

fn load_fortunes(path: &str) -> Vec<String> {
    let f = File::open(&path).unwrap();
    let reader = BufReader::new(f);
    let result: io::Result<Vec<String>> = reader.lines().collect();
    let raw_data: Vec<String> = result.unwrap();
    let mut result: Vec<String> = Vec::new();
    let mut fortune_part: String = String::new();

    for line in raw_data {
        if line == "%" {
            result.push(fortune_part);
            fortune_part = String::new();
        } else {
            fortune_part = fortune_part + &line + "\n";
        }
    }

    result
}
