use std::sync::Arc;
use egui;
use glm::*;
use bytemuck;

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
        0., 1., 0., 0.,
        -sin(r), 0., cos(r), 0.,
        0., 0., 0., 1.)
}

pub struct MeshView {
    pub view_size: egui::Vec2,
    pub scale: f32,
    pub translation: Vector3::<f32>,
    pub rotation: Matrix4<f32>,
    shader_program: glow::Program,
    gl: Arc<glow::Context>
}

impl MeshView {
    pub fn new(gl: Arc<glow::Context>, size: egui::Vec2) -> Self {
        use glow::HasContext as _;

        let phong_fragment = r#"
        #version 330 core
        uniform vec3 light_direction = vec3(-1,-1,1);
        uniform vec3 light_color = vec3(1,1,1);
        uniform float diffuse = 0.5;
        uniform float ambient = 0.1;
        uniform float specular = 0.1;
        
        void main()
        {
          float d = dot(fNormal, normalize(light_direction));
          vec3 reflection = light_direction - fNormal * d * 2.;
          float s = max(0., dot(vec3(0.,0.,-1.), reflection));
          float intensity = ambient + diffuse * d + specular * s;
          gl_FragColor = vec4(light_color * intensity, 1.0);
        }"#;

        unsafe {
            let shader_program = gl.create_program().expect("Cannot create program");

            let (vertex_shader_source, fragment_shader_source) = (
                r#"
                    #version 330 core
                    layout (location = 0) in vec3 a_pos;
                    uniform mat4 u_transformation;
                    const vec4 colors[3] = vec4[3](
                        vec4(1.0, 0.0, 0.0, 1.0),
                        vec4(0.0, 1.0, 0.0, 1.0),
                        vec4(0.0, 0.0, 1.0, 1.0)
                    );
                    out vec4 v_color;
                    uniform float u_angle;
                    void main() {
                        v_color = colors[gl_VertexID % 3];
                        gl_Position = u_transformation * vec4(a_pos.x, a_pos.y, a_pos.z * 0.01 , 1.0);
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
                    gl.attach_shader(shader_program, shader);
                    shader
                })
                .collect();

            gl.link_program(shader_program);
            if !gl.get_program_link_status(shader_program) {
                panic!("{}", gl.get_program_info_log(shader_program));
            }

            for shader in shaders {
                gl.detach_shader(shader_program, shader);
                gl.delete_shader(shader);
            }
            return Self { 
                view_size: size,
                scale: 1.,
                translation: Vector3 { x: 0., y: 0., z: 0. },
                rotation: mat4(
                    1., 0., 0., 0.,
                    0., 1., 0., 0.,
                    0., 0., 1., 0.,
                    0., 0., 0., 1.),
                shader_program,
                gl
            };
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
        use glow::HasContext as _;

        let (rect, response) =
            ui.allocate_exact_size(self.view_size, egui::Sense::drag());

        if response.dragged() {
            self.rotation =
                rotation_y(response.drag_delta().x * 0.0625) *
                rotation_x(response.drag_delta().y * 0.0625) *
                self.rotation;
            self.scale += response.drag_delta().x * 0.01;
        }

        let angle = self.scale;
        let gl = self.gl.to_owned();
        let shader_program = self.shader_program;
        let transformation_matrix = self.rotation;
        let mut transformation = [0.0 as f32; 16];
        for i in 0..transformation.len() {
            transformation[i] = transformation_matrix.as_array()[i/4][i%4];
        }
        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            unsafe {
                gl.enable(glow::DEPTH_TEST);
                gl.clear(glow::DEPTH_BUFFER_BIT);
                gl.use_program(Some(shader_program));
                gl.uniform_1_f32(
                    gl.get_uniform_location(shader_program, "u_angle").as_ref(),
                    1.0,
                );
                gl.uniform_matrix_4_f32_slice(
                    gl.get_uniform_location(shader_program, "u_transformation").as_ref(),
                    false,
                    &transformation,
                );
            }
            mesh.draw(angle);
        });

        if ui.is_rect_visible(rect) {
            ui.painter().add(egui::PaintCallback {
                rect,
                callback: Arc::new(cb),
            });
        }
    }
}


impl Drop for MeshView {
    fn drop(&mut self) {
        use glow::HasContext as _;
        unsafe {
            self.gl.as_ref().delete_program(self.shader_program);
        }
    }
}

pub struct RenderableMesh {
    triangles: Vec::<Triangle>,
    vertex_buffer: glow::Buffer,
    vertex_array: glow::VertexArray,
    gl: Arc<glow::Context>
}

impl RenderableMesh {
    pub fn new(gl: Arc<glow::Context>, triangles: Vec::<Triangle>) -> Self {
        use glow::HasContext as _;

        let mut triangle_vertices = Vec::<f32>::new();
        for &t in &triangles {
            for &v in &t {
                for f in v.as_array() {
                    triangle_vertices.push(f.to_owned());
                }
            }
        }
        
        let vertices: [f32; 9] = [
            -0.5, -0.5, 0.0, // left
            0.5, -0.5, 0.0, // right
            0.0,  0.5, 0.0  // top
        ];
        let u8_buffer: &[u8] = bytemuck::cast_slice(&triangle_vertices[..]);
        print!("{} {} {}\n", &triangle_vertices[0], &triangle_vertices[1], &triangle_vertices[2]);

        // let triangle_vertex_bytes: &[u8] = core::slice::from_raw_parts(
        //     triangle_vertices.as_ptr() as *const u8,
        //     triangle_vertices.len() * core::mem::size_of::<f32>());

        unsafe {
            let vertex_buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, u8_buffer, glow::STATIC_DRAW);

            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
            gl.bind_vertex_array(Some(vertex_array));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

            return Self {
                triangles,
                vertex_buffer,
                vertex_array,
                gl
            };
        }
    }
    pub fn draw(&self, angle: f32) {
        use glow::HasContext as _;
        unsafe {
            self.gl.bind_vertex_array(Some(self.vertex_array));
            self.gl.draw_arrays(glow::TRIANGLES, 0, self.triangles.len() as i32 * 3);
        }
    }
}

impl Drop for RenderableMesh {
    fn drop(&mut self) {
        use glow::HasContext as _;
        unsafe {
            self.gl.as_ref().delete_vertex_array(self.vertex_array);
            self.gl.as_ref().delete_buffer(self.vertex_buffer);
        }
    }
}
