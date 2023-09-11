#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::fs::File;
use std::sync::Arc;
use std::sync::Mutex;
use eframe::egui;
use egui::Vec2;
use nalgebra_glm::Vec3;
mod geometry;
use crate::geometry::triangle::*;
use crate::geometry::heightmap::*;
use eframe::egui_glow;
use egui_glow::glow;
mod mesh_view;

struct AppState {
    renderable_mesh: Option<Arc<Mutex<mesh_view::RenderableMesh>>>,
    gl: Arc<glow::Context>,
    heightmap_path: Option<String>,
    heightmap: Option<Heightmap>,
    error: Option<String>
}

impl AppState {
    fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            renderable_mesh: None,
            gl: gl,
            heightmap_path: None,
            heightmap: None,
            error: None
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Heightmap To STL");

            // File Selection
            if ui.button("Select File").clicked() {
                if let Some(rfd_result) = rfd::FileDialog::new().pick_file() {
                    let path = rfd_result.display().to_string();
                    self.heightmap_path = Some(path.clone());
                    self.error = None;
                    if path.ends_with(".png") {
                        match read_heightmap_image(&path) {
                            Err(_) => {
                                self.error = Some("Error Exporting".to_string());
                                self.heightmap = None;
                            },
                            Ok(heightmap) => {
                                self.heightmap = Some(heightmap);
                                self.renderable_mesh = None;
                            }
                        }
                    } else {
                        match read_heightmap(File::open(path).unwrap()) {
                            Err(_) => {
                                self.error = Some("Error Exporting".to_string());
                                self.heightmap = None;
                            },
                            Ok(heightmap) => {
                                self.heightmap = Some(heightmap);
                                self.renderable_mesh = None;
                            }
                        }
                    }
                }
            }

            ui.horizontal(|ui| {
                if let Some(heightmap_path) = &self.heightmap_path {
                    ui.label("File: ");
                    ui.monospace(heightmap_path);
                }
            });

            // Error Message
            // If there's an error, don't show any option but file selection
            if let Some(error) = &self.error {
                ui.label("Error:");
                ui.monospace(error);
                return;
            }

            if let Some(heightmap) = &self.heightmap {
                if ui.button("Export").clicked() {

                    if let Some(rfd_result) = rfd::FileDialog::new().save_file() {
                        let output_file = rfd_result.display().to_string();
                        let triangles = heightmap.get_triangles();
                        match write_stl_binary(output_file, &triangles) {
                            Err(_) => {
                                self.error = Some("Error Exporting".to_string());
                            },
                            Ok(_) => {}
                        };
                    }
                }
                if let None = self.renderable_mesh {
                    let mesh_gl = self.gl.to_owned();
                    let heightmap_mesh = heightmap.get_triangles();
                    let mut mesh = mesh_view::RenderableMesh::new(mesh_gl, &heightmap_mesh).unwrap();
                    mesh.translation = Vec3::new(
                        -heightmap.size.x as f32 * heightmap.scale.x * 0.5,
                        -heightmap.size.y as f32 * heightmap.scale.y * 0.5,
                        0.0
                    );
                    self.renderable_mesh = Some(Arc::new(Mutex::new(mesh)));
                }
                if let Some(mesh) = &self.renderable_mesh {
                    let mut style = (*ctx.style()).clone();
                    style.spacing.slider_width = 350.;
                    ctx.set_style(style);
                    ui.vertical_centered(|ui| {
                        ui.add(mesh_view::MeshView::new(Vec2::new(400., 400.), mesh.to_owned()));
                        ui.horizontal(|ui| {
                            let mut mesh = mesh.lock().unwrap();
                            if ui.button("reset").clicked() {
                                mesh.scale = 1.0;
                                mesh.reset_rotation();
                                mesh.translation = Vec3::new(
                                    -heightmap.size.x as f32 * heightmap.scale.x * 0.5,
                                    -heightmap.size.y as f32 * heightmap.scale.y * 0.5,
                                    0.0
                                );
                            }
                            ui.add(egui::Slider::new(&mut mesh.scale, 0.0..=2.0));
                        })
                    });
                }
            }
        });
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        let file = File::open(args[1].to_owned()).expect("Failed to open heightmap file");
        let heightmap = read_heightmap(file).expect("Failed to parse heightmap file");
        let triangles = heightmap.get_triangles();
        write_stl_binary(args[2].to_owned(), &triangles).expect("Error saving STL");
    } else {
        let mut options = eframe::NativeOptions::default();
        options.initial_window_size = Some(egui::vec2(500., 600.));
        eframe::run_native(
            "Heightmap To STL",
            options,
            Box::new(|cc|
                Box::new(AppState::new(cc.gl.to_owned().expect("Could not get gl context"))))
        )
    }
}
