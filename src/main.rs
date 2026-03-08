// alive ██, dead 2 spaces

use rand;
use std::fmt;
use std::io::{self, BufWriter, Write};
use std::{thread, time};
use terminal_size::{Height, Width, terminal_size};

const GENERATION_TIMEOUT_MS: u64 = 100;

fn main() {
    // Set up handler for Ctrl+C
    ctrlc::set_handler(|| {
        let mut stdout = std::io::stdout();
        let _ = write!(stdout, "\x1B[?25h"); // Enable cursor
        let _ = stdout.flush();
        std::process::exit(0);
    })
    .expect("Error setting Ctrl+C handler");

    // Hide cursor
    print!("\x1B[?25l");

    // Get terminal window dimensions
    let (term_width, term_height) = if let Some((Width(w), Height(h))) = terminal_size() {
        (w as usize, h as usize)
    } else {
        // Fallback dimensions
        (80, 24)
    };

    let mut screen: Screen = Screen::new(term_width / 2, term_height); // /2 because 2 chars are used to draw dead or alive cell

    // Randomly populate the screen with dead and alive cells
    for y in 0..screen.height {
        for x in 0..screen.width {
            screen.current[y][x] = rand::random_bool(0.5);
        }
    }

    loop {
        draw(&mut screen);
        update(&mut screen);
        thread::sleep(time::Duration::from_millis(GENERATION_TIMEOUT_MS))
    }
}

fn update(screen: &mut Screen) {
    // Four rules:
    // 1. Any live cell with fewer than two live neighbours dies.
    // 2. Any live cell with two or three live neighbours lives on to the next generation.
    // 3. Any live cell with more than three live neighbours dies, as if by overpopulation.
    // 4. Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.

    let grid = &screen.current;

    for y in 0..screen.height {
        for x in 0..screen.width {
            let mut num_alive_neighbours: i32 = 0;
            let x_minus_1 = (screen.width + x - 1) % screen.width;
            let y_minus_1 = (screen.height + y - 1) % screen.height;
            let x_plus_1 = (x + 1) % screen.width;
            let y_plus_1 = (y + 1) % screen.height;

            if grid[y_minus_1][x_minus_1] {
                num_alive_neighbours += 1;
            }
            if grid[y][x_minus_1] {
                num_alive_neighbours += 1;
            }
            if grid[y_plus_1][x_minus_1] {
                num_alive_neighbours += 1;
            }
            if grid[y_minus_1][x] {
                num_alive_neighbours += 1;
            }
            if grid[y_plus_1][x] {
                num_alive_neighbours += 1;
            }
            if grid[y_minus_1][x_plus_1] {
                num_alive_neighbours += 1;
            }
            if grid[y][x_plus_1] {
                num_alive_neighbours += 1;
            }
            if grid[y_plus_1][x_plus_1] {
                num_alive_neighbours += 1;
            }

            let old_cell = grid[y][x];
            let mut new_cell = false;
            if (old_cell && (num_alive_neighbours == 3 || num_alive_neighbours == 2))
                || (!old_cell && num_alive_neighbours == 3)
            {
                new_cell = true;
            }
            screen.next[y][x] = new_cell;
        }
    }
    std::mem::swap(&mut screen.current, &mut screen.next);
}

fn draw(screen: &mut Screen) {
    // To prevent writing line by line and force the whole buffer to be
    // written at the same time
    let stdout = io::stdout();
    let lock = stdout.lock();
    let mut buffer = BufWriter::new(lock);
    print!("\x1B[1;1H"); // Move cursor to top left corner 
    print!("{}", screen);
    let _ = buffer.flush();
}

struct Screen {
    current: Vec<Vec<bool>>, // Matrix to store dead and alive cells
    next: Vec<Vec<bool>>, // Matrix to store dead and alive cells for the next generation
    width: usize,
    height: usize,
}

impl Screen {
    fn new(width: usize, height: usize) -> Self {
        Self {
            current: vec![vec![false; width]; height],
            next: vec![vec![false; width]; height],
            width,
            height,
        }
    }
}

impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, row) in self.current.iter().enumerate() {
            for cell in row {
                if *cell {
                    write!(f, "██")?;
                } else {
                    write!(f, "  ")?;
                }
            }

            if i < self.current.len() - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
