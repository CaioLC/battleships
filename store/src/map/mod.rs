use std::ops::{Add, Sub};

use bevy::prelude::*;
use bevy::render::{mesh::Indices, render_resource::PrimitiveTopology};

#[derive(Clone, Copy, Debug, Hash)]
pub struct CubeCoords {
    pub q: i32,
    pub r: i32,
    pub s: i32,
}
impl Add for CubeCoords {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
            s: self.s + rhs.s,
        }
    }
}
impl Sub for CubeCoords {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
            s: self.s - rhs.s,
        }
    }
}

const CUBE_NEIGHBORS: [CubeCoords; 6] = [
    CubeCoords { q: 1, r: 0, s: -1 },
    CubeCoords { q: 1, r: -1, s: 0 },
    CubeCoords { q: 0, r: -1, s: 1 },
    CubeCoords { q: -1, r: 0, s: 1 },
    CubeCoords { q: -1, r: 1, s: 0 },
    CubeCoords { q: 0, r: 1, s: -1 },
];

const CUBE_DIAGONALS: [CubeCoords; 6] = [
    CubeCoords { q: 2, r: -1, s: -1 },
    CubeCoords { q: 1, r: -2, s: 1 },
    CubeCoords { q: -1, r: -1, s: 2 },
    CubeCoords { q: -2, r: 1, s: 1 },
    CubeCoords { q: -1, r: 2, s: -1 },
    CubeCoords { q: 1, r: 1, s: -2 },
];

#[derive(Clone, Debug)]
pub struct Hexagon {
    pub size: f32,
    pub padding: f32,
    pub height: f32,
    pub width: f32,
    pub coords: Option<CubeCoords>,
}

impl Hexagon {
    /// Create a new Hexagon struct
    pub fn new(size: f32, padding: f32) -> Self {
        Hexagon {
            size,
            padding,
            height: 3.0_f32.sqrt() * size + padding,
            width: 2.0 * size + padding,
            coords: None,
            // neighbors: None,
        }
    }

    /// Return the Vec2 coordinate of point i in a Hexagon
    fn hex_corner_pos(&self, i: usize) -> Vec2 {
        let angle = 60.0_f32.to_radians() * i as f32;
        return Vec2 {
            x: self.size * angle.cos(),
            y: self.size * angle.sin(),
        };
    }

    /// Generate a ['MaterialMeshBundle'] based on Hexagon coordinates and size.
    pub fn to_mesh(&self) -> Mesh {
        let mut vectors = Vec::with_capacity(8);
        vectors.push([0.0, 0.0, 0.0]);
        let mut indices = Vec::new();
        for i in 0..6 {
            let vec2d_pos = self.hex_corner_pos(i);
            trace!("{:?}", vec2d_pos);
            vectors.push([vec2d_pos.x, vec2d_pos.y, 0.0]);
            indices.push(0);
            indices.push(i as u32 + 1);
            if i < 5 {
                indices.push(i as u32 + 2);
            } else {
                indices.push(1);
            }
        }
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vectors);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 7]);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh
    }

    pub fn world_pos(&self) -> Vec3 {
        let coords = self
            .coords
            .as_ref()
            .expect("Cannot return Vec3 for a hex without a coordinate");

        // this is for axial coordinates
        let y_offset = self.height * (coords.s as f32 + 0.5 * coords.q as f32);
        let x_offset = 0.75 * self.width * coords.q as f32;

        // this is for offset coordinates only
        // let y_offset = (coordinates[0] % 2) as f32 * self.height * 0.5;
        // let x_offset = 0.75 * self.width;

        trace!("x: {:?}, y: {:?}", x_offset, y_offset);
        Vec3::new(x_offset, y_offset, 0.0)
    }

    pub fn distance(&self, hex: &Hexagon) -> Option<f32> {
        if let Some(coords) = &self.coords {
            if let Some(hex_coords) = &hex.coords {
                let dist = hex_coords.clone() - coords.clone();
                return Some((dist.q.abs() + dist.r.abs() + dist.s.abs()) as f32 / 2.0);
            }
        }
        None
    }
}

#[derive(Component, Debug)]
pub struct HexMap {
    pub total_hex_size: f32,
    pub hexes: Vec<Hexagon>,
}

impl HexMap {
    pub fn new_from_axial(radius: i32, hex_size: f32, padding: f32) -> Self {
        let mut hex = Hexagon::new(hex_size, padding);
        let mut hexes = Vec::new();
        for q in -radius..=radius {
            for s in -radius..=radius {
                let r: i32 = -s - q;
                if r.abs() > 3 {
                    continue;
                }
                hex.coords = Some(CubeCoords { q, r, s });
                println!("{:?}", &hex.coords);
                hexes.push(hex.clone());
            }
        }
        HexMap {
            total_hex_size: hex_size + padding,
            hexes,
        }
    }

    pub fn world_pos_to_coordinates(&self, pos: Vec2) -> CubeCoords {
        let basis_vec = Mat2::from_cols(
            Vec2 {
                x: 1.5,
                y: 3_f32.sqrt() / 2.0,
            },
            Vec2 {
                x: 0.,
                y: 3_f32.sqrt(),
            },
        );

        let q_r = basis_vec * pos / self.total_hex_size;
        CubeCoords {
            q: q_r.x as i32,
            r: q_r.y as i32,
            s: (-q_r.x - q_r.y) as i32,
        }
    }
    // pub fn get_hex_from_pos(pos: Vec3) -> &Hexagon {}
    // pub fn coordinate_from_pos(pos: Vec2) -> [u32; 3] {}
}

// fn hexes_from_offset(offset_type: OffsetType, size: f32) -> Vec<Hexagon> {
//     let mut hex = Hexagon::new(size);
//     let mut hexes = Vec::new();
//     match offset_type {
//         OffsetType::EvenQ(width, height) => {
//             for i in 0..height {
//                 for j in 0..width {
//                     // TODO: convert Offset to Axial Coords
//                     hex.coordinates = Some(offset_to_axial_coords(i, j));
//                     hexes.push(hex.clone());
//                 }
//             }
//         }
//     };
//     hexes
// }
// fn offset_to_axial_coords(x: i32, y: i32) -> CubeCoords {}

// fn axial_to_offset_coords(x: u32, y: u32) -> [u32; 2] {}

pub enum CoordinateSystem {
    // Square maps. u32, u32 sets width and heigh respectively
    // Offset(OffsetType),
    // Round maps. u32 the radius (number of 'rings' around center)
    Axial(i32),
}

pub enum OffsetType {
    // vertical layout. shoves odd columns up
    EvenQ(u32, u32),
}
