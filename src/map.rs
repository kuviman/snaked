use super::*;

const DIRECTIONS: [vec2<isize>; 4] = [vec2(-1, 0), vec2(1, 0), vec2(0, -1), vec2(0, 1)];

#[derive(Clone)]
pub enum Item {
    Food,
    Reverse,
    SnakeSpeedUp,
    SnakeSpeedDown,
}

#[derive(Default, Clone)]
pub enum MapCell {
    #[default]
    Empty,
    Wall,
    Player(Id),
    Item(Item),
    /// Head = max .0, tail = min .0
    SnakePart(u32),
}

pub struct Map {
    cells: Vec<Vec<MapCell>>,
}

impl Map {
    pub fn diff(&self, a: vec2<usize>, b: vec2<usize>) -> vec2<isize> {
        a.zip(b).zip(self.size()).map(|((a, b), size)| {
            let a = a as isize;
            let b = b as isize;
            let size = size as isize;
            (-1..=1)
                .map(|x| a - b + size * x)
                .min_by_key(|value| value.abs())
                .unwrap()
        })
    }
    pub fn distance(&self, a: vec2<usize>, b: vec2<usize>) -> usize {
        let diff = self.diff(a, b);
        (diff.x.abs() + diff.y.abs()) as usize
    }
    pub fn neighbors(&self, pos: vec2<usize>) -> impl Iterator<Item = vec2<usize>> + '_ {
        DIRECTIONS
            .into_iter()
            .map(move |dir| self.add_dir(pos, dir))
    }

    pub fn add_dir(&self, pos: vec2<usize>, dir: vec2<isize>) -> vec2<usize> {
        pos.zip(dir)
            .zip(self.size())
            .map(|((pos, dir), size)| (pos as isize + size as isize + dir) as usize % size)
    }

    pub fn iter(&self) -> impl Iterator<Item = (vec2<usize>, &MapCell)> + '_ {
        self.cells.iter().enumerate().flat_map(|(x, row)| {
            row.iter()
                .enumerate()
                .map(move |(y, cell)| (vec2(x, y), cell))
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (vec2<usize>, &mut MapCell)> + '_ {
        self.cells.iter_mut().enumerate().flat_map(|(x, row)| {
            row.iter_mut()
                .enumerate()
                .map(move |(y, cell)| (vec2(x, y), cell))
        })
    }
    pub fn size(&self) -> vec2<usize> {
        vec2(self.cells.len(), self.cells[0].len())
    }
    pub fn parse(s: &str) -> Self {
        Self {
            cells: {
                let mut cells: Vec<Vec<MapCell>> = vec![];
                for (y, line) in s.lines().enumerate() {
                    for (x, c) in line.chars().enumerate() {
                        let cell = match c {
                            ' ' => MapCell::Empty,
                            '#' => MapCell::Wall,
                            _ => {
                                if let Some(x) = c.to_digit(10) {
                                    MapCell::SnakePart(x)
                                } else {
                                    panic!("Unexpected character {c:?}");
                                }
                            }
                        };
                        cells.resize_with(cells.len().max(x + 1), default);
                        let row = &mut cells[x];
                        row.resize_with(row.len().max(y + 1), default);
                        row[y] = cell;
                    }
                }
                let height = cells.iter().map(|row| row.len()).max().unwrap();
                for row in &mut cells {
                    row.resize_with(height, default);
                    row.reverse();
                }
                cells
            },
        }
    }

    pub fn save(&self, path: impl AsRef<std::path::Path>) {
        let f = std::fs::File::create(path).unwrap();
        let mut writer = std::io::BufWriter::new(f);
        for y in (0..self.size().y).rev() {
            for x in 0..self.size().x {
                let c = match self.cells[x][y] {
                    MapCell::Wall => '#',
                    _ => ' ',
                };
                write!(writer, "{c}").unwrap();
            }
            writeln!(writer).unwrap();
        }
    }
}

impl Index<vec2<usize>> for Map {
    type Output = MapCell;
    fn index(&self, pos: vec2<usize>) -> &MapCell {
        &self.cells[pos.x][pos.y]
    }
}

impl IndexMut<vec2<usize>> for Map {
    fn index_mut(&mut self, pos: vec2<usize>) -> &mut MapCell {
        &mut self.cells[pos.x][pos.y]
    }
}
