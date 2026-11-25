use crate::{
    constants::{BLACK_KEYS, BLACK_NOTE_HEIGHT, TOTAL_KEYS, WHITE_NOTE_HEIGHT},
    note::Note,
};
use anyhow::Result;
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    style::{self, Color, Stylize},
    terminal,
};
use std::io::{self, BufWriter, Stdout};

#[derive(Debug)]
pub struct Drawer {
    pub stdout: BufWriter<Stdout>,
    pub width: u16,
    pub height: u16,
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

    pub fn draw_notes(&mut self, notes: &[Note]) -> Result<()> {
        for note in notes {
            let _ = self.draw_note(note);
        }

        Ok(())
    }

    fn draw_note(&mut self, note: &Note) -> Result<()> {
        let _ = self.clear_note(note.key);

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

    pub fn clear_note(&mut self, note: u16) -> Result<()> {
        let max_y = self.height.saturating_sub(WHITE_NOTE_HEIGHT);
        for x in 0..self.width {
            let key = self.get_key_note(x);
            if note != key {
                continue;
            }
            for y in 0..max_y {
                if note == key {
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
