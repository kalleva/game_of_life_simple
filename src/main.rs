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

    let mut screen: Screen = Screen::new(term_width / 2, term_height);

    // Randomly populate the screen with dead and alive cells
    for y in 0..screen.height {
        for x in 0..screen.width {
            screen.grid1[y][x] = rand::random_bool(0.5);
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

    let (old_grid, new_grid) = if screen.grid2_used {
        (&screen.grid2, &mut screen.grid1)
    } else {
        (&screen.grid1, &mut screen.grid2)
    };

    for y in 0..screen.height {
        for x in 0..screen.width {
            let mut num_alive_neighbours: i32 = 0;
            let x_minus_1 = (screen.width + x - 1) % screen.width;
            let y_minus_1 = (screen.height + y - 1) % screen.height;
            let x_plus_1 = (x + 1) % screen.width;
            let y_plus_1 = (y + 1) % screen.height;

            if old_grid[y_minus_1][x_minus_1] {
                num_alive_neighbours += 1;
            }
            if old_grid[y][x_minus_1] {
                num_alive_neighbours += 1;
            }
            if old_grid[y_plus_1][x_minus_1] {
                num_alive_neighbours += 1;
            }
            if old_grid[y_minus_1][x] {
                num_alive_neighbours += 1;
            }
            if old_grid[y_plus_1][x] {
                num_alive_neighbours += 1;
            }
            if old_grid[y_minus_1][x_plus_1] {
                num_alive_neighbours += 1;
            }
            if old_grid[y][x_plus_1] {
                num_alive_neighbours += 1;
            }
            if old_grid[y_plus_1][x_plus_1] {
                num_alive_neighbours += 1;
            }

            let old_cell = old_grid[y][x];
            let mut new_cell = false;
            if (old_cell && (num_alive_neighbours == 3 || num_alive_neighbours == 2))
                || (!old_cell && num_alive_neighbours == 3)
            {
                new_cell = true;
            }
            new_grid[y][x] = new_cell;
        }
    }
    screen.grid2_used = !screen.grid2_used;
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
    grid1: Vec<Vec<bool>>, // Matrix to store dead and alive cells
    grid2: Vec<Vec<bool>>, // Matrix to store dead and alive cells
    grid2_used: bool,
    width: usize,  // grid width
    height: usize, // grid height
}

impl Screen {
    fn new(width: usize, height: usize) -> Self {
        Self {
            grid1: vec![vec![false; width]; height],
            grid2: vec![vec![false; width]; height],
            grid2_used: false,
            width,
            height,
        }
    }
}

impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let grid = if self.grid2_used {
            &self.grid2
        } else {
            &self.grid1
        };
        for (i, row) in grid.iter().enumerate() {
            for cell in row {
                if *cell {
                    write!(f, "██")?;
                } else {
                    write!(f, "  ")?;
                }
            }

            if i < grid.len() - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
