use std::time;

use anyhow::Result;
use chrono::{Local, TimeDelta};
use crossterm::{ExecutableCommand, QueueableCommand, cursor, terminal};
use std::io::Write;
use std::thread;

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

        self.notes
            .retain(|note| note.end.is_none() || note.end.is_some_and(|end| now - end < max_age));
    }
}
