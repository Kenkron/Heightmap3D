#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::fs::File;
use std::sync::Arc;
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
    renderable_mesh: Option<Arc<mesh_view::RenderableMesh>>,
    view_3d: mesh_view::MeshView,
    gl: Arc<glow::Context>,
    heightmap_path: Option<String>,
    heightmap: Option<Heightmap>,
    error: Option<String>
}

impl AppState {
    fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            renderable_mesh: None,
            view_3d: mesh_view::MeshView::new(gl.to_owned(), Vec2::new(400., 400.)).unwrap(),
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
                    self.heightmap_path = Some(rfd_result.display().to_string());
                    self.error = None;
                    match read_heightmap(File::open(rfd_result.display().to_string()).unwrap()) {
                        Err(_) => {
                            self.error = Some("Error Exporting".to_string());
                            self.heightmap = None;
                        },
                        Ok(heightmap) => {
                            self.heightmap = Some(heightmap);
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
            // If there's an error, don't show any option bug file selection
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
                    self.renderable_mesh = 
                        Some(Arc::new(
                            mesh_view::RenderableMesh::new(
                                mesh_gl, heightmap_mesh).unwrap()));
                    self.view_3d.translation = Vec3::new(
                        -heightmap.size.x as f32 * heightmap.scale.x * 0.5,
                        -heightmap.size.y as f32 * heightmap.scale.y * 0.5,
                        0.0
                    );
                }
                if let Some(mesh) = &self.renderable_mesh {
                    let mut style = (*ctx.style()).clone();
                    style.spacing.slider_width = 400.;
                    ctx.set_style(style);
                    ui.vertical_centered(|ui| {
                        self.view_3d.show_mesh(ui, mesh.to_owned());
                        ui.add(egui::Slider::new(&mut self.view_3d.scale, 0.0..=2.0));
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
