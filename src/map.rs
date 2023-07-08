use super::*;

#[derive(Default)]
pub enum MapCell {
    #[default]
    Empty,
    Wall,
    Player(Id),
    SnakePart(u32),
}

pub struct Map {
    cells: Vec<Vec<MapCell>>,
}

impl Map {
    pub fn iter(&self) -> impl Iterator<Item = (vec2<usize>, &MapCell)> + '_ {
        self.cells.iter().enumerate().flat_map(|(x, row)| {
            row.iter()
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
