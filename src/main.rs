use crossterm::{
    cursor,
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::io::{self, Stdout, Write};

// white keys =  52, black keys = 36
const TOTAL_KEYS: u8 = 88;

const WHITE_NOTE_HEIGHT: u16 = 10;
const BLACK_NOTE_HEIGHT: u16 = 6;

const BLACK_KEYS: [u16; 36] = [
    1, 4, 6, 9, 11, 13, 16, 18, 21, 23, 25, 28, 30, 33, 35, 37, 40, 42, 45, 47, 49, 52, 54, 57, 59,
    61, 64, 66, 69, 71, 73, 76, 78, 81, 83, 85,
];

fn main() -> io::Result<()> {
    let mut drawer = Drawer::new();

    drawer.clear()?;
    drawer.draw_piano()?;

    Ok(())
}

#[derive(Debug)]
struct Drawer {
    stdout: Stdout,
    width: u16,
    height: u16,
}

impl Drawer {
    pub fn new() -> Self {
        let stdout = io::stdout();

        let size = terminal::size();
        let size = size.unwrap_or((0, 0));

        Drawer {
            stdout,
            width: size.0,
            height: size.1,
        }
    }

    pub fn draw_piano(&mut self) -> io::Result<()> {
        let note_height = WHITE_NOTE_HEIGHT.min(self.height);
        let start_y = self.height.saturating_sub(note_height);

        for y in start_y..self.height {
            for x in 0..self.width {
                self.draw_pixel(x, y, Color::White)?;
            }
        }

        let stop_y = self
            .height
            .saturating_sub(WHITE_NOTE_HEIGHT - BLACK_NOTE_HEIGHT);
        for y in start_y..stop_y {
            for x in 0..self.width {
                let key = self.get_key(x);
                if BLACK_KEYS.contains(&key) {
                    self.draw_pixel(x, y, Color::DarkGrey)?;
                }
            }
        }

        let end_y = self.height.saturating_sub(1);
        self.stdout.queue(cursor::MoveTo(0, end_y))?;
        self.stdout.flush()?;
        Ok(())
    }

    pub fn draw_pixel(&mut self, x: u16, y: u16, color: Color) -> io::Result<()> {
        self.stdout
            .queue(cursor::MoveTo(x, y))?
            .queue(style::PrintStyledContent("█".stylize().with(color)))?;

        Ok(())
    }

    /// Returns the key (0 to 87) from a x position
    pub fn get_key(&self, x_pos: u16) -> u16 {
        let ratio = x_pos as f32 / self.width as f32;
        let key = ratio * TOTAL_KEYS as f32;

        let key = key.floor() as u16;
        key
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))?
            .execute(cursor::MoveTo(0, 0))?;

        Ok(())
    }
}
