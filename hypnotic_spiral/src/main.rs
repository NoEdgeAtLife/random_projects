use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
    style::{Color, SetForegroundColor},
};
use rand::Rng;
use std::{
    io::{stdout, Write, Result},
    time::{Duration, Instant},
};

fn main() -> Result<()> {
    // Enter alternate screen
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;

    // Hide cursor
    execute!(stdout, cursor::Hide)?;

    // Get terminal size
    let (width, height) = terminal::size()?;

    // Spiral settings
    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;
    let mut radius = 1.0;
    let mut angle: f64 = 0.0;
    let expansion_factor = 0.05;
    let mut direction = 1.0;
    let mut iterations = 0;
    let max_radius = (width.min(height) as f64) / 2.0;

    // Animation characters
    let chars = vec!['*', '+', '.', '@', '#', '$', '%', '&', '=', '~'];
    let mut rng = rand::thread_rng();

    let mut last_color_change = Instant::now();
    let mut current_color = Color::White;
    let colors = vec![
        Color::Red, Color::Green, Color::Blue, Color::Yellow, 
        Color::Magenta, Color::Cyan, Color::White
    ];
    
    'animation: loop {
        // Check for key press to exit
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.code == KeyCode::Char('q') || key_event.code == KeyCode::Esc {
                    break 'animation;
                }
            }
        }

        // Clear screen every few iterations for a more hypnotic effect
        if iterations % 50 == 0 {
            execute!(stdout, terminal::Clear(ClearType::All))?;
        }

        // Change color periodically
        if last_color_change.elapsed() > Duration::from_millis(300) {
            current_color = *colors.choose(&mut rng).unwrap_or(&Color::White);
            last_color_change = Instant::now();
        }

        // Draw a point on the spiral
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();
        
        if x >= 0.0 && x < width as f64 && y >= 0.0 && y < height as f64 {
            // Choose a random character for the spiral point
            let ch = chars[rng.gen_range(0..chars.len())];
            
            execute!(
                stdout,
                cursor::MoveTo(x as u16, y as u16),
                SetForegroundColor(current_color),
            )?;
            write!(stdout, "{}", ch)?;
            stdout.flush()?;
        }

        // Update spiral parameters
        angle += 0.1;
        radius += expansion_factor * direction;

        // Reverse direction if reaching limits
        if radius >= max_radius || radius <= 1.0 {
            direction *= -1.0;
        }

        // Sleep for a short time to control animation speed
        std::thread::sleep(Duration::from_millis(5));
        iterations += 1;
    }

    // Cleanup
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

trait ArrayChoose<T> {
    fn choose(&self, rng: &mut impl Rng) -> Option<&T>;
}

impl<T> ArrayChoose<T> for Vec<T> {
    fn choose(&self, rng: &mut impl Rng) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self[rng.gen_range(0..self.len())])
        }
    }
}
