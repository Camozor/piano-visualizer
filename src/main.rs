use anyhow::Result;
use chrono::{DateTime, Local};
use crossterm::{
    cursor,
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
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

const FRAME_MS: u64 = 300;
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
struct Drawer {
    stdout: BufWriter<Stdout>,
    width: u16,
    height: u16,
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
            notes: vec![Note {
                key: 3,
                start: Local::now(),
                end: None,
            }],
        };

        visualizer
    }

    pub fn run_loop(&mut self) -> Result<()> {
        // let dt = Local::now();

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
        }
    }
}

#[derive(Debug)]
struct Note {
    key: u16,
    start: DateTime<Local>,
    end: Option<DateTime<Local>>,
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

        for y in start_y..self.height {
            for x in 0..self.width {
                self.draw_pixel(x, y, Color::White)?;
            }
        }

        let stop_y = self
            .height
            .saturating_sub(WHITE_NOTE_HEIGHT - BLACK_NOTE_HEIGHT + 1);
        for y in start_y..stop_y {
            for x in 0..self.width {
                let key = self.get_key_note(x);
                if BLACK_KEYS.contains(&key) {
                    self.draw_pixel(x, y, Color::DarkGrey)?;
                }
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

    fn draw_notes(&mut self, notes: &[Note]) -> Result<()> {
        for note in notes {
            let _ = self.draw_note(note);
        }

        Ok(())
    }

    fn draw_note(&mut self, note: &Note) -> Result<()> {
        let stop_y = self.height.saturating_sub(WHITE_NOTE_HEIGHT);

        for x in 0..self.width {
            let key = self.get_key_note(x);
            if note.key != key {
                continue;
            }
            for y in 0..stop_y {
                if note.key == key {
                    self.draw_pixel(x, y, Color::Red)?;
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
        key
    }

    pub fn clear(&mut self) -> Result<()> {
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))?
            .execute(cursor::MoveTo(0, 0))?;

        Ok(())
    }
}
