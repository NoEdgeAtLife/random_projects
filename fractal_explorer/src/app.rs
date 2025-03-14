use crate::color_palette::{ColorPalette, PaletteType};
use crate::fractal::{self, FractalParams, FractalType};
use eframe::egui::{self, Context, Key, PointerButton, RichText, Sense};
use eframe::{epaint::ColorImage, Frame};
use egui::TextureHandle;
use num_complex::Complex;
use rfd::FileDialog;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct FractalExplorer {
    fractal_params: Arc<FractalParams>,
    fractal_data: Arc<Mutex<Vec<usize>>>,
    color_palette: ColorPalette,
    texture: Option<TextureHandle>,
    texture_size: (usize, usize),
    show_ui: bool,
    is_rendering: Arc<Mutex<bool>>,
    render_status: String,
}

impl FractalExplorer {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Enable custom fonts
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(20.0, egui::FontFamily::Proportional)
        );
        cc.egui_ctx.set_style(style);

        let fractal_params = Arc::new(FractalParams::default());
        
        Self {
            fractal_params,
            fractal_data: Arc::new(Mutex::new(Vec::new())),
            color_palette: ColorPalette::default(),
            texture: None,
            texture_size: (0, 0),
            show_ui: true,
            is_rendering: Arc::new(Mutex::new(false)),
            render_status: "Ready".to_string(),
        }
    }

    fn render_fractal(&mut self, ctx: &Context, size: [usize; 2]) {
        // Start a new render only if not already rendering
        if *self.is_rendering.lock().unwrap() {
            return;
        }

        // Check if we need to re-render
        if self.texture_size.0 != size[0] || self.texture_size.1 != size[1] || self.texture.is_none() {
            *self.is_rendering.lock().unwrap() = true;
            self.render_status = "Rendering...".to_string();

            // Update texture size
            self.texture_size = (size[0], size[1]);
            
            // Create clones for the thread
            let fractal_params = Arc::clone(&self.fractal_params);
            let fractal_data = Arc::clone(&self.fractal_data);
            let is_rendering = Arc::clone(&self.is_rendering);
            let ctx = ctx.clone();
            
            // Create a new color palette for the thread
            let color_palette = self.color_palette.clone();
            let max_iterations = fractal_params.max_iterations;
            
            // Start rendering in a background thread
            thread::spawn(move || {
                // Calculate fractal data
                let data = fractal::calculate_fractal(size[0], size[1], fractal_params);
                
                // Store fractal data for potential reuse
                *fractal_data.lock().unwrap() = data.clone();

                // Create image for egui
                let mut pixels = vec![0u8; size[0] * size[1] * 4];
                
                for y in 0..size[1] {
                    for x in 0..size[0] {
                        let idx = y * size[0] + x;
                        let iterations = data[idx];
                        let color = color_palette.get_color(iterations, max_iterations);
                        
                        let pixel_idx = idx * 4;
                        pixels[pixel_idx] = color[0];
                        pixels[pixel_idx + 1] = color[1];
                        pixels[pixel_idx + 2] = color[2];
                        pixels[pixel_idx + 3] = color[3];
                    }
                }
                
                // Create image - no need to retain the reference but we still need to create it
                let _color_image = ColorImage::from_rgba_unmultiplied([size[0], size[1]], &pixels);
                
                // Update UI on the main thread
                ctx.request_repaint();
                
                // Mark rendering as complete
                *is_rendering.lock().unwrap() = false;
            });
        }
    }

    fn create_or_update_texture(&mut self, ctx: &Context) -> Result<(), String> {
        if self.texture.is_none() && self.texture_size.0 > 0 && self.texture_size.1 > 0 {
            // Create a placeholder texture
            let size = [self.texture_size.0, self.texture_size.1];
            let pixels = vec![128u8; size[0] * size[1] * 4]; // Gray placeholder
            let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);
            self.texture = Some(ctx.load_texture(
                "fractal_image", 
                color_image,
                Default::default()
            ));
        }
        
        if let Some(texture_handle) = &mut self.texture {
            if !self.fractal_data.lock().unwrap().is_empty() && !*self.is_rendering.lock().unwrap() {
                let data = self.fractal_data.lock().unwrap().clone();
                let max_iterations = self.fractal_params.max_iterations;
                
                let size = [self.texture_size.0, self.texture_size.1];
                let mut pixels = vec![0u8; size[0] * size[1] * 4];
                
                for y in 0..size[1] {
                    for x in 0..size[0] {
                        let idx = y * size[0] + x;
                        let iterations = data[idx];
                        let color = self.color_palette.get_color(iterations, max_iterations);
                        
                        let pixel_idx = idx * 4;
                        pixels[pixel_idx] = color[0];
                        pixels[pixel_idx + 1] = color[1];
                        pixels[pixel_idx + 2] = color[2];
                        pixels[pixel_idx + 3] = color[3];
                    }
                }
                
                let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);
                *texture_handle = ctx.load_texture(
                    "fractal_image", 
                    color_image,
                    Default::default()
                );
                
                self.render_status = "Ready".to_string();
            }
        }
        
        Ok(())
    }

    fn handle_key_presses(&mut self, ctx: &Context) {
        let input = ctx.input(|i| i.clone());
        
        // Toggle UI visibility
        if input.key_pressed(Key::Space) {
            self.show_ui = !self.show_ui;
            ctx.request_repaint();
        }
        
        // Reset view
        if input.key_pressed(Key::R) {
            let mut new_params = FractalParams::default();
            new_params.fractal_type = self.fractal_params.fractal_type.clone();
            self.fractal_params = Arc::new(new_params);
            ctx.request_repaint();
        }
        
        // Save image - remove the Ctrl modifier requirement
        if input.key_pressed(Key::S) {
            self.save_image();
        }
        
        // Change color palette
        if input.key_pressed(Key::Num1) {
            self.color_palette.update_palette(PaletteType::Rainbow);
            ctx.request_repaint();
        } else if input.key_pressed(Key::Num2) {
            self.color_palette.update_palette(PaletteType::Fire);
            ctx.request_repaint();
        } else if input.key_pressed(Key::Num3) {
            self.color_palette.update_palette(PaletteType::Ocean);
            ctx.request_repaint();
        } else if input.key_pressed(Key::Num4) {
            self.color_palette.update_palette(PaletteType::Grayscale);
            ctx.request_repaint();
        } else if input.key_pressed(Key::Num5) {
            self.color_palette.update_palette(PaletteType::Electric);
            ctx.request_repaint();
        }
    }

    fn save_image(&self) {
        if self.texture_size.0 == 0 || self.texture_size.1 == 0 || self.fractal_data.lock().unwrap().is_empty() {
            return;
        }

        if let Some(path) = FileDialog::new()
            .add_filter("PNG Image", &["png"])
            .set_directory(".")
            .save_file() {
                
            let size = [self.texture_size.0, self.texture_size.1];
            let data = self.fractal_data.lock().unwrap().clone();
            let max_iterations = self.fractal_params.max_iterations;
            let color_palette = self.color_palette.clone(); // Clone for the thread
            
            // Generate image in a background thread
            thread::spawn(move || {
                let mut img_buffer = image::RgbaImage::new(size[0] as u32, size[1] as u32);
                
                for y in 0..size[1] {
                    for x in 0..size[0] {
                        let idx = y * size[0] + x;
                        let iterations = data[idx];
                        let color = color_palette.get_color(iterations, max_iterations);
                        
                        img_buffer.put_pixel(
                            x as u32, 
                            y as u32, 
                            image::Rgba([color[0], color[1], color[2], color[3]])
                        );
                    }
                }
                
                let _ = img_buffer.save(path);
            });
        }
    }
}

