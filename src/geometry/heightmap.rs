use std::fs::File;
use std::io::{BufRead, BufReader};
extern crate nalgebra_glm as glm;
use glm::{Vec2, Vec3, TVec2};
use crate::geometry::triangle::*;
use crate::geometry::ReadError;

pub struct Heightmap {
    pub size: TVec2::<i32>,
    pub scale: Vec2,
    pub samples: Vec::<f32>,
    pub invert_y: bool
}

fn add_triangles(triangles: &mut Vec::<Triangle>, corners: [Vec3; 4]) {
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
        let x_scale = Vec2::new(scale[0], 0.);
        let y_scale = Vec2::new(0., scale[1]);
        for i in 0..self.size[0] + 1 {
            for j in 0 ..self.size[1] + 1 {
                let z = self.sample(i, j);
                let corner = Vec2::new(i as f32, j as f32).component_mul(&self.scale);
                let corners = [
                    corner.insert_row(2, z),
                    (corner + x_scale).insert_row(2, z),
                    (corner + x_scale + y_scale).insert_row(2, z),
                    (corner + y_scale).insert_row(2, z)
                ];
                if i < self.size[0] && j < self.size[1] {
                    add_triangles(&mut result, corners);
                }
                let bottom_z = self.sample(i, j - 1);
                let bottom_corners = [
                    (corner).insert_row(2, bottom_z),
                    (corner + x_scale).insert_row(2, bottom_z),
                    (corner + x_scale).insert_row(2, z),
                    (corner).insert_row(2, z)
                ];
                add_triangles(&mut result, bottom_corners);
                let left_z = self.sample(i - 1, j);
                let left_corners = [
                    (corner + y_scale).insert_row(2, left_z),
                    (corner).insert_row(2, left_z),
                    (corner).insert_row(2, z),
                    (corner + y_scale).insert_row(2, z)
                ];
                add_triangles(&mut result, left_corners);
            }
        }
        let mut extents = self.scale;
        extents[0] *= self.size[0] as f32;
        extents[1] *= self.size[1] as f32;
        let floor_corners = [
            Vec3::new(0.,0.,0.),
            Vec3::new(0.,extents[1],0.),
            Vec3::new(extents[0],extents[1],0.),
            Vec3::new(extents[0],0.,0.),
        ];
        add_triangles(&mut result, floor_corners);
        return result;
    }
}

pub fn read_heightmap(file: File)
-> Result<Heightmap, ReadError> {
    let reader = BufReader::new(file);
    let mut size = TVec2::<i32>::new(0, 0);
    let mut scale = Vec2::new(1., 1.);
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