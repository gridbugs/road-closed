use std::collections::{HashSet, VecDeque};

use crate::car::CAR;
use coord_2d::{ICoord, UCoord, icoord};
use direction::DirectionsCardinal;
use grid_2d::Grid;
use perlin2::Perlin2;
use rand::{Rng, seq::SliceRandom};

const MAP_SIZE: UCoord = UCoord::new_u16(200, 100);
const ROAD_WIDTH: u32 = 12;
const ROAD_PADDING: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Ground,
    Road,
    Tree,
    DeadTree,
    FallenTree,
    FireWood,
    Grass,
    Floor,
    Wall,
    BridgeRailing,
    Door,
    Window,
    Car(char),
    Player,
    Water,
    WaterTree,
}
impl Tile {
    fn traversable(&self) -> bool {
        match self {
            Tile::Grass | Tile::Ground | Tile::Road | Tile::Floor | Tile::FireWood => true,
            _ => false,
        }
    }
}

fn make_cabin<R: Rng>(size: UCoord, rng: &mut R) -> Grid<Tile> {
    let mut grid = Grid::new_copy(size, Tile::Floor);
    for cell in grid.edge_iter_mut() {
        *cell = Tile::Wall;
    }
    let mut openings = vec![Tile::Door, Tile::Window, Tile::Window, Tile::Window];
    openings.shuffle(rng);
    grid[UCoord::new(0, size.height() / 2)] = openings[0];
    grid[UCoord::new(size.width() - 1, size.height() / 2)] = openings[1];
    grid[UCoord::new(size.width() / 2, 0)] = openings[2];
    grid[UCoord::new(size.width() / 2, size.height() - 1)] = openings[3];
    grid
}

fn make_car() -> Grid<Tile> {
    let size = UCoord::new(CAR[0].len() as u32, CAR.len() as u32);
    let mut grid = Grid::new_copy(size, Tile::Floor);
    for (i, row) in CAR.iter().enumerate() {
        for (j, ch) in row.chars().enumerate() {
            grid[ICoord::new(j as i32, i as i32)] = Tile::Car(ch);
        }
    }
    grid
}

pub struct Map1 {
    pub grid: Grid<Tile>,
    pub player_coord: ICoord,
    pub empty_space_far_from_player: Vec<ICoord>,
    pub cabin_centres: Vec<ICoord>,
}

impl Map1 {
    pub fn try_generate<R: Rng>(rng: &mut R) -> Option<Self> {
        let perlin = Perlin2::new(rng);
        let mut grid = Grid::new_fn(MAP_SIZE, |_| None);

        for (coord, cell) in grid.enumerate_mut() {
            let noise = perlin
                .noise((coord.x as f64 / 160.0, coord.y as f64 / 100.0))
                .abs();
            if noise < 0.05 {
                *cell = Some(Tile::Water);
            }
            if noise > 0.08 {
                if rng.random::<f64>() < 0.0 + noise * 0.3 {
                    let x = rng.random::<f64>();
                    *cell = Some(if x < 0.05 {
                        Tile::FireWood
                    } else if x < 0.1 {
                        Tile::DeadTree
                    } else if x < 0.2 {
                        Tile::DeadTree
                    } else {
                        Tile::Tree
                    });
                }
            }
            let noise2 = perlin
                .noise((coord.x as f64 / 32.0, coord.y as f64 / 20.0))
                .abs();
            if noise2 > 0.4 {
                if *cell == Some(Tile::Tree) {
                    *cell = Some(Tile::WaterTree);
                } else {
                    *cell = Some(Tile::Water);
                }
            }
        }

        let padded_width = ROAD_WIDTH + ROAD_PADDING * 2;
        for i in 0..MAP_SIZE.height() {
            let padded_left = (MAP_SIZE.width() - padded_width) / 2;
            let mut over_water = false;
            for j in padded_left..(padded_left + padded_width) {
                let coord = ICoord::new(j as i32, i as i32);
                let cell = grid[coord];
                match cell {
                    Some(Tile::Water) | Some(Tile::WaterTree) => over_water = true,
                    _ => (),
                }
            }
            for j in padded_left..(padded_left + padded_width) {
                let coord = ICoord::new(j as i32, i as i32);
                grid[coord] = Some(if over_water {
                    Tile::BridgeRailing
                } else if rng.random::<f32>() < 0.5 {
                    Tile::Grass
                } else {
                    Tile::Ground
                });
            }
            for j in ((MAP_SIZE.width() - ROAD_WIDTH) / 2)..((MAP_SIZE.width() + ROAD_WIDTH) / 2) {
                let coord = ICoord::new(j as i32, i as i32);
                grid[coord] = Some(Tile::Road);
            }
        }

        let car = make_car();
        for (coord, &cell) in car.enumerate() {
            grid[(MAP_SIZE.to_icoord() - car.size().to_icoord()) / 2 + coord + ICoord::new(2, 1)] =
                Some(cell);
        }

        grid[(MAP_SIZE.to_icoord() / 2) - ICoord::new(4, 0)] = Some(Tile::Player);

        let perlin_noise = |coord: ICoord| {
            let perlin_coord = (coord.x as f64 / 20.0, coord.y as f64 / 10.0);
            perlin.noise(perlin_coord).abs()
        };

        let mut cabin_centres = Vec::new();
        let mut num_cabins = 0;
        'outer: for _ in 0..100 {
            let cabin_size = UCoord::new(rng.random_range(7..=10), rng.random_range(6..=9));
            if num_cabins >= 6 {
                break;
            }
            let cabin_top_left = ICoord {
                x: rng.random_range(0..MAP_SIZE.width() as i32 - cabin_size.width() as i32),
                y: rng.random_range(0..MAP_SIZE.height() as i32 - cabin_size.height() as i32),
            };
            for relative_coord in (cabin_size + UCoord::new(2, 2)).icoord_iter_row_major() {
                let coord = cabin_top_left + relative_coord - icoord(1, 1);
                match grid.get(coord) {
                    Some(Some(Tile::Water | Tile::WaterTree | Tile::Road | Tile::Wall)) | None => {
                        continue 'outer;
                    }
                    _ => (),
                }
            }
            let cabin = make_cabin(cabin_size, rng);
            for (relative_coord, cell) in cabin.enumerate() {
                let coord = cabin_top_left + relative_coord;
                if perlin_noise(coord) > 0.05 {
                    grid[coord] = Some(*cell);
                }
            }
            cabin_centres.push(cabin_top_left + cabin_size.to_icoord() / 2);
            num_cabins += 1;
        }

