use crate::car::CAR;
use coord_2d::{ICoord, UCoord};
use grid_2d::Grid;
use perlin2::Perlin2;
use rand::{Rng, seq::SliceRandom};

const MAP_SIZE: UCoord = UCoord::new_u16(200, 100);
const ROAD_WIDTH: u32 = 12;
const ROAD_PADDING: u32 = 2;

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
    Door,
    Window,
    Car(char),
    Player,
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
}

impl Map1 {
    pub fn generate<R: Rng>(rng: &mut R) -> Self {
        let perlin = Perlin2::new(rng);
        let mut grid = Grid::new_fn(MAP_SIZE, |_| None);
        let padded_width = ROAD_WIDTH + ROAD_PADDING * 2;

        // road
        for i in 0..MAP_SIZE.height() {
            let padded_left = (MAP_SIZE.width() - padded_width) / 2;
            for j in padded_left..(padded_left + padded_width) {
                let coord = ICoord::new(j as i32, i as i32);
                grid[coord] = Some(if rng.random::<f32>() < 0.5 {
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

        // grid of trees
        let spacing = 3;
        let spacing_y = 2;
        let grid_tree_chance = 0.95;
        let grid_tree_dead_chance = 0.05;
        for i in 0..MAP_SIZE.height() {
            if i % (spacing_y + 1) != 0 {
                continue;
            }
            let mut j = ((MAP_SIZE.width() - padded_width) / 2 - 1) as i32;
            while j > 0 {
                if rng.random::<f32>() < grid_tree_chance {
                    let coord = ICoord::new(j, i as i32);
                    grid[coord] = Some(if rng.random::<f32>() < grid_tree_dead_chance {
                        Tile::DeadTree
                    } else {
                        Tile::Tree
                    });
                }
                j -= spacing as i32 + 1;
            }
            let mut j = ((MAP_SIZE.width() + padded_width) / 2) as i32;
            while j < MAP_SIZE.width() as i32 {
                if rng.random::<f32>() < grid_tree_chance {
                    let coord = ICoord::new(j, i as i32);
                    grid[coord] = Some(if rng.random::<f32>() < grid_tree_dead_chance {
                        Tile::DeadTree
                    } else {
                        Tile::Tree
                    });
                }
                j += spacing as i32 + 1;
            }
        }

        let wild_tile = |rng: &mut R| {
            let roll = rng.random::<f32>();
            if roll < 0.01 {
                Tile::FireWood
            } else if roll < 0.1 {
                Tile::DeadTree
            } else if roll < 0.2 {
                Tile::FallenTree
            } else {
                Tile::Tree
            }
        };

        let mut wild_candidates = grid
            .coord_iter()
            .filter(|coord| grid[*coord].is_none())
            .collect::<Vec<_>>();
        let mut road_and_padding = grid
            .coord_iter()
            .filter(|coord| {
                let mid_y = MAP_SIZE.height() as i32 / 2;
                let mid_x = MAP_SIZE.width() as i32 / 2;
                (coord.y < mid_y - 8 || coord.y > mid_y + 8)
                    && (coord.x < mid_x - 3 || coord.x > mid_x + 3)
                    && (grid[*coord] == Some(Tile::Ground)
                        || grid[*coord] == Some(Tile::Road)
                        || grid[*coord] == Some(Tile::Grass))
            })
            .collect::<Vec<_>>();
        road_and_padding.shuffle(rng);
        wild_candidates.shuffle(rng);

        let num_wild_road_and_padding_cells = 20;
        for coord in road_and_padding
            .into_iter()
            .take(num_wild_road_and_padding_cells)
        {
            grid[coord] = Some(wild_tile(rng));
        }

        let num_wild_cells = 800;
        for coord in wild_candidates.into_iter().take(num_wild_cells) {
            grid[coord] = Some(wild_tile(rng));
        }
        let mut cabin_canditates_left = vec![];
        let mut cabin_canditates_right = vec![];
        let perlin_noise = |coord: ICoord| {
            let perlin_coord = (coord.x as f64 / 50.0, coord.y as f64 / 25.0);
            perlin.noise(perlin_coord).abs()
        };
        for (coord, cell) in grid.enumerate_mut() {
            if *cell == Some(Tile::Road) {
                continue;
            }
            let noise = perlin_noise(coord);
            if noise < 0.04 {
                *cell = None;
            }
            if noise < 0.02 && coord.x > 20 && coord.x < 50 && coord.y > 20 && coord.y < 80 {
                cabin_canditates_left.push(coord);
            }
            if noise < 0.02 && coord.x > 150 && coord.x < 180 && coord.y > 20 && coord.y < 80 {
                cabin_canditates_right.push(coord);
            }
        }
        let cabin_size = UCoord::new(8, 6);
        cabin_canditates_left.shuffle(rng);
        cabin_canditates_right.shuffle(rng);

        if let Some(cabin_left_center) = cabin_canditates_left.pop() {
            for (coord, cell) in make_cabin(cabin_size, rng).enumerate() {
                let coord = cabin_left_center - cabin_size.to_icoord() / 2 + coord;
                if perlin_noise(coord) > 0.015 {
                    grid[coord] = Some(*cell);
                }
            }

            for cabin_coord in cabin_canditates_left {
                if (cabin_coord.y - cabin_left_center.y).abs() > 20 {
                    for (coord, cell) in make_cabin(cabin_size, rng).enumerate() {
                        let coord = cabin_coord - cabin_size.to_icoord() / 2 + coord;
                        if perlin_noise(coord) > 0.015 {
                            grid[coord] = Some(*cell);
                        }
                    }
                    break;
                }
            }
        }

        if let Some(cabin_right_center) = cabin_canditates_right.pop() {
            for (coord, cell) in make_cabin(cabin_size, rng).enumerate() {
                let coord = cabin_right_center - cabin_size.to_icoord() / 2 + coord;
                if perlin_noise(coord) > 0.015 {
                    grid[coord] = Some(*cell);
                }
            }

            for cabin_coord in cabin_canditates_right {
                if (cabin_coord.y - cabin_right_center.y).abs() > 20 {
                    for (coord, cell) in make_cabin(cabin_size, rng).enumerate() {
                        let coord = cabin_coord - cabin_size.to_icoord() / 2 + coord;
                        if perlin_noise(coord) > 0.015 {
                            grid[coord] = Some(*cell);
                        }
                    }
                    break;
                }
            }
        }

        let car = make_car();
        for (coord, &cell) in car.enumerate() {
            grid[(MAP_SIZE.to_icoord() - car.size().to_icoord()) / 2 + coord + ICoord::new(2, 1)] =
                Some(cell);
        }

        grid[(MAP_SIZE.to_icoord() / 2) - ICoord::new(4, 0)] = Some(Tile::Player);

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

        let empty_space_far_from_player = grid
            .enumerate()
            .filter_map(|(coord, cell)| {
                if coord.manhattan_distance(player_coord) < 15 {
                    match cell {
                        Tile::Floor | Tile::Ground | Tile::Grass | Tile::Road => Some(coord),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Self {
            grid,
            player_coord,
            empty_space_far_from_player,
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
                    Tile::Door => print!("+"),
                    Tile::Window => print!("="),
                    Tile::Car(ch) => print!("{}", ch),
                    Tile::Player => print!("@"),
                }
            }
            println!("");
        }
    }
}
