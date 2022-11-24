use std::fs::File;
use std::io::{BufRead, BufReader};
use glm::{Vector2, Vector3};
use crate::geometry::triangle::*;
use crate::geometry::ReadError;

pub struct Heightmap {
    pub size: Vector2::<i32>,
    pub scale: Vector2::<f32>,
    pub samples: Vec::<f32>,
    pub invert_y: bool
}

fn add_triangles(triangles: &mut Vec::<Triangle>, corners: [Vector3::<f32>; 4]) {
    triangles.push([corners[0], corners[1], corners[2]]);
    triangles.push([corners[0], corners[2], corners[3]]);
}

impl Heightmap {
    pub fn sample(&self, i: i32, j: i32) -> f32{
        if i < 0 || j < 0 || i >= self.size[0] || j >= self.size[1] {
            return 0.;
        } else {
            if self.invert_y {
                let index = ((self.size[1] as i32 - j - 1) * self.size[1] + i) as usize;
                return self.samples[index];
            } else {
                let index = (j * self.size[1] + i) as usize;
                return self.samples[index];
            }
        }
    }

    pub fn get_triangles(&self)
    -> Vec::<Triangle> {
        let mut result = Vec::<Triangle>::new();
        let scale = self.scale;
        let x_scale = Vector2::<f32>{x: scale[0], y: 0.};
        let y_scale = Vector2::<f32>{x: 0., y: scale[1]};
        for i in 0..self.size[0] + 1 {
            for j in 0 ..self.size[1] + 1 {
                let z = self.sample(i, j);
                let corner = Vector2::<f32>{x: i as f32, y: j as f32} * self.scale;
                let corners = [
                    (corner).extend(z),
                    (corner + x_scale).extend(z),
                    (corner + x_scale + y_scale).extend(z),
                    (corner + y_scale).extend(z)
                ];
                if i < self.size[0] && j < self.size[1] {
                    add_triangles(&mut result, corners);
                }
                let bottom_z = self.sample(i, j - 1);
                let bottom_corners = [
                    (corner).extend(bottom_z),
                    (corner + x_scale).extend(bottom_z),
                    (corner + x_scale).extend(z),
                    (corner).extend(z)
                ];
                add_triangles(&mut result, bottom_corners);
                let left_z = self.sample(i - 1, j);
                let left_corners = [
                    (corner + y_scale).extend(left_z),
                    (corner).extend(left_z),
                    (corner).extend(z),
                    (corner + y_scale).extend(z)
                ];
                add_triangles(&mut result, left_corners);
            }
        }
        let mut extents = self.scale;
        extents[0] *= self.size[0] as f32;
        extents[1] *= self.size[1] as f32;
        let floor_corners = [
            Vector3::<f32>::new(0.,0.,0.),
            Vector3::<f32>::new(0.,extents[1],0.),
            Vector3::<f32>::new(extents[0],extents[1],0.),
            Vector3::<f32>::new(extents[0],0.,0.),
        ];
        add_triangles(&mut result, floor_corners);
        return result;
    }
}

pub fn read_heightmap(file: File)
-> Result<Heightmap, ReadError> {
    let reader = BufReader::new(file);
    let mut size = Vector2::<i32>{x: 0, y: 0};
    let mut scale = Vector2::<f32>{x: 1., y: 1.};
    let mut samples = Vec::<f32>::new();
    let mut line_num = 0;
    for line_result in reader.lines() {
        let line = line_result?;
        match line_num {
            0 => {
                let parts: Vec<&str> = line.split(",").collect();
                size[0] = parts[0].trim().parse()?;
                size[1] = parts[1].trim().parse()?;
            },
            1 => {
                let parts: Vec<&str> = line.split(",").collect();
                scale[0] = parts[0].trim().parse()?;
                scale[1] = parts[1].trim().parse()?;
            },
            _ => {
                samples.push(line.trim().parse()?);
            }
        }
        line_num += 1;
    }
    return Ok(Heightmap{size: size, scale: scale, samples: samples, invert_y: false});
}