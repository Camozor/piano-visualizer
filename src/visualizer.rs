use anyhow::Result;
use chrono::{Local, TimeDelta};
use crossterm::{ExecutableCommand, QueueableCommand, cursor, terminal};
use std::io::Write;
use std::thread;
use std::time;

use crate::{constants::TIME_NOTE_ON_SCREEN_MS, drawer::Drawer, note::Note};

const FRAME_MS: u64 = 5;

#[derive(Debug)]
pub struct Visualizer {
    pub drawer: Drawer,
    pub notes: Vec<Note>,
}

impl Visualizer {
    pub fn new() -> Self {
        let drawer = Drawer::new();

        let visualizer = Visualizer {
            drawer,
            notes: vec![
                Note {
                    key: 3,
                    start: Local::now(),
                    end: None,
                },
                Note {
                    key: 7,
                    start: Local::now() - TimeDelta::milliseconds(3000),
                    end: Some(Local::now() - TimeDelta::milliseconds(1000)),
                },
            ],
        };

        visualizer
    }

    pub fn run_loop(&mut self) -> Result<()> {
        self.drawer.clear()?;
        self.drawer.stdout.execute(terminal::EnterAlternateScreen)?;
        self.drawer.stdout.execute(cursor::Hide)?;

        loop {
            self.drawer.stdout.queue(cursor::MoveTo(0, 0))?;
            let _ = self.drawer.draw_piano();
            let _ = self.drawer.draw_notes(&self.notes);
            self.drawer.stdout.queue(cursor::MoveTo(0, 0))?;
            self.drawer.stdout.flush()?;

            let frame_duration = time::Duration::from_millis(FRAME_MS);
            thread::sleep(frame_duration);
            self.remove_old_notes();
        }
    }

    fn remove_old_notes(&mut self) {
        let now = Local::now();
        let max_age = TimeDelta::milliseconds(TIME_NOTE_ON_SCREEN_MS as i64);

        let old_notes: Vec<u16> = self.notes.iter().map(|note| note.key).collect();

        self.notes
            .retain(|note| note.end.is_none() || note.end.is_some_and(|end| now - end < max_age));

        let new_notes: Vec<u16> = self.notes.iter().map(|note| note.key).collect();

        let diff = substract(old_notes, new_notes);

        for note in diff {
            let _ = self.drawer.clear_note(note);
        }
    }
}

fn substract(vec1: Vec<u16>, vec2: Vec<u16>) -> Vec<u16> {
    let mut substracted = vec![];
    for e1 in vec1 {
        if !vec2.contains(&e1) {
            substracted.push(e1);
        }
    }

    substracted
}
