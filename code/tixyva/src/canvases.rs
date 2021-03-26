use std::io::{stdout, Write};

use crossterm::{
    cursor, execute, queue,
    style::{self, style, Color},
    terminal,
};

use crate::Globals;

pub const WIDTH: usize = 32;
pub const HEIGHT: usize = 32;
const FILLS: &[char] = &[' ', '.', '+', '#', '░', '▒', '▓', '█'];

#[derive(Debug, Clone)]
pub struct Canvas(pub Vec<f64>);

impl Canvas {
    /// Make a new blank canvas with the given size
    pub fn new() -> Self {
        Canvas(vec![0.0; WIDTH * HEIGHT])
    }
}

impl Globals {
    pub(crate) fn render_canvas(&mut self) -> anyhow::Result<()> {
        execute!(stdout(), cursor::Hide)?;

        let size = terminal::size()?;
        // center the canvas
        let start_col = size.0 / 2 - WIDTH as u16;
        let start_row = size.1 / 2 - HEIGHT as u16 / 2;

        for (idx, &value) in self.canvas.0.iter().enumerate() {
            let color = if value == 0.0 {
                Color::Black
            } else if value > 0.0 {
                // Lavender color
                Color::Rgb {
                    r: 0xd5,
                    g: 0xb5,
                    b: 0xfc,
                }
            } else {
                // Green color
                Color::Rgb {
                    r: 0xb5,
                    g: 0xfc,
                    b: 0xd5,
                }
            };

            let fill_amount = ((value.abs() * FILLS.len() as f64) as usize).min(FILLS.len() - 1);
            let fill = FILLS[fill_amount].to_string().repeat(2);

            let x = idx % WIDTH;
            let y = idx / WIDTH;
            queue!(
                stdout(),
                cursor::MoveTo(start_col + x as u16 * 2, start_row + y as u16),
                style::PrintStyledContent(style(fill).with(color)),
            )?;
        }

        stdout().flush()?;
        Ok(())
    }
}