impl eframe::App for FractalExplorer {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // Handle key presses
        self.handle_key_presses(ctx);
        
        // Calculate available size
        let available_size = ctx.available_rect().size();
        let size = [
            available_size.x as usize, 
            available_size.y as usize
        ];
        
        // Render the fractal if needed
        self.render_fractal(ctx, size);
        
        // Update the texture if needed
        if let Err(err) = self.create_or_update_texture(ctx) {
            self.render_status = format!("Error: {}", err);
        }
        
        // Display the fractal image with interaction support
        if let Some(texture) = &self.texture {
            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    // Make the image interactive by adding a sense of click and drag
                    let response = ui.add(egui::Image::new(texture, available_size)
                        .sense(Sense::click_and_drag()));

                    // Handle mouse interactions directly here
                    let mut should_update_fractal = false;
                    let mut julia_point_selected = false;
                    let mut new_params = (*self.fractal_params).clone();

                    // Handle drag for panning
                    if response.dragged_by(PointerButton::Primary) {
                        let drag_delta = response.drag_delta();
                        let size = self.texture_size;
                        
                        if size.0 > 0 && size.1 > 0 {
                            // Scale the drag amount based on current zoom level
                            let dx = 2.0 * drag_delta.x as f64 * (3.0 / new_params.zoom) / size.0 as f64;
                            let dy = 2.0 * drag_delta.y as f64 * (3.0 / new_params.zoom) / size.1 as f64;
                            
                            new_params.center_x -= dx;
                            new_params.center_y -= dy;
                            
                            should_update_fractal = true;
                        }
                    }

                    // Get scroll info directly from the context
                    ctx.input(|i| {
                        // Handle zoom with scroll wheel - make it smoother
                        if i.scroll_delta.y != 0.0 && response.hovered() {
                            // Use a logarithmic zoom factor for smoother zoom
                            let zoom_delta = i.scroll_delta.y * 0.001;
                            let zoom_factor : f64 = (1.0 + zoom_delta).exp().into();
                            
                            if let Some(pointer_pos) = i.pointer.hover_pos() {
                                let size = self.texture_size;
                                if size.0 > 0 && size.1 > 0 {
                                    // Convert screen coordinates to complex plane coordinates
                                    let x_rel = pointer_pos.x as f64 / size.0 as f64;
                                    let y_rel = pointer_pos.y as f64 / size.1 as f64;
                                    
                                    // Calculate offsets in complex plane coordinates
                                    let x_offset = (x_rel - 0.5) * (6.0 / new_params.zoom);
                                    let y_offset = (y_rel - 0.5) * (6.0 / new_params.zoom);
                                    
                                    // Update center and zoom to zoom into the mouse position
                                    new_params.center_x += x_offset * (1.0 - 1.0 / zoom_factor);
                                    new_params.center_y += y_offset * (1.0 - 1.0 / zoom_factor);
                                    new_params.zoom *= zoom_factor;
                                    
                                    should_update_fractal = true;
                                }
                            }
                        }

                        // Handle right-click to select Julia set constant
                        if response.secondary_clicked() {
                            if let Some(pointer_pos) = i.pointer.hover_pos() {
                                let size = self.texture_size;
                                if size.0 > 0 && size.1 > 0 {
                                    let c = fractal::screen_to_complex(
                                        pointer_pos.x as f64, 
                                        pointer_pos.y as f64, 
                                        size.0, 
                                        size.1, 
                                        &new_params
                                    );
                                    
                                    new_params.julia_constant = c;
                                    new_params.fractal_type = FractalType::Julia;
                                    
                                    julia_point_selected = true;
                                    should_update_fractal = true;
                                }
                            }
                        }
                    });
                    
                    // Update the fractal if needed
                    if should_update_fractal {
                        self.fractal_params = Arc::new(new_params);
                        
                        if julia_point_selected {
                            // Reset view for a better Julia set exploration
                            let mut params = (*self.fractal_params).clone();
                            params.center_x = 0.0;
                            params.center_y = 0.0;
                            params.zoom = 1.0;
                            self.fractal_params = Arc::new(params);
                        }
                        
                        ctx.request_repaint();
                    }
                });
        }
        
        // Show UI controls
        if self.show_ui {
            egui::Window::new("Fractal Explorer")
                .resizable(false)
                .default_pos([10.0, 10.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Fractal Explorer");
                        ui.label(RichText::new("Press Space to toggle UI").italics());
                    });
                    
                    ui.separator();
                    
                    // Fractal type selection
                    ui.horizontal(|ui| {
                        ui.label("Fractal Type:");
                        let mut fractal_type_index = match self.fractal_params.fractal_type {
                            FractalType::Mandelbrot => 0,
                            FractalType::Julia => 1,
                            FractalType::BurningShip => 2,
                        };
                        
                        let fractal_types = ["Mandelbrot", "Julia", "Burning Ship"];
                        egui::ComboBox::from_label("")
                            .selected_text(fractal_types[fractal_type_index])
                            .show_ui(ui, |ui| {
                                let mut changed = false;
                                for (idx, name) in fractal_types.iter().enumerate() {
                                    changed |= ui.selectable_value(&mut fractal_type_index, idx, *name).changed();
                                }
                                if changed {
                                    let mut new_params = (*self.fractal_params).clone();
                                    new_params.fractal_type = match fractal_type_index {
                                        0 => FractalType::Mandelbrot,
                                        1 => FractalType::Julia,
                                        2 => FractalType::BurningShip,
                                        _ => FractalType::Mandelbrot,
                                    };
                                    self.fractal_params = Arc::new(new_params);
                                }
                            });
                    });
                    
                    // Max iterations
                    ui.horizontal(|ui| {
                        ui.label("Max Iterations:");
                        let mut max_iterations = self.fractal_params.max_iterations as i32;
                        if ui.add(egui::Slider::new(&mut max_iterations, 100..=10000)).changed() {
                            let mut new_params = (*self.fractal_params).clone();
                            new_params.max_iterations = max_iterations as usize;
                            self.fractal_params = Arc::new(new_params);
                        }
                    });
                    
                    if let FractalType::Julia = self.fractal_params.fractal_type {
                        ui.group(|ui| {
                            ui.label("Julia Set Parameters:");
                            let mut real = self.fractal_params.julia_constant.re;
                            let mut imag = self.fractal_params.julia_constant.im;
                            
                            let mut changed = false;
                            changed |= ui.add(egui::Slider::new(&mut real, -2.0..=2.0).text("Real")).changed();
                            changed |= ui.add(egui::Slider::new(&mut imag, -2.0..=2.0).text("Imaginary")).changed();
                            
                            if changed {
                                let mut new_params = (*self.fractal_params).clone();
                                new_params.julia_constant = Complex::new(real, imag);
                                self.fractal_params = Arc::new(new_params);
                            }
                        });
                    }
                    
                    // Color controls
                    ui.collapsing("Color Settings", |ui| {
                        // Color palette selection
                        ui.horizontal(|ui| {
                            ui.label("Color Palette:");
                            let mut palette_index = match self.color_palette.palette_type {
                                PaletteType::Rainbow => 0,
                                PaletteType::Fire => 1,
                                PaletteType::Ocean => 2,
                                PaletteType::Grayscale => 3,
                                PaletteType::Electric => 4,
                            };
                            
                            let palette_names = ["Rainbow", "Fire", "Ocean", "Grayscale", "Electric"];
                            egui::ComboBox::from_label("")
                                .selected_text(palette_names[palette_index])
                                .show_ui(ui, |ui| {
                                    let mut changed = false;
                                    for (idx, name) in palette_names.iter().enumerate() {
                                        changed |= ui.selectable_value(&mut palette_index, idx, *name).changed();
                                    }
                                    if changed {
                                        self.color_palette.update_palette(match palette_index {
                                            0 => PaletteType::Rainbow,
                                            1 => PaletteType::Fire,
                                            2 => PaletteType::Ocean,
                                            3 => PaletteType::Grayscale,
                                            4 => PaletteType::Electric,
                                            _ => PaletteType::Rainbow,
                                        });
                                    }
                                });
                        });
                        
                        // Color cycling and offset
                        ui.checkbox(&mut self.color_palette.cycle_colors, "Cycle Colors");
                        
                        ui.horizontal(|ui| {
                            ui.label("Color Offset:");
                            if ui.add(egui::Slider::new(&mut self.color_palette.color_offset, 0.0..=1.0)).changed() {
                                // Color will update automatically
                            }
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Color Scale:");
                            if ui.add(egui::Slider::new(&mut self.color_palette.color_scale, 0.1..=5.0)).changed() {
                                // Color will update automatically
                            }
                        });
                    });
                    
                    ui.separator();
                    
                    // Controls help
                    ui.collapsing("Controls", |ui| {
                        ui.label("• Mouse wheel: Zoom in/out");
                        ui.label("• Left click + drag: Pan around");
                        ui.label("• Right click: Generate Julia set from point");
                        ui.label("• Space: Toggle UI controls");
                        ui.label("• S: Save current view as PNG");
                        ui.label("• R: Reset to default view");
                        ui.label("• 1-5: Switch between color palettes");
                        ui.label("• Esc: Exit application");
                    });
                    
                    ui.separator();
                    
                    // Current coordinates and status
                    if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                        let size = self.texture_size;
                        if size.0 > 0 && size.1 > 0 {
                            let c = fractal::screen_to_complex(pos.x as f64, pos.y as f64, size.0, size.1, &self.fractal_params);
                            ui.horizontal(|ui| {
                                ui.label(format!("Position: {:.6} + {:.6}i", c.re, c.im));
                                if ui.button("Set as Julia").clicked() {
                                    let mut new_params = (*self.fractal_params).clone();
                                    new_params.julia_constant = c;
                                    new_params.fractal_type = FractalType::Julia;
                                    self.fractal_params = Arc::new(new_params);
                                }
                            });
                        }
                    }
                    
                    ui.label(format!("Status: {}", self.render_status));
                    
                    // Save image button
                    if ui.button("Save Image").clicked() {
                        self.save_image();
                    }
                });
        }
    }
}