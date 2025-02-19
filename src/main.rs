use rand::distr::{Bernoulli, Distribution};
use termion;
use std::{convert::identity, fmt, io::Write};
use itertools::Itertools;

#[derive(Clone)]
struct Grid<T> {
    width: u16,
    height: u16,
    cells: Vec<T>
}

struct Row<T> {
    width: u16,
    cells: Vec<T>
}

impl<T> Grid<T> {
    fn new(width: u16, height: u16, d: &impl Distribution<T>) -> Self {
        Self {
            width,
            height,
            cells: (0..width*height)
                .map(|_|d.sample(&mut rand::rng()))
                .collect()
        }
    }

    fn get(&self, x: u16, y: u16) -> Option<&T> {
        if x < self.width && y < self.height {
            let (x,y,w) = (x as usize, y as usize, self.width as usize);
            self.cells.get(w * y + x)
        } else {
            None
        }
    }
}
impl<T: Copy> Grid<T> {
    fn get_row(&self,y:u16) -> Option<Row<T>> {
        if y < self.height {
            Some((0..self.width)
                .map(|x| *self.get(x,y).unwrap())
                .collect())
        } else {
            None
        }
    }

    fn from_interior(interior: &Grid<T>, d: &impl Distribution<T>) -> Self {
        let (width, height) = (interior.width+2, interior.height+2);
        Self {
            width,
            height,
            cells: (0..height).flat_map(|y| (0..width).map(move |x|{
                if 0 < x && x< width-1 && 0 < y && y < height-1 {
                    *interior.get(x-1,y-1).unwrap()
                } else {
                    d.sample(&mut rand::rng())
                }
            }))
            .collect()
        }
    }
}

impl<T> FromIterator<T> for Row<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let cells: Vec<T> = iter.into_iter().collect();
        Self {
            width: cells.len() as u16,
            cells
        }
    }
}

impl fmt::Display for Row<bool> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out: String = self.cells
            .iter()
            .map(|&x| if x {'#'} else {' '})
            .collect();
        f.write_str(&out)
    }
}

impl fmt::Display for Grid<bool> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out: String = (0..self.height)
            .map(|y| (0..self.width)
                .map(|x|*self.get(x,y).unwrap())
                .collect::<Row<bool>>()
                .to_string())
            .join("\n");
        f.write_str(&out)
    }
}

fn surroundings(x:u16,y:u16) -> impl Iterator<Item=(u16,u16)> {
    (x-1..=x+1)
        .map(move |i|(i,y+1))
        .chain((x-1..=x+1)
            .map(move |i|(i,y-1))
        ).chain((0..=1)
            .map(move |i|(x-1+(2*i),y))
        )
}

fn interior_changed(grid: &Grid<bool>) -> Grid<bool>{
    let (width,height) = (grid.width-2,grid.height-2);
    Grid {
        width,
        height,
        cells: (1..=height).flat_map(|y| (1..=width).map(move |x| {
            let count = surroundings(x,y)
                .map(|(x0,y0)| *grid.get(x0,y0).unwrap())
                .filter(|&x| identity::<bool>(x))
                .collect::<Vec<_>>()
                .len();
            if count < 2 || count > 3 || (count == 2 && !grid.get(x,y).unwrap()){
                false
            } else {
                true
            }
        })).collect()
    }
}

fn update(grid: Grid<bool>,d: &impl Distribution<bool>) -> Grid<bool> {
    Grid::from_interior(&interior_changed(&grid), d)
}

fn main() {
    let d = Bernoulli::new(0.5).unwrap();
    // random boolean generator ;)
    fn rbg(d: &impl Distribution<bool>) -> bool {
        d.sample(&mut rand::rng())
    }
    let (x,y) = termion::terminal_size().unwrap();
    let mut bool_grid = Grid::new(x-1, y-1, &d);
    loop {
        let formatted = format!("\x1B[2J\x1B[0;0H{}", bool_grid);
        let stdout = std::io::stdout();
    stdout.lock().write_all(formatted.as_bytes()).unwrap();
        bool_grid = update(bool_grid,&d);
    }
}
