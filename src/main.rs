use std::{env::args, f32::consts::PI};

use hound::{self, WavSpec};
use realfft::RealFftPlanner;
use rustfft::num_complex::Complex;

const MIDI_NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

fn frequency_to_midi(frequency: f32) -> f32 {
    69.0 + 12.0 * (frequency / 440.0).log2()
}

fn midi_to_note_name(midi_note: f32) -> String {
    let midi_number = midi_note.round() as i32;
    let note_index = midi_number % 12;
    let octave = (midi_number / 12) - 1;
    format!("{}{}", MIDI_NOTE_NAMES[note_index as usize], octave)
}

fn main() {
    let args: Vec<String> = args().collect();

    if args.len() != 2 {
        println!("Usage: ./{} <audio file>", args[0]);
        return;
    }

    let audio_path = &args[1];

    // let audio_path = "saw-c#.wav";
    // let pitch = detect_pitch(&audio_path);
    // match pitch {
    //     Some(p) => {
    //         let midi_note = frequency_to_midi(p);
    //         let note_name = midi_to_note_name(midi_note);
    //         println!(
    //             "The detected pitch is {:.2} Hz | {midi_note} => {note_name}",
    //             p,
    //         )
    //     }
    //     None => println!("Could not detect pitch"),
    // }

    // if let Some((frequency, midi_note, note_name)) = detect_bass_root_note(&audio_path) {
    //     println!(
    //         "Bass root note is {:.2} Hz | MIDI Note: {:.2} | Note Name: {}",
    //         frequency, midi_note, note_name
    //     );
    // } else {
    //     println!("Could not detect bass root note");
    // }

    detect_note_switches(&audio_path);
}

fn detect_pitch(audio_path: &str) -> Option<f32> {
    // Read the audio file
    let reader = hound::WavReader::open(audio_path).expect("Failed to open WAV file");
    let spec = reader.spec();
    let samples: Vec<f32> = reader
        .into_samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();

    // Set up FFT
    let mut planner = RealFftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(samples.len());

    // Perform FFT
    let mut input = samples.clone();
    let mut output = fft.make_output_vec();
    fft.process(&mut input, &mut output).expect("FFT failed");

    // Calculate magnitudes
    let magnitudes: Vec<f32> = output.iter().map(|c| c.norm()).collect();

    // Find the peak frequency
    let max_index = magnitudes
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, _)| i)?;
    let frequency = max_index as f32 * spec.sample_rate as f32 / samples.len() as f32;

    Some(frequency)
}

fn detect_bass_root_note(audio_path: &str) -> Option<(f32, f32, String)> {
    // Read the audio file
    let fft = process_fft(audio_path);

    // Calculate magnitudes
    let magnitudes: Vec<f32> = fft.complex.iter().map(|c| c.norm()).collect();

    // Analyze lower frequencies to find the bass root note
    let bass_threshold = 80.0; // 200.0 // Adjust threshold as needed
    let sample_rate = fft.spec.sample_rate as f32;
    let bin_freq = sample_rate / fft.samples.len() as f32;
    let mut max_magnitude = 0.0;
    let mut bass_freq = 0.0;

    for (i, &magnitude) in magnitudes.iter().enumerate() {
        let freq = i as f32 * bin_freq;
        if freq > bass_threshold {
            break;
        }
        if magnitude > max_magnitude {
            max_magnitude = magnitude;
            bass_freq = freq;
        }
    }

    if bass_freq > 0.0 {
        let midi_note = frequency_to_midi(bass_freq);
        let note_name = midi_to_note_name(midi_note);
        Some((bass_freq, midi_note, note_name))
    } else {
        None
    }
}

fn detect_note_switches(audio_path: &str) {
    // Read the audio file
    let reader = hound::WavReader::open(audio_path).expect("Failed to open WAV file");
    let spec = reader.spec();
    let samples: Vec<f32> = reader
        .into_samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();

    let frame_size = 2048 * 8; // Adjust frame size as needed
    let hop_size = frame_size / 2; // 50% overlap
    let sample_rate = spec.sample_rate as f32;
    let bin_freq = sample_rate / frame_size as f32;
    let bass_threshold = 80.0; // Adjust threshold as needed
    let debounce_frames = 10; // Number of frames to debounce repeated note detection

    // Set up FFT with the correct frame size
    let mut planner = RealFftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(frame_size);

    let mut prev_note = String::new();
    let mut output = fft.make_output_vec();
    let mut last_detection_frame = 0;

    for i in (0..samples.len()).step_by(hop_size) {
        if i + frame_size >= samples.len() {
            break;
        }

        let frame = &samples[i..i + frame_size];

        // Apply a Hann window to the frame
        let windowed_frame: Vec<f32> = frame
            .iter()
            .enumerate()
            .map(|(i, &x)| x * 0.5 * (1.0 - (2.0 * PI * i as f32 / (frame_size - 1) as f32).cos()))
            .collect();

        // Perform FFT
        let mut input = windowed_frame.to_vec();
        fft.process(&mut input, &mut output).expect("FFT failed");

        // Calculate magnitudes
        let magnitudes: Vec<f32> = output.iter().map(|c| c.norm()).collect();

        // Analyze lower frequencies to find the bass root note
        let mut max_magnitude = 0.0;
        let mut bass_freq = 0.0;

        for (j, &magnitude) in magnitudes.iter().enumerate() {
            let freq = j as f32 * bin_freq;
            if freq > bass_threshold {
                break;
            }
            if magnitude > max_magnitude {
                max_magnitude = magnitude;
                bass_freq = freq;
            }
        }

        if bass_freq > 0.0 {
            let midi_note = frequency_to_midi(bass_freq);
            let note_name = midi_to_note_name(midi_note);
            if note_name != prev_note {
                println!(
                    "Detected bass note switch: {} at {:.2} Hz",
                    note_name, bass_freq
                );
                prev_note = note_name;
            }
        }
    }
}

struct ProcessedFFT {
    spec: WavSpec,
    samples: Vec<f32>,
    complex: Vec<Complex<f32>>,
}

fn process_fft(audio_path: &str) -> ProcessedFFT {
    let reader = hound::WavReader::open(audio_path).expect("Failed to open WAV file");
    let spec = reader.spec();
    let samples: Vec<f32> = reader
        .into_samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();

    // Set up FFT
    let mut planner = RealFftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(samples.len());

    // Perform FFT
    let mut input = samples.clone();
    let mut output = fft.make_output_vec();
    fft.process(&mut input, &mut output).expect("FFT failed");

    ProcessedFFT {
        spec,
        samples,
        complex: output,
    }
}
