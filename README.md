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

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
