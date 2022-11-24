use std::sync::Arc;
use egui;
use glm::*;

pub type Triangle = [Vector3<f32>; 3];

use eframe::{egui_glow, glow::Context};
use egui_glow::glow;

fn rotation_x(r: f32) -> Matrix4<f32>{
    return mat4(
        1., 0., 0., 0.,
        0., cos(r), -sin(r), 0.,
        0., sin(r), cos(r), 0.,
        0., 0., 0., 1.)
}
fn rotation_y(r: f32) -> Matrix4<f32>{
    return mat4(
        cos(r), 0., sin(r), 0.,
        0., 0., 0., 0.,
        -sin(r), 0., cos(r), 0.,
        0., 0., 0., 1.)
}

pub struct MeshView {
    view_size: egui::Vec2,
    scale: f32,
    translation: Vector3::<f32>,
    rotation: Matrix4<f32>
}

impl Default for MeshView {
    fn default() -> Self {
        Self { 
            view_size: egui::vec2(0.,0.),
            scale: 1.,
            translation: Vector3 { x: 0., y: 0., z: 0. },
            rotation: mat4(
                1., 0., 0., 0.,
                0., 1., 0., 0.,
                0., 0., 1., 0.,
                0., 0., 0., 1.)
        }
    }
}

impl MeshView {
    pub fn new(size: egui::Vec2) -> Self {
        Self { 
            view_size: size,
            scale: 1.,
            translation: Vector3 { x: 0., y: 0., z: 0. },
            rotation: mat4(
                1., 0., 0., 0.,
                0., 1., 0., 0.,
                0., 0., 1., 0.,
                0., 0., 0., 1.)
        }
    }
    fn combine_transformations(&self) -> Matrix4<f32> {
        let scaling = mat4(
            self.scale, 0., 0., 0.,
            0., self.scale, 0., 0.,
            0., 0., self.scale, 0.,
            0., 0., 0., self.scale);
        let translating = mat4(
            1., 0., 0., self.translation[0],
            0., 1., 0., self.translation[1],
            0., 0., 1., self.translation[2],
            0., 0., 0., 1.);
        return translating * scaling * self.rotation;
    }
    pub fn show_mesh(&mut self, ui: &mut egui::Ui, mesh: Arc<RenderableMesh>) {
        let (rect, response) =
            ui.allocate_exact_size(self.view_size, egui::Sense::drag());

        if response.dragged() {
            self.rotation =
                rotation_y(response.drag_delta().x * 0.0625) *
                rotation_x(response.drag_delta().y * 0.0625) *
                self.rotation;
        }

        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            mesh.draw(1.0);
        });

        if ui.is_rect_visible(rect) {
            ui.painter().add(egui::PaintCallback {
                rect,
                callback: Arc::new(cb),
            });
        }
    }
}

pub struct RenderableMesh {
    shaders: glow::Program,
    triangles: Vec::<Triangle>,
    vertex_array: glow::VertexArray,
    gl: Arc<glow::Context>
}

impl RenderableMesh {
    pub fn new(gl: Arc<glow::Context>, triangles: Vec::<Triangle>) -> Self {

        use glow::HasContext as _;

        let none_fragment = 
        r#"
            #version 330 core
            precision mediump float;
            in vec4 v_color;
            out vec4 out_color;
            void main() {
                out_color = v_color;
            }
        "#;

        let phong_fragment = r#"
        #version 330 core
        vec3 light_direction = vec3(-1,-1,1);
        vec3 light_color = vec3(1,1,1);
        float diffuse = 0.5;
        float ambient = 0.1;
        float specular = 0.1;
        
        void main()
        {
          float d = dot(fNormal, normalize(light_direction));
          vec3 reflection = light_direction - fNormal * d * 2.;
          float s = max(0., dot(vec3(0.,0.,-1.), reflection));
          float intensity = ambient + diffuse * d + specular * s;
          gl_FragColor = vec4(light_color * intensity, 1.0);
        }"#;

        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            let (vertex_shader_source, fragment_shader_source) = (
                r#"
                    #version 330 core
                    const vec2 verts[3] = vec2[3](
                        vec2(0.0, 1.0),
                        vec2(-1.0, -1.0),
                        vec2(1.0, -1.0)
                    );
                    const vec4 colors[3] = vec4[3](
                        vec4(1.0, 0.0, 0.0, 1.0),
                        vec4(0.0, 1.0, 0.0, 1.0),
                        vec4(0.0, 0.0, 1.0, 1.0)
                    );
                    out vec4 v_color;
                    uniform float u_angle;
                    void main() {
                        v_color = colors[gl_VertexID];
                        gl_Position = vec4(verts[gl_VertexID], 0.0, 1.0);
                        gl_Position.x *= cos(u_angle);
                    }
                "#,
                r#"
                #version 330 core
                precision mediump float;
                in vec4 v_color;
                out vec4 out_color;
                void main() {
                    out_color = v_color;
                }
                "#,
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            let shaders: Vec<_> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(
                        shader,
                        shader_source
                    );
                    gl.compile_shader(shader);
                    if !gl.get_shader_compile_status(shader) {
                        panic!(
                            "Failed to compile custom_3d_glow: {}",
                            gl.get_shader_info_log(shader)
                        );
                    }
                    gl.attach_shader(program, shader);
                    shader
                })
                .collect();

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("{}", gl.get_program_info_log(program));
            }

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

            return Self {
                shaders: program,
                triangles,
                vertex_array,
                gl
            };
        }
    }
    pub fn draw(&self, angle: f32) {
        use glow::HasContext as _;
        unsafe {
            self.gl.use_program(Some(self.shaders));
            self.gl.uniform_1_f32(
                self.gl.get_uniform_location(self.shaders, "u_angle").as_ref(),
                angle,
            );
            self.gl.bind_vertex_array(Some(self.vertex_array));
            self.gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }
}

impl Drop for RenderableMesh {
    fn drop(&mut self) {
        use glow::HasContext as _;
        unsafe {
            self.gl.as_ref().delete_program(self.shaders);
            self.gl.as_ref().delete_vertex_array(self.vertex_array);
        }
    }
}
