use piston_window::{clear, rectangle, Event, PistonWindow, Window, WindowSettings};

pub struct Graphics {
    window: PistonWindow,
    pixels: [[bool; 64]; 32],
}

impl Graphics {
    pub fn new() -> Self {
        let window: PistonWindow = WindowSettings::new("Chip8 Emulator", [640, 320])
            .automatic_close(true)
            .build()
            .unwrap();

        Graphics {
            window,
            pixels: [[false; 64]; 32] // Array of (Array of false * 64) * 32
        }
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, is_active: bool) {
        self.pixels[y as usize][x as usize] = is_active;
    }

    pub fn get_pixel(&mut self, x: u8, y: u8) -> bool {
        self.pixels[y as usize][x as usize]
    }

    pub fn clear_pixels(&mut self) {
        self.pixels = [[false; 64]; 32];
    }

    pub fn render_window(&mut self) -> Option<()> {
        if let Some(event) = self.window.next() {
            self.window.draw_2d(&event, |context, g, _device| {
                clear([0.0, 0.0, 0.0, 0.0], g);

                // for pixel in self.pixels {
                //     for p in pixel {
                //         print!("{} ", if p { 1 } else { 0 });
                //     }
                //     println!();
                // }
                // println!("=======================================");

                // Draw the active pixels only.
                for (row_index, row) in self.pixels.iter().enumerate() {
                    for (col_index, is_active) in row.iter().enumerate() {
                        let cell_color = if *is_active { 1.0 } else { 0.0 };
                        let rect = [col_index as f64 * 10.0, row_index as f64 * 10.0, 10.0, 10.0];
                        rectangle(
                            [cell_color, cell_color, cell_color, 1.0],
                            rect,
                            context.transform,
                            g,
                        )
                    }
                }
                // rectangle([1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 100.0, 100.0], context.transform, g);
            });
            return Some(());
        } else {
            return None;
        }
    }
}
