use crate::car::CAR;
use coord_2d::*;
use grid_2d::Grid;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Ground,
    Road,
    Tree,
    DeadTree,
    Grass,
    Wall,
    Door,
    Window,
    Floor,
    Car(char),
    Player,
    Typewriter,
}

const MAP_SIZE: UCoord = UCoord::new_u16(70, 60);
const CAR_POSITION: UCoord = UCoord::new_u16(35, 50);
const CABIN_POSITION: UCoord = UCoord::new_u16(35, 20);
const ROAD_WIDTH: u32 = 12;
const ROAD_PADDING: u32 = 2;

fn make_cabin(size: UCoord) -> Grid<Tile> {
    let mut grid = Grid::new_copy(size, Tile::Floor);
    for cell in grid.edge_iter_mut() {
        *cell = Tile::Wall;
    }
    grid[UCoord::new(size.width() / 2, size.height() - 1)] = Tile::Door;
    grid[UCoord::new(0, size.height() / 2)] = Tile::Window;
    grid[UCoord::new(size.width() - 1, size.height() / 2)] = Tile::Window;
    grid[size.to_icoord() / 2 - icoord(1, 1)] = Tile::Typewriter;
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
}

impl Map1 {
    pub fn generate<R: Rng>(rng: &mut R) -> Self {
        let mut grid = Grid::new_fn(MAP_SIZE, |_| None);
        let padded_width = ROAD_WIDTH + ROAD_PADDING * 2;

        for i in 0..MAP_SIZE.height() {
            grid[icoord(MAP_SIZE.width() as i32 / 2, i as i32)] =
                Some(if rng.random::<f32>() < 0.5 {
                    Tile::Grass
                } else {
                    Tile::Ground
                });
        }
        // road
        for i in (CAR_POSITION.height() - 5)..MAP_SIZE.height() {
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

        let car = make_car();
        for (coord, &cell) in car.enumerate() {
            grid[CAR_POSITION.to_icoord() - car.size().to_icoord() / 2 + coord] = Some(cell);
        }
        grid[CAR_POSITION.to_icoord() + icoord(-5, -1)] = Some(Tile::Player);

        for (coord, cell) in grid.enumerate_mut() {
            if coord.distance2(CABIN_POSITION.to_icoord()) < 50 {
                continue;
            }
            if cell.is_none() {
                if rng.random::<f32>() < 0.4 {
                    *cell = Some(if rng.random::<f32>() < 0.1 {
                        Tile::DeadTree
                    } else {
                        Tile::Tree
                    });
                }
            }
        }

        let cabin = make_cabin(ucoord(11, 6));
        for (coord, cell) in cabin.enumerate() {
            let coord = CABIN_POSITION - cabin.size().to_icoord() / 2 + coord;
            grid[coord] = Some(*cell);
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
        Self { grid, player_coord }
    }
}
