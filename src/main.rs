use anyhow::Result;
use chrono::{DateTime, Local, TimeDelta};
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    style::{self, Color, Stylize},
    terminal,
};
use std::io::{self, BufWriter, Stdout, Write};
use std::{thread, time};

// white keys =  52, black keys = 36
const TOTAL_KEYS: u8 = 88;

const WHITE_NOTE_HEIGHT: u16 = 10;
const BLACK_NOTE_HEIGHT: u16 = 6;

const BLACK_KEYS: [u16; 36] = [
    1, 4, 6, 9, 11, 13, 16, 18, 21, 23, 25, 28, 30, 33, 35, 37, 40, 42, 45, 47, 49, 52, 54, 57, 59,
    61, 64, 66, 69, 71, 73, 76, 78, 81, 83, 85,
];

const FRAME_MS: u64 = 5;
const TIME_NOTE_ON_SCREEN_MS: u32 = 5_000;

fn main() -> Result<()> {
    let mut visualizer = Visualizer::new();
    let _ = visualizer.run_loop();

    let _ = visualizer.drawer.stdout.execute(cursor::Show);
    let _ = visualizer
        .drawer
        .stdout
        .execute(terminal::LeaveAlternateScreen);

    Ok(())
}

#[derive(Debug)]
struct Visualizer {
    drawer: Drawer,
    notes: Vec<Note>,
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

#[derive(Debug)]
struct Drawer {
    stdout: BufWriter<Stdout>,
    width: u16,
    height: u16,
}

impl Drawer {
    pub fn new() -> Self {
        let stdout = io::stdout();
        let stdout = BufWriter::new(stdout);

        let size = terminal::size();
        let size = size.unwrap_or((0, 0));

        Drawer {
            stdout,
            width: size.0,
            height: size.1,
        }
    }

    pub fn draw_piano(&mut self) -> Result<()> {
        let note_height = WHITE_NOTE_HEIGHT.min(self.height);
        let start_y = self.height.saturating_sub(note_height);
        let black_stop_y = self
            .height
            .saturating_sub(WHITE_NOTE_HEIGHT - BLACK_NOTE_HEIGHT);

        let is_black_col: Vec<bool> = (0..self.width)
            .map(|x| BLACK_KEYS.contains(&self.get_key_note(x)))
            .collect();

        for y in start_y..self.height {
            for x in 0..self.width {
                let color = if y < black_stop_y && is_black_col[x as usize] {
                    Color::DarkGrey
                } else {
                    Color::White
                };
                self.draw_pixel(x, y, color)?;
            }
        }

        Ok(())
    }

    fn draw_pixel(&mut self, x: u16, y: u16, color: Color) -> Result<()> {
        self.stdout
            .queue(cursor::MoveTo(x, y))?
            .queue(style::PrintStyledContent("█".stylize().with(color)))?;

        Ok(())
    }

    fn clear_pixel(&mut self, x: u16, y: u16) -> Result<()> {
        self.stdout
            .queue(cursor::MoveTo(x, y))?
            .queue(style::PrintStyledContent(" ".reset()))?;

        Ok(())
    }

    fn draw_notes(&mut self, notes: &[Note]) -> Result<()> {
        for note in notes {
            let _ = self.draw_note(note);
        }

        Ok(())
    }

    fn draw_note(&mut self, note: &Note) -> Result<()> {
        let _ = self.clear_note(note);

        let max_y = self.height.saturating_sub(WHITE_NOTE_HEIGHT);
        let start_y = note.compute_start_note_y(max_y);
        let end_y = note.compute_end_note_y(max_y);

        for x in 0..self.width {
            let key = self.get_key_note(x);
            if note.key != key {
                continue;
            }
            for y in start_y..end_y {
                if note.key == key {
                    self.draw_pixel(x, y, Color::Red)?;
                }
            }
        }

        Ok(())
    }

    fn clear_note(&mut self, note: &Note) -> Result<()> {
        let max_y = self.height.saturating_sub(WHITE_NOTE_HEIGHT);
        for x in 0..self.width {
            let key = self.get_key_note(x);
            if note.key != key {
                continue;
            }
            for y in 0..max_y {
                if note.key == key {
                    self.clear_pixel(x, y)?;
                }
            }
        }

        Ok(())
    }

    /// Returns the key (0 to 87) from a x position
    fn get_key_note(&self, x_pos: u16) -> u16 {
        let ratio = x_pos as f32 / self.width as f32;
        let key = ratio * TOTAL_KEYS as f32;

        let key = key.floor() as u16;
        key.min((TOTAL_KEYS - 1) as u16)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))?
            .execute(cursor::MoveTo(0, 0))?;

        Ok(())
    }
}

#[derive(Debug)]
struct Note {
    key: u16,
    start: DateTime<Local>,
    end: Option<DateTime<Local>>,
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
