use crate::constants::TIME_NOTE_ON_SCREEN_MS;
use chrono::{DateTime, Local};

#[derive(Debug)]
pub struct Note {
    pub key: u16,
    pub start: DateTime<Local>,
    pub end: Option<DateTime<Local>>,
}

impl Note {
    /// The highest point for the note
    pub fn compute_start_note_y(&self, max_height: u16) -> u16 {
        let duration = self.start - Local::now();
        let millis = duration.num_milliseconds().abs();
        let ratio = millis as f32 / TIME_NOTE_ON_SCREEN_MS as f32;
        let height = (ratio * max_height as f32).floor();

        if height as u16 >= max_height {
            return 0;
        }

        max_height - height as u16
    }

    /// The lowest point for the note
    pub fn compute_end_note_y(&self, max_height: u16) -> u16 {
        match self.end {
            Some(end) => {
                let duration = end - Local::now();
                let millis = duration.num_milliseconds().abs();
                let ratio = millis as f32 / TIME_NOTE_ON_SCREEN_MS as f32;
                let height = (ratio * max_height as f32).floor();

                if height as u16 >= max_height {
                    return 0;
                }

                max_height - height as u16
            }
            None => max_height,
        }
    }
}
