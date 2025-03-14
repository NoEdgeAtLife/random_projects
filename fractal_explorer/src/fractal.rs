use num_complex::Complex;
use rayon::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub enum FractalType {
    Mandelbrot,
    Julia,
    BurningShip,
}

#[derive(Clone)]
pub struct FractalParams {
    pub fractal_type: FractalType,
    pub max_iterations: usize,
    pub escape_radius: f64,
    pub julia_constant: Complex<f64>,
    pub zoom: f64,
    pub center_x: f64,
    pub center_y: f64,
}

impl Default for FractalParams {
    fn default() -> Self {
        Self {
            fractal_type: FractalType::Mandelbrot,
            max_iterations: 1000,
            escape_radius: 2.0,
            julia_constant: Complex::new(-0.7, 0.27015),
            zoom: 1.0,
            center_x: 0.0,
            center_y: 0.0,
        }
    }
}

pub fn calculate_fractal(
    width: usize,
    height: usize,
    params: Arc<FractalParams>
) -> Vec<usize> {
    let mut data = vec![0; width * height];
    
    let aspect_ratio = width as f64 / height as f64;
    let scale_x = 3.0 / params.zoom;
    let scale_y = 3.0 / (params.zoom * aspect_ratio);
    
    data.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
        for x in 0..width {
            // Scale pixel coordinates to the complex plane
            let scaled_x = params.center_x + (x as f64 / width as f64 - 0.5) * scale_x;
            let scaled_y = params.center_y + (y as f64 / height as f64 - 0.5) * scale_y;
            
            // Calculate iterations for this point
            let iterations = match params.fractal_type {
                FractalType::Mandelbrot => mandelbrot_iterations(
                    Complex::new(scaled_x, scaled_y),
                    params.max_iterations,
                    params.escape_radius,
                ),
                FractalType::Julia => julia_iterations(
                    Complex::new(scaled_x, scaled_y),
                    params.julia_constant,
                    params.max_iterations,
                    params.escape_radius,
                ),
                FractalType::BurningShip => burning_ship_iterations(
                    Complex::new(scaled_x, scaled_y),
                    params.max_iterations,
                    params.escape_radius,
                ),
            };
            
            row[x] = iterations;
        }
    });
    
    data
}

fn mandelbrot_iterations(c: Complex<f64>, max_iterations: usize, escape_radius: f64) -> usize {
    let mut z = Complex::new(0.0, 0.0);
    let escape_radius_squared = escape_radius * escape_radius;
    
    for i in 0..max_iterations {
        z = z * z + c;
        if z.norm_sqr() > escape_radius_squared {
            return i;
        }
    }
    
    max_iterations
}

fn julia_iterations(z: Complex<f64>, c: Complex<f64>, max_iterations: usize, escape_radius: f64) -> usize {
    let mut z = z;
    let escape_radius_squared = escape_radius * escape_radius;
    
    for i in 0..max_iterations {
        z = z * z + c;
        if z.norm_sqr() > escape_radius_squared {
            return i;
        }
    }
    
    max_iterations
}

fn burning_ship_iterations(c: Complex<f64>, max_iterations: usize, escape_radius: f64) -> usize {
    let mut z = Complex::new(0.0, 0.0);
    let escape_radius_squared = escape_radius * escape_radius;
    
    for i in 0..max_iterations {
        // Take absolute values of real and imaginary parts before squaring
        // Use explicit f64 casts to avoid ambiguity
        let re = z.re;
        let im = z.im;
        let z_abs = Complex::new(if re < 0.0 { -re } else { re }, if im < 0.0 { -im } else { im });
        z = z_abs * z_abs + c;
        
        if z.norm_sqr() > escape_radius_squared {
            return i;
        }
    }
    
    max_iterations
}

// Convert screen coordinates to complex plane
pub fn screen_to_complex(
    screen_x: f64,
    screen_y: f64,
    width: usize,
    height: usize,
    params: &FractalParams,
) -> Complex<f64> {
    let aspect_ratio = width as f64 / height as f64;
    let scale_x = 3.0 / params.zoom;
    let scale_y = 3.0 / (params.zoom * aspect_ratio);
    
    let re = params.center_x + (screen_x / width as f64 - 0.5) * scale_x;
    let im = params.center_y + (screen_y / height as f64 - 0.5) * scale_y;
    
    Complex::new(re, im)
}