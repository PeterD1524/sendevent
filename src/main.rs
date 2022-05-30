use std::{
    fs::File,
    io::{self, BufReader},
};

use clap::Parser;
use sendevent;

#[derive(Parser)]
struct Args {
    #[clap(long)]
    device: Option<String>,
    #[clap(long)]
    path: Option<String>,
}

fn main() {
    let args = Args::parse();
    let device = args
        .device
        .as_ref()
        .and_then(|device| Some(device.as_str()));
    if let Some(path) = args.path {
        let file = File::open(path).unwrap();
        let mut reader = BufReader::new(file);
        sendevent::send_events_from_reader(&mut reader, device);
    } else {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        sendevent::send_events_from_reader(&mut reader, device);
    }
}
