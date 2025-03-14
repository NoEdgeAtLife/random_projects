use colorgrad::{Gradient, CustomGradient, Color};

#[derive(Clone)]
pub enum PaletteType {
    Rainbow,
    Fire,
    Ocean,
    Grayscale,
    Electric,
}

pub struct ColorPalette {
    pub palette_type: PaletteType,
    pub gradient: Gradient,
    pub cycle_colors: bool,
    pub color_offset: f64,
    pub color_scale: f64,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            palette_type: PaletteType::Rainbow,
            gradient: create_rainbow_gradient(),
            cycle_colors: true,
            color_offset: 0.0,
            color_scale: 1.0,
        }
    }
}

impl ColorPalette {
    pub fn new(palette_type: PaletteType) -> Self {
        let gradient = match palette_type.clone() {
            PaletteType::Rainbow => create_rainbow_gradient(),
            PaletteType::Fire => create_fire_gradient(),
            PaletteType::Ocean => create_ocean_gradient(),
            PaletteType::Grayscale => create_grayscale_gradient(),
            PaletteType::Electric => create_electric_gradient(),
        };

        Self {
            palette_type,
            gradient,
            cycle_colors: true,
            color_offset: 0.0,
            color_scale: 1.0,
        }
    }

    pub fn get_color(&self, iterations: usize, max_iterations: usize) -> [u8; 4] {
        if iterations >= max_iterations {
            return [0, 0, 0, 255]; // Black for points in the set
        }

        let t = if self.cycle_colors {
            // Smooth and cycle the coloring
            let smooth_val = iterations as f64 + 1.0 - (iterations as f64).ln().ln() / (2.0_f64).ln();
            let normalized = (smooth_val * 0.05 * self.color_scale + self.color_offset) % 1.0;
            normalized
        } else {
            // Linear mapping from iterations to color
            (iterations as f64 / max_iterations as f64) * self.color_scale + self.color_offset
        };

        let rgba = self.gradient.at(t).to_rgba8();
        [rgba[0], rgba[1], rgba[2], 255]
    }

    pub fn update_palette(&mut self, palette_type: PaletteType) {
        self.palette_type = palette_type.clone();
        self.gradient = match palette_type {
            PaletteType::Rainbow => create_rainbow_gradient(),
            PaletteType::Fire => create_fire_gradient(),
            PaletteType::Ocean => create_ocean_gradient(),
            PaletteType::Grayscale => create_grayscale_gradient(),
            PaletteType::Electric => create_electric_gradient(),
        };
    }
}

fn create_rainbow_gradient() -> Gradient {
    CustomGradient::new()
        .colors(&[
            Color::from_rgba8(148, 0, 211, 255),   // Violet
            Color::from_rgba8(75, 0, 130, 255),    // Indigo
            Color::from_rgba8(0, 0, 255, 255),     // Blue
            Color::from_rgba8(0, 255, 0, 255),     // Green
            Color::from_rgba8(255, 255, 0, 255),   // Yellow
            Color::from_rgba8(255, 0, 0, 255),     // Red
        ])
        .domain(&[0.0, 0.2, 0.4, 0.6, 0.8, 1.0])
        .build()
        .unwrap()
}

fn create_fire_gradient() -> Gradient {
    CustomGradient::new()
        .colors(&[
            Color::from_rgba8(0, 0, 0, 255),       // Black
            Color::from_rgba8(128, 0, 0, 255),     // Dark Red
            Color::from_rgba8(255, 0, 0, 255),     // Red
            Color::from_rgba8(255, 128, 0, 255),   // Orange
            Color::from_rgba8(255, 255, 0, 255),   // Yellow
            Color::from_rgba8(255, 255, 255, 255), // White
        ])
        .domain(&[0.0, 0.2, 0.4, 0.6, 0.8, 1.0])
        .build()
        .unwrap()
}

fn create_ocean_gradient() -> Gradient {
    CustomGradient::new()
        .colors(&[
            Color::from_rgba8(0, 0, 32, 255),      // Deep Blue
            Color::from_rgba8(0, 0, 128, 255),    // Navy Blue
            Color::from_rgba8(0, 128, 255, 255),   // Azure
            Color::from_rgba8(0, 255, 255, 255),  // Cyan
            Color::from_rgba8(240, 255, 255, 255), // Light Cyan
        ])
        .domain(&[0.0, 0.25, 0.5, 0.75, 1.0])
        .build()
        .unwrap()
}

fn create_grayscale_gradient() -> Gradient {
    CustomGradient::new()
        .colors(&[
            Color::from_rgba8(0, 0, 0, 255),       // Black
            Color::from_rgba8(255, 255, 255, 255), // White
        ])
        .domain(&[0.0, 1.0])
        .build()
        .unwrap()
}

fn create_electric_gradient() -> Gradient {
    CustomGradient::new()
        .colors(&[
            Color::from_rgba8(0, 0, 0, 255),       // Black
            Color::from_rgba8(32, 0, 50, 255),    // Dark Purple
            Color::from_rgba8(64, 0, 128, 255),    // Purple
            Color::from_rgba8(0, 0, 255, 255),    // Blue
            Color::from_rgba8(50, 255, 255, 255),  // Cyan
            Color::from_rgba8(200, 255, 50, 255), // Light Green
            Color::from_rgba8(255, 255, 0, 255),   // Yellow
            Color::from_rgba8(255, 255, 255, 255), // White
        ])
        .domain(&[0.0, 0.15, 0.3, 0.45, 0.6, 0.75, 0.9, 1.0])
        .build()
        .unwrap()
}

impl Clone for ColorPalette {
    fn clone(&self) -> Self {
        ColorPalette::new(match self.palette_type {
            PaletteType::Rainbow => PaletteType::Rainbow,
            PaletteType::Fire => PaletteType::Fire,
            PaletteType::Ocean => PaletteType::Ocean,
            PaletteType::Grayscale => PaletteType::Grayscale,
            PaletteType::Electric => PaletteType::Electric,
        })
    }
}