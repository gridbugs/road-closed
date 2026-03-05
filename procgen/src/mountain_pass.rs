use crate::car::CAR;
use coord_2d::{ICoord, UCoord, icoord};
use direction::{CardinalDirection, DirectionsCardinal};
use grid_2d::Grid;
use line_2d::Direction;
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
    CliffFace,
    Door,
    Window,
    Car(char),
    Player,
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
    pub cabin_centers: Vec<ICoord>,
}

fn conway_grid<R: Rng>(size: UCoord, rng: &mut R) -> Grid<bool> {
    let mut conway_grid = Grid::new_fn(size, |_| rng.random::<bool>());
    let mut conway_grid_tmp = conway_grid.clone();
    for _ in 0..20 {
        for ((coord, &cell), next_cell) in conway_grid.enumerate().zip(conway_grid_tmp.iter_mut()) {
            let mut alive_count = 0;
            for direction in Direction::all() {
                if let Some(&nei) = conway_grid.get(coord + direction.coord()) {
                    alive_count += nei as u8;
                }
            }
            if cell {
                *next_cell = alive_count >= 4 && alive_count <= 8;
            } else {
                *next_cell = alive_count == 5;
            }
        }
        for i in 0..size.height() as i32 {
            for j in ((MAP_SIZE.width() as i32 - ROAD_WIDTH as i32 - 4) / 2)
                ..((MAP_SIZE.width() as i32 + ROAD_WIDTH as i32 + 4) / 2)
            {
                let coord = icoord(j, i);
                conway_grid_tmp[coord] = true;
            }
        }
        std::mem::swap(&mut conway_grid, &mut conway_grid_tmp);
    }
    for cell in conway_grid.iter_mut() {
        if !*cell {
            *cell = rng.random();
        }
    }
    for _ in 0..20 {
        for ((coord, &cell), next_cell) in conway_grid.enumerate().zip(conway_grid_tmp.iter_mut()) {
            let mut alive_count = 0;
            for direction in Direction::all() {
                if let Some(&nei) = conway_grid.get(coord + direction.coord()) {
                    alive_count += nei as u8;
                }
            }
            if cell {
                *next_cell = alive_count >= 4 && alive_count <= 8;
            } else {
                *next_cell = alive_count == 5;
            }
        }
        for i in 0..size.height() as i32 {
            for j in ((MAP_SIZE.width() as i32 - ROAD_WIDTH as i32) / 2)
                ..((MAP_SIZE.width() as i32 + ROAD_WIDTH as i32) / 2)
            {
                let coord = icoord(j, i);
                conway_grid_tmp[coord] = true;
            }
        }

        std::mem::swap(&mut conway_grid, &mut conway_grid_tmp);
    }
    for _ in 0..1 {
        for ((coord, &cell), next_cell) in conway_grid.enumerate().zip(conway_grid_tmp.iter_mut()) {
            let mut alive_count = 0;
            for direction in Direction::all() {
                if let Some(&nei) = conway_grid.get(coord + direction.coord()) {
                    alive_count += nei as u8;
                }
            }
            if !cell {
                *next_cell = alive_count > 4;
            }
        }
        std::mem::swap(&mut conway_grid, &mut conway_grid_tmp);
    }
    let mut seen = Grid::new_copy(size, false);
    let mut biggest = Vec::new();
    for (coord, &cell) in conway_grid.enumerate() {
        if cell && !*seen.get_checked(coord) {
            let mut stack = vec![coord];
            let mut chunk = vec![coord];
            *seen.get_checked_mut(coord) = true;
            while let Some(coord) = stack.pop() {
                for direction in CardinalDirection::all() {
                    let nei_coord = coord + direction.coord();
                    if let Some(&nei_cell) = conway_grid.get(nei_coord) {
                        if nei_cell && !*seen.get_checked(nei_coord) {
                            *seen.get_checked_mut(nei_coord) = true;
                            stack.push(nei_coord);
                            chunk.push(nei_coord);
                        }
                    }
                }
            }
            if chunk.len() > biggest.len() {
                biggest = chunk;
            }
        }
    }
    for cell in conway_grid.iter_mut() {
        *cell = false;
    }
    for coord in biggest {
        *conway_grid.get_checked_mut(coord) = true;
    }
    conway_grid
}

impl Map1 {
    pub fn generate<R: Rng>(rng: &mut R) -> Self {
        let perlin = Perlin2::new(rng);
        let conway_grid = conway_grid(MAP_SIZE, rng);
        let mut grid = conway_grid.map(|alive| {
            if alive {
                let x = rng.random::<f32>();
                if x < 0.03 {
                    Tile::DeadTree
                } else if x < 0.09 {
                    Tile::FireWood
                } else if x < 0.1 {
                    Tile::Tree
                } else if x < 0.5 {
                    Tile::Grass
                } else {
                    Tile::Ground
                }
            } else {
                Tile::CliffFace
            }
        });

        let padded_width = ROAD_WIDTH + ROAD_PADDING * 2;
        for i in 0..MAP_SIZE.height() {
            let padded_left = (MAP_SIZE.width() - padded_width) / 2;
            for j in padded_left..(padded_left + padded_width) {
                let coord = ICoord::new(j as i32, i as i32);
                let x = rng.random::<f32>();
                grid[coord] = if x < 0.01 {
                    Tile::DeadTree
                } else if x < 0.02 {
                    Tile::FireWood
                } else if x < 0.05 {
                    Tile::Tree
                } else if x < 0.5 {
                    Tile::Grass
                } else {
                    Tile::Ground
                };
            }
            for j in ((MAP_SIZE.width() - ROAD_WIDTH) / 2)..((MAP_SIZE.width() + ROAD_WIDTH) / 2) {
                let coord = ICoord::new(j as i32, i as i32);
                grid[coord] = Tile::Road;
            }
        }

        let car = make_car();
        for (coord, &cell) in car.enumerate() {
            grid[(MAP_SIZE.to_icoord() - car.size().to_icoord()) / 2 + coord + ICoord::new(2, 1)] =
                cell;
        }

        grid[(MAP_SIZE.to_icoord() / 2) - ICoord::new(4, 0)] = Tile::Player;

        let perlin_noise = |coord: ICoord| {
            let perlin_coord = (coord.x as f64 / 20.0, coord.y as f64 / 10.0);
            perlin.noise(perlin_coord).abs()
        };

        let mut cabin_centers = Vec::new();
        let mut num_cabins = 0;
        'outer: for _ in 0..100 {
            let cabin_size = UCoord::new(rng.random_range(5..=7), rng.random_range(4..=7));
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
                    Some(Tile::Road | Tile::Wall) | None => {
                        continue 'outer;
                    }
                    _ => (),
                }
            }
            let cabin = make_cabin(cabin_size, rng);
            for (relative_coord, cell) in cabin.enumerate() {
                let coord = cabin_top_left + relative_coord;
                if perlin_noise(coord) > 0.05 {
                    grid[coord] = *cell;
                }
            }
            cabin_centers.push(cabin_top_left + cabin_size.to_icoord() / 2);
            num_cabins += 1;
        }

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
            player_coord,
            grid,
            empty_space_far_from_player,
            cabin_centers,
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
                    Tile::CliffFace => print!("#"),
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
