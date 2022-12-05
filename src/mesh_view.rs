use std::sync::{Arc, Mutex};
use egui::Widget;
extern crate nalgebra_glm as glm;
use bytemuck;

pub type Triangle = [Vec3; 3];

use eframe::egui_glow;
use egui_glow::glow;
use glm::{Vec3, Mat4};

const VERTEX_SHADER_SOURCE: &str = r#"
#version 330 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec3 a_normal;
uniform mat4 u_transformation;
uniform mat3 u_normal_rotation;
const vec4 colors[3] = vec4[3](
    vec4(1.0, 0.0, 0.0, 1.0),
    vec4(0.0, 1.0, 0.0, 1.0),
    vec4(0.0, 0.0, 1.0, 1.0)
);
out vec3 v_normal;
void main() {
    v_normal = u_normal_rotation * a_normal;
    gl_Position = u_transformation * vec4(a_pos.x, a_pos.y, a_pos.z , 1.0);
    gl_Position.z *= 0.001;
}
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330 core
precision mediump float;
in vec3 v_normal;
out vec4 out_color;
vec3 light_direction = vec3(-1,-1,-1);
vec3 light_color = vec3(1,1,1);
float diffuse = 0.4;
float ambient = 0.2;
float specular = 0.4;

void main()
{
  vec3 normal_3 = normalize(vec3(v_normal.x, v_normal.y, v_normal.z));
  float d = dot(normal_3, normalize(light_direction));
  vec3 reflection = light_direction - normal_3 * d * 2.;
  float s = max(0., dot(vec3(0.,0.,-1.), reflection));
  float intensity = ambient + diffuse * max(0, d) + specular * s;
  out_color = vec4(light_color * intensity, 1.0);
}
"#;

fn create_shader_program(gl: &Arc<glow::Context>) -> Result<glow::Program, String>{
    use glow::HasContext as _;

    unsafe {
        let shader_program = gl.create_program()?;

        let shader_sources = [
            (glow::VERTEX_SHADER, VERTEX_SHADER_SOURCE),
            (glow::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE),
        ];

        let mut shaders: Vec<glow::NativeShader> = Vec::new();
        for (shader_type, shader_source) in &shader_sources {
            let shader = gl.create_shader(*shader_type)?;
            gl.shader_source(shader, shader_source);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                return Err(format!(
                    "Failed to compile shader: {}",
                    gl.get_shader_info_log(shader)));
            }
            gl.attach_shader(shader_program, shader);
            shaders.push(shader);
        }

        gl.link_program(shader_program);
        if !gl.get_program_link_status(shader_program) {
            return Err(format!("{}", gl.get_program_info_log(shader_program)));
        }

        for shader in shaders {
            gl.detach_shader(shader_program, shader);
            gl.delete_shader(shader);
        }
        return Ok(shader_program);
    }
}

/// A simple Widget to view triangles in 3D space
pub struct MeshView {
    pub view_size: egui::Vec2,
    pub mesh: Arc<Mutex<RenderableMesh>>
}

impl MeshView {
    pub fn new(size: egui::Vec2, mesh: Arc<Mutex<RenderableMesh>>) -> Result<Self, String> {
        return Ok(Self {
            view_size: size,
            mesh
        });
    }
}

impl Widget for MeshView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(self.view_size, egui::Sense::drag());

        {
            let mut mesh = self.mesh.lock().unwrap();

            if response.dragged_by(egui::PointerButton::Primary) {
                mesh.rotate_y(-response.drag_delta().x * 0.01);
                mesh.rotate_x(-response.drag_delta().y * 0.01);
            }
            if response.dragged_by(egui::PointerButton::Secondary) {
                let matrix = mesh.combine_transformations();
                if let Some(inverse_matrix) = matrix.try_inverse() {
                    let delta4 = inverse_matrix * glm::Vec4::new(
                        2. * response.drag_delta().x / self.view_size.x,
                        -2. * response.drag_delta().y / self.view_size.y,
                        0., 0.);
                    mesh.translation += Vec3::new(delta4.x, delta4.y, delta4.z);
                }
            }
            if response.dragged_by(egui::PointerButton::Middle) {
                mesh.scale *= std::f32::consts::E.powf(-response.drag_delta().y * 0.01);
            }
        }

        let cb = egui_glow::CallbackFn::new(move |_info, _painter| {
            self.mesh.lock().unwrap().draw();
        });

        if ui.is_rect_visible(rect) {
            ui.painter().add(egui::PaintCallback {
                rect,
                callback: Arc::new(cb),
            });
        }
        return response;
    }
}

pub struct RenderableMesh {
    /// Position of the mesh (relative to its original coordinate system)
    pub translation: Vec3,
    /// Size of the mesh during render
    pub scale: f32,
    /// Rotation matrix for the mesh.
    pub rotation: Mat4,
    pub right_handed: bool,
    vertex_buffer: glow::Buffer,
    vertex_array: glow::VertexArray,
    triangle_count: usize,
    shader_program: glow::Program,
    gl: Arc<glow::Context>
}

/// A triangle mesh that can be rendered.
///
/// This structure contains all the data required to render a triangle mesh
/// to a glow::Context. It uses a simple phong shader with directional lighting,
/// and provides some basic fields for transformations.
impl RenderableMesh {

