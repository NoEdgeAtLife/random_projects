# Random Projects

A collection of random coding projects and experiments.

## Projects

### Hypnotic Spiral

A terminal-based animation that creates a mesmerizing hypnotic spiral using ASCII characters and colors.

**Features:**
- Expanding and contracting spiral pattern
- Random ASCII characters for visual texture
- Dynamic color changes
- Responsive to terminal size

**How to run:**
```bash
cd hypnotic_spiral
cargo run
```

**Controls:**
- Press `q` or `Esc` to exit the animation

**Dependencies:**
- crossterm (terminal manipulation)
- rand (random number generation)

### Fractal Explorer

An interactive fractal visualization application that allows users to explore various types of fractals including Mandelbrot and Julia sets with real-time parameter manipulation.

**Features:**
- Multiple fractal types (Mandelbrot, Julia, Burning Ship)
- Real-time color palette adjustments
- Zoom and pan capabilities
- Parameter manipulation for Julia set variations
- Export high-resolution images
- Multi-threaded rendering for improved performance

**How to run:**
```bash
cd fractal_explorer
cargo run --release
```

**Controls:**
- Mouse wheel: Zoom in/out
- Left click + drag: Pan around
- Right click: Generate Julia set from selected point
- Space: Toggle UI controls
- S: Save current view as PNG
- R: Reset to default view
- 1-5: Switch between different color palettes
- Escape: Exit application

**Dependencies:**
- egui (for GUI components)
- image (for image export)
- rayon (for parallel processing)
- num-complex (complex number calculations)

### Particle Life

An artificial life simulation where particles interact with each other based on attraction/repulsion rules, creating emergent behavior and patterns.

**Features:**
- 5000 particles with 5 different types
- GPU-accelerated rendering using WGPU
- Particle interactions based on type-specific attraction/repulsion rules
- Smooth, real-time simulation with parallel processing
- Wrapping boundaries (particles move across screen edges)

**How to run:**
```bash
cd particle_life
cargo run --release
```

**Controls:**
- Escape: Exit the simulation

**Dependencies:**
- wgpu (GPU rendering)
- winit (window management)
- rayon (parallel processing)
- cgmath (vector mathematics)

### Word Guesser

A text-based word guessing game similar to hangman, where you try to guess a programming-related word with a limited number of attempts.

**Features:**
- Collection of programming-related words
- Colored terminal feedback for correct and incorrect guesses
- Option to guess a single letter or the entire word
- Tracking of previously guessed letters
- Limited attempts (6) to guess the word

**How to run:**
```bash
cd word_guesser
cargo run
```

**How to play:**
- Enter a single letter to guess if it's in the word
- Enter a complete word to attempt to guess the full solution
- Try to solve the word before you run out of attempts!

**Dependencies:**
- rand (random word selection)
- colored (terminal text coloring)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
