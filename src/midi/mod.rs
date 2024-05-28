const MIDI_NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

pub fn frequency_to_midi(frequency: f32) -> f32 {
    69.0 + 12.0 * (frequency / 440.0).log2()
}

pub fn midi_to_note_name(midi_note: f32) -> String {
    let midi_number = midi_note.round() as i32;
    let note_index = midi_number % 12;
    let octave = (midi_number / 12) - 1;
    format!("{}{}", MIDI_NOTE_NAMES[note_index as usize], octave)
}
