mod analyzer;
mod midi;

use std::env::args;

use hound::{self};

fn main() {
    let args: Vec<String> = args().collect();

    if args.len() != 2 {
        println!("Usage: ./{} <audio file>", args[0]);
        return;
    }

    let audio_path = &args[1];

    let reader = hound::WavReader::open(audio_path).expect("Failed to open WAV file");

    // detect_note_switches(&audio_path);
    for note in analyzer::detect_note_switches(reader) {
        // println!("{}", note);
    }
}