    /// Creates a RenderableMesh from a list of Triangles
    ///
    /// This function creates buffers and shaders for the gl context,
    /// which are cleaned up when the RenderableMesh is dropped.
    pub fn new(gl: Arc<glow::Context>, triangles: Vec::<Triangle>) -> Result<Self, String> {
        use glow::HasContext as _;

        let mut triangle_vertices = Vec::<f32>::new();
        for &t in &triangles {
            // Only add triangles with non-zero area
            let cross_product = glm::cross(&(t[1] - t[0]), &(t[2] - t[0]));
            if glm::dot(&cross_product, &cross_product) > 0.0 {
                let normal = cross_product.normalize();
                for &v in &t {
                    for f in &v {
                        triangle_vertices.push(f.to_owned());
                    }
                    for f in &normal {
                        triangle_vertices.push(f.to_owned());
                    }
                }
            }
        }

        let u8_buffer: &[u8] = bytemuck::cast_slice(&triangle_vertices[..]);

        unsafe {
            let vertex_buffer = gl.create_buffer()?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, u8_buffer, glow::STATIC_DRAW);

            let vertex_array = match gl.create_vertex_array() {
                Ok(val) => { val },
                Err(val) => {
                    // Delete the vertex buffer before erroring
                    gl.as_ref().delete_buffer(vertex_buffer);
                    return Err(val);
                }
            };
            gl.bind_vertex_array(Some(vertex_array));
            gl.enable_vertex_attrib_array(0);
            let bpv = 12; // Bytes Per Vector3
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, bpv * 2, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, bpv * 2, bpv);

            return Ok(Self {
                scale: 1.,
                translation: Vec3::new(0., 0., 0.),
                rotation: Mat4::identity(),
                right_handed: true,
                vertex_buffer,
                vertex_array,
                shader_program: create_shader_program(&gl)?,
                triangle_count: triangles.len(),
                gl
            });
        }
    }

    /// Combines the transformations (translation, scale, rotatioin)
    /// into a single transformation matrix.
    pub fn combine_transformations(&self) -> Mat4 {
        // The negative z coordinate makes the coordinates right handed in the shader
        // There's probably a better way to do this
        let scaling = Mat4::new(
            self.scale, 0., 0., 0.,
            0., self.scale, 0., 0.,
            0., 0., -self.scale, 0.,
            0., 0., 0., 1.0);
        let translating = Mat4::new(
            1., 0., 0., self.translation[0],
            0., 1., 0., self.translation[1],
            0., 0., 1., self.translation[2],
            0., 0., 0., 1.);
        return self.rotation * scaling * translating;
    }

    /// Renders the mesh to its glow::Context using its combined transformations
    /// As side effects, this enables the depth test, clears and uses the depth buffer,
    /// and sets the shader program to that of the Renderable Mesh
    pub fn draw(&self) {
        use glow::HasContext as _;
        let transformation_matrix = self.combine_transformations();
        let transformation = transformation_matrix.as_slice().to_owned();
        let normal_rotation = match transformation_matrix.try_inverse() {
            Some(result) => {result.transpose().as_slice().to_owned()},
            None => {Mat4::identity().as_slice().to_owned()}
        };
        unsafe {
            self.gl.enable(glow::DEPTH_TEST);
            self.gl.clear(glow::DEPTH_BUFFER_BIT);
            self.gl.use_program(Some(self.shader_program));
            self.gl.uniform_matrix_4_f32_slice(
                self.gl.get_uniform_location(self.shader_program, "u_transformation").as_ref(),
                false,
                &transformation,
            );
            self.gl.uniform_matrix_3_f32_slice(
                self.gl.get_uniform_location(self.shader_program, "u_normal_rotation").as_ref(),
                false,
                &normal_rotation,
            );
            self.gl.bind_vertex_array(Some(self.vertex_array));
            self.gl.draw_arrays(glow::TRIANGLES, 0, self.triangle_count as i32 * 3);
        }
    }

    /// Reference to the glow::Context used to create this mesh's buffers and shaders
    pub fn get_gl(&self) -> Arc<glow::Context> {
        return self.gl.to_owned();
    }
    /// The number of triangles in the vertex buffer
    pub fn get_triangle_count(&self) -> usize{
        return self.triangle_count;
    }
    /// Sets the rotation matrix back to the identity matrix
    pub fn reset_rotation(&mut self) {
        self.rotation = Mat4::identity();
    }
    /// Rotate around the x axis (relative to the model's current rotation)
    pub fn rotate_x(&mut self, radians: f32) {
        self.rotation = glm::rotate_x(&self.rotation, radians);
    }
    /// Rotate around the y axis (relative to the model's current rotation)
    pub fn rotate_y(&mut self, radians: f32) {
        self.rotation = glm::rotate_y(&self.rotation, radians);
    }
    /// Rotate around the z axis (relative to the model's current rotation)
    pub fn rotate_z(&mut self, radians: f32) {
        self.rotation = glm::rotate_z(&self.rotation, radians);
    }
}

impl Drop for RenderableMesh {
    fn drop(&mut self) {
        use glow::HasContext as _;
        unsafe {
            self.gl.as_ref().delete_vertex_array(self.vertex_array);
            self.gl.as_ref().delete_buffer(self.vertex_buffer);
            self.gl.as_ref().delete_program(self.shader_program);
        }
    }
}
