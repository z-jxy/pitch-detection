use std::{f32::consts::PI, fs::File, io::BufReader};

use hound::WavReader;
use realfft::RealFftPlanner;

use crate::midi;

const FRAME_SIZE: usize = 2048 * 8;

pub fn detect_note_switches(reader: WavReader<BufReader<File>>) -> Vec<String> {
    let spec = reader.spec();
    let samples: Vec<f32> = reader
        .into_samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();

    // let frame_size = 2048 * 8; // Adjust frame size as needed
    let hop_size = FRAME_SIZE / 4; // 50% overlap
    let sample_rate = spec.sample_rate as f32;
    let bin_freq = sample_rate / FRAME_SIZE as f32;
    let bass_threshold = 80.0; // Adjust threshold as needed

    // Set up FFT with the correct frame size
    let mut planner = RealFftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FRAME_SIZE);

    let mut prev_note = String::new();
    let mut output = fft.make_output_vec();
    let mut notes = vec![];
    let mut detected_notes = Vec::new();
    let debounce_frames = 10; // Number of frames to debounce repeated note detection

    for i in (0..samples.len()).step_by(hop_size) {
        if i + FRAME_SIZE >= samples.len() {
            break;
        }

        let frame = &samples[i..i + FRAME_SIZE];

        // Apply a Hann window to the frame
        let mut windowed_frame: Vec<f32> = frame
            .iter()
            .enumerate()
            .map(|(i, &x)| x * 0.5 * (1.0 - (2.0 * PI * i as f32 / (FRAME_SIZE - 1) as f32).cos()))
            .collect();

        // Perform FFT
        // let mut input = windowed_frame.to_vec();
        fft.process(&mut windowed_frame, &mut output)
            .expect("FFT failed");

        // Calculate magnitudes
        let magnitudes: Vec<f32> = output.iter().map(|c| c.norm()).collect();

        if let Some(bass_freq) = pick_peak_frequency(&magnitudes, bin_freq, bass_threshold) {
            let midi_note = midi::frequency_to_midi(bass_freq);
            let note_name = midi::midi_to_note_name(midi_note);

            detected_notes.push(note_name.clone());

            if detected_notes.len() >= debounce_frames {
                let last_detected_note = &detected_notes[detected_notes.len() - debounce_frames];
                if note_name != prev_note && note_name == *last_detected_note {
                    println!(
                        "Detected bass note switch: {} at {:.2} Hz",
                        note_name, bass_freq
                    );
                    notes.push(note_name.clone());
                    prev_note = note_name;
                    detected_notes.clear();
                }
            }
        }
    }

    notes
}

fn pick_peak_frequency(magnitudes: &[f32], bin_freq: f32, bass_threshold: f32) -> Option<f32> {
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
        Some(bass_freq)
    } else {
        None
    }
}
