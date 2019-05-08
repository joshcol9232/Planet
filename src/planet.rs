use amethyst::ecs::{System, Component, DenseVecStorage, ReadStorage, Resources, Entities, Join};
use na::{Point3, Vector3};

use std::collections::HashMap;

use crate::movement::SphericalCoords;

const TAO: f32 = 1.618033988749895;
//const TERR_HEIGHT_LIM: f32 = 10.0;  // Multiplied by radius

pub struct MapSystem;


impl<'s> System<'s> for MapSystem {
   type SystemData = (
      ReadStorage<'s, Tile>,
   );

   fn run(&mut self, tile_map: Self::SystemData) {
      println!("Running planet mapping");
      for tile in (&tile_map).join() {
         println!("biome: {}", match &tile.biome { Biome::Plains => "Plains" });
      }
   }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Tile {
   pub mesh_vertices: [usize; 3],
   pub biome: Biome,
}

impl Component for Tile {
   type Storage = DenseVecStorage<Self>;
}

impl Tile {
   fn new(vertices: [usize; 3], biome: Biome) -> Tile {
      Tile {
         mesh_vertices: vertices,
         biome: biome,
         ..Default::default()
      }
   }
}

impl Default for Tile {
   fn default() -> Tile {
      Tile {
         mesh_vertices: [0usize, 0, 0],
         biome: Biome::Plains,
      }
   }
}


#[derive(Clone, Eq, PartialEq)]
pub enum Biome {
   Plains,
}


pub struct PlanetGenerator {
   vertices: Vec<Point3<f32>>,
   faces: Vec<Point3<usize>>,
   mid_point_index_cache: HashMap<usize, usize>,
   radius: f32,
}

impl PlanetGenerator {
   fn new(rad: f32) -> PlanetGenerator {
      PlanetGenerator {
         vertices: vec![],
         faces: vec![],
         mid_point_index_cache: HashMap::new(),
         radius: rad,
      }
   }

   pub fn generate_planet_mesh(rad: f32, iter: u16) -> (Vec<Point3<f32>>, Vec<Tile>) {
      let mut pl_gen = PlanetGenerator::new(rad);
      pl_gen.generate_icosphere(iter);
      let tiles = pl_gen.assign_biomes();
      (pl_gen.vertices, tiles)
   }

   fn assign_biomes(&self) -> Vec<Tile> {
      let mut out: Vec<Tile> = vec![];
      for face in self.faces.iter() {
         out.push(Tile { mesh_vertices: [face.x, face.y, face.z], biome: Biome::Plains });
      }
      out
   }

   /*
   fn generate_terrain(&mut self, max_height: f32) -> HashMap<usize, Tile> {
      let mut tile_map: HashMap<usize, Tile> = HashMap::new();
      for (i, face) in self.faces.iter().enumerate() {
         tile_map.insert(i, Tile { biome: Biome::Plains });
      }

      tile_map
   }
   */

   fn generate_icosphere(&mut self, recursion_level: u16) {
      self.add_vertex(Point3::new(-1.0, TAO, 0.0));   // 0
      self.add_vertex(Point3::new(1.0, TAO, 0.0));    // 1
      self.add_vertex(Point3::new(-1.0, -TAO, 0.0));  // 2
      self.add_vertex(Point3::new(1.0, -TAO, 0.0));   // 3

      self.add_vertex(Point3::new(0.0, -1.0, TAO));   // 4
      self.add_vertex(Point3::new(0.0, 1.0, TAO));   // 5
      self.add_vertex(Point3::new(0.0, -1.0, -TAO));  // 6
      self.add_vertex(Point3::new(0.0, 1.0, -TAO));   // 7

      self.add_vertex(Point3::new(TAO, 0.0, -1.0));   // 8
      self.add_vertex(Point3::new(TAO, 0.0, 1.0));    // 9
      self.add_vertex(Point3::new(-TAO, 0.0, -1.0));  //10
      self.add_vertex(Point3::new(-TAO, 0.0, 1.0));  //11

      self.faces =   // Then, each face will be given a biome
         vec![Point3::new(0, 11, 5),
              Point3::new(0, 5, 1),
              Point3::new(0, 1, 7),
              Point3::new(0, 7, 10),
              Point3::new(0, 10, 11),

              Point3::new(1, 5, 9),
              Point3::new(5, 11, 4),
              Point3::new(11, 10, 2),
              Point3::new(10, 7, 6),
              Point3::new(7, 1, 8),

              Point3::new(3, 9, 4),
              Point3::new(3, 4, 2),
              Point3::new(3, 2, 6),
              Point3::new(3, 6, 8),
              Point3::new(3, 8, 9),

              Point3::new(4, 9, 5),
              Point3::new(2, 4, 11),
              Point3::new(6, 2, 10),
              Point3::new(8, 6, 7),
              Point3::new(9, 8, 1)];

      for _ in 0..recursion_level {  // Further split icosohedron into an icosphere
         let mut faces2: Vec<Point3<usize>> = vec![];
         let temp_faces = self.faces.clone();
         for tri in temp_faces.iter() {
            let a = self.get_mid_point(tri.x, tri.y);
            let b = self.get_mid_point(tri.y, tri.z);
            let c = self.get_mid_point(tri.z, tri.x);

            faces2.push(Point3::new(tri.x, a, c));
            faces2.push(Point3::new(tri.y, b, a));
            faces2.push(Point3::new(tri.z, c, b));
            faces2.push(Point3::new(a, b, c));

         }
         
         self.faces = faces2;
      }

      for i in 0..self.vertices.len() {
        self.vertices[i] *= self.radius
      }
   }

   fn get_mid_point(&mut self, p1: usize, p2: usize) -> usize { // Returns index of new point
      let (small_ind, big_ind) = if p1 < p2 { (p1, p2) } else { (p2, p1) };   // Since an edge is shared between two faces, the points may already be calculated.
      let key = (small_ind << 8) + big_ind;
      
      if self.mid_point_index_cache.contains_key(&key) {
         println!("Found in cache");
         return *self.mid_point_index_cache.get(&key).unwrap();
      }

      // Not in cache, so calculate it

      let point1 = self.vertices[p1 as usize];
      let point2 = self.vertices[p2 as usize];
      let middle = Point3::new(
         (point1.x + point2.x)/2.0,
         (point1.y + point2.y)/2.0,
         (point1.z + point2.z)/2.0);

      let i = self.add_vertex(middle);

      // Add to the cache
      self.mid_point_index_cache.insert(key, i);

      i
   }

   fn add_vertex(&mut self, p: Point3<f32>) -> usize {
      let len = ((p.x * p.x) + (p.y * p.y) + (p.z * p.z)).sqrt();
      self.vertices.push(p/len);
      self.vertices.len() - 1  // Returns index.
   }
}