        let grid = grid.map(|cell| {
            cell.unwrap_or(if rng.random::<f32>() < 0.5 {
                Tile::Grass
            } else {
                Tile::Ground
            })
        });

        let player_coord = grid
            .enumerate()
            .find_map(|(coord, cell)| {
                if let Tile::Player = cell {
                    Some(coord)
                } else {
                    None
                }
            })
            .unwrap();

        let num_reachable = {
            let mut num_reachable = 1;
            let mut to_visit = VecDeque::new();
            let mut visited = HashSet::new();
            let start = player_coord;
            to_visit.push_back(start);
            visited.insert(start);
            while let Some(coord) = to_visit.pop_back() {
                for d in DirectionsCardinal {
                    let neighbour_coord = coord + d.coord();
                    if visited.insert(neighbour_coord) {
                        if let Some(cell) = grid.get(neighbour_coord) {
                            if cell.traversable() {
                                to_visit.push_back(neighbour_coord);
                                num_reachable += 1;
                            }
                        }
                    }
                }
            }
            num_reachable
        };
        let num_traversable = grid.iter().filter(|cell| cell.traversable()).count();
        if (num_reachable as f32 / num_traversable as f32) < 0.8 {
            return None;
        }

        let empty_space_far_from_player = grid
            .enumerate()
            .filter_map(|(coord, cell)| {
                if coord.manhattan_distance(player_coord) > 15 {
                    match cell {
                        Tile::Floor | Tile::Ground | Tile::Grass | Tile::Road => Some(coord),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let player_coord = grid
            .enumerate()
            .find_map(|(coord, cell)| {
                if let Tile::Player = cell {
                    Some(coord)
                } else {
                    None
                }
            })
            .unwrap();

        Some(Self {
            grid,
            player_coord,
            empty_space_far_from_player,
            cabin_centres,
        })
    }

    pub fn generate<R: Rng>(rng: &mut R) -> Self {
        loop {
            if let Some(x) = Self::try_generate(rng) {
                return x;
            }
        }
    }

    pub fn print(&self) {
        for row in self.grid.rows() {
            for cell in row {
                match cell {
                    Tile::Ground => print!("."),
                    Tile::Road => print!(","),
                    Tile::Tree => print!("&"),
                    Tile::DeadTree => print!("$"),
                    Tile::FallenTree => print!("%"),
                    Tile::FireWood => print!("*"),
                    Tile::Grass => print!("\""),
                    Tile::Floor => print!("."),
                    Tile::Wall => print!("#"),
                    Tile::BridgeRailing => print!("#"),
                    Tile::Door => print!("+"),
                    Tile::Window => print!("="),
                    Tile::Car(ch) => print!("{}", ch),
                    Tile::Player => print!("@"),
                    Tile::Water => print!("~"),
                    Tile::WaterTree => print!("T"),
                }
            }
            println!("");
        }
    }
}
