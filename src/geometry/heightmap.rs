use std::fs::File;
use std::io::{BufRead, BufReader};
extern crate nalgebra_glm as glm;
use glm::{Vec2, Vec3, TVec2};
use image::GenericImageView;
use crate::geometry::triangle::*;
use crate::geometry::ReadError;
use image::io::Reader as ImageReader;
use rand::random;

pub struct Heightmap {
    pub size: TVec2::<i32>,
    pub scale: Vec2,
    pub samples: Vec::<f32>,
    pub invert_y: bool
}

// Utility function to add a square face to a Vec of Triangles.
fn add_rect(triangles: &mut Vec::<Triangle>, corners: [Vec3; 4]) {
    triangles.push([corners[0], corners[1], corners[2]]);
    triangles.push([corners[0], corners[2], corners[3]]);
}

impl Heightmap {
    pub fn sample(&self, i: i32, j: i32) -> f32{
        if i < 0 || j < 0 || i >= self.size[0] || j >= self.size[1] {
            0.
        } else if self.invert_y {
            let index = ((self.size[1] - j - 1) * self.size[0] + i) as usize;
            self.samples[index]
        } else {
            let index = (j * self.size[0] + i) as usize;
            self.samples[index]
        }
    }

    pub fn get_triangles(&self)
    -> Vec::<Triangle> {
        let mut result = Vec::<Triangle>::new();
        let scale = self.scale;
        let x_scale = Vec2::new(scale[0], 0.);
        let y_scale = Vec2::new(0., scale[1]);
        for j in 0 ..self.size[1] + 1 {
            // when does I next change
            // (used to combine all surfaces in a row into a single rect)
            let mut next_i = 0;
            for i in 0..self.size[0] + 1 {
                let z = self.sample(i, j);
                let corner = Vec2::new(i as f32, j as f32).component_mul(&self.scale);
                if i < self.size[0] && j < self.size[1] && z > 0. && i >= next_i {
                    for ni in (i + 1)..=self.size[0] {
                        next_i = ni;
                        if self.sample(ni, j) != z {
                            break;
                        }
                    }
                    let i_count = (next_i - i) as f32;
                    let xs = x_scale * i_count;
                    add_rect(&mut result,
                        [
                            corner.insert_row(2, z),
                            (corner + xs).insert_row(2, z),
                            (corner + xs + y_scale).insert_row(2, z),
                            (corner + y_scale).insert_row(2, z)
                        ]);
                    add_rect(&mut result,
                        [
                            (corner + xs).insert_row(2, 0.),
                            (corner).insert_row(2, 0.),
                            (corner + y_scale).insert_row(2, 0.),
                            (corner + xs + y_scale).insert_row(2, 0.)
                        ]);
                }
                let bottom_z = self.sample(i, j - 1);
                let bottom_corners = [
                    (corner).insert_row(2, bottom_z),
                    (corner + x_scale).insert_row(2, bottom_z),
                    (corner + x_scale).insert_row(2, z),
                    (corner).insert_row(2, z)
                ];
                add_rect(&mut result, bottom_corners);
                let left_z = self.sample(i - 1, j);
                let left_corners = [
                    (corner + y_scale).insert_row(2, left_z),
                    (corner).insert_row(2, left_z),
                    (corner).insert_row(2, z),
                    (corner + y_scale).insert_row(2, z)
                ];
                add_rect(&mut result, left_corners);
            }
        }
        // let mut extents = self.scale;
        // extents[0] *= self.size[0] as f32;
        // extents[1] *= self.size[1] as f32;
        // let floor_corners = [
        //     Vec3::new(0.,0.,0.),
        //     Vec3::new(0.,extents[1],0.),
        //     Vec3::new(extents[0],extents[1],0.),
        //     Vec3::new(extents[0],0.,0.),
        // ];
        // add_rect(&mut result, floor_corners);
        result
    }
}

pub fn read_heightmap_image(filename: &str)
-> Result<Heightmap, ReadError> {
    let image = ImageReader::open(filename)?.decode()?;
    let size = TVec2::<i32>::new(image.width() as i32, image.height() as i32);
    let scale = Vec2::new(1., 1.);
    let mut samples = Vec::<f32>::new();
    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y);
            let max_channel = pixel[0].max(pixel[1]).max(pixel[2]) as f32 / 255.;
            let mut randomness = 0.;
            if pixel[0] == 0 && max_channel > 0. {
                randomness = 1./255.;
            }
            let sample = max_channel + randomness * random::<f32>();
            samples.push(sample * size.max() as f32 * 1./32.);
        }
    }
    Ok(Heightmap{size, scale, samples, invert_y: true})
}

pub fn read_heightmap(file: File)
-> Result<Heightmap, ReadError> {
    let reader = BufReader::new(file);
    let mut size = TVec2::<i32>::new(0, 0);
    let mut scale = Vec2::new(1., 1.);
    let mut samples = Vec::<f32>::new();
    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        match line_num {
            0 => {
                let parts: Vec<&str> = line.split(',').collect();
                size[0] = parts[0].trim().parse()?;
                size[1] = parts[1].trim().parse()?;
            },
            1 => {
                let parts: Vec<&str> = line.split(',').collect();
                scale[0] = parts[0].trim().parse()?;
                scale[1] = parts[1].trim().parse()?;
            },
            _ => {
                samples.push(line.trim().parse()?);
            }
        }
    }
    Ok(Heightmap{size, scale, samples, invert_y: false})
}