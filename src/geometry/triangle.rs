use std::fs::File;
use std::io::Write;
use glm::Vector3;

pub type Triangle = [Vector3::<f32>; 3];

fn write_vec3(file: &mut File, vector: &Vector3::<f32>)
-> Result<(), std::io::Error>{
    file.write_all(&vector[0].to_le_bytes())?;
    file.write_all(&vector[1].to_le_bytes())?;
    file.write_all(&vector[2].to_le_bytes())?;
    return Ok(());
}

pub fn write_stl_binary(
    path: String,
    triangles: &Vec::<Triangle>)
-> Result<(), std::io::Error> {
    let mut output = File::create(path)?;
    output.write_all(&[0 as u8; 80])?;
    output.write_all(&(triangles.len() as u32).to_le_bytes())?;
    for triangle in triangles {
        let edge1 = triangle[1] - triangle[0];
        let edge2 = triangle[2] - triangle[0];
        let normal = glm::normalize(glm::cross(edge1, edge2));
        write_vec3(&mut output, &normal)?;
        for vertex in triangle {
            write_vec3(&mut output, vertex)?;
        }
        output.write(&[0 as u8; 2])?;
    }
    return Ok(());
}