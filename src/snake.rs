use super::*;

const DIRECTIONS: [vec2<isize>; 4] = [vec2(-1, 0), vec2(1, 0), vec2(0, -1), vec2(0, 1)];

fn neighbors(pos: vec2<usize>, map: &Map) -> impl Iterator<Item = vec2<usize>> + '_ {
    DIRECTIONS.into_iter().map(move |dir| {
        pos.zip(dir)
            .zip(map.size())
            .map(|((pos, dir), size)| (pos as isize + size as isize + dir) as usize % size)
    })
}

pub fn head(map: &Map) -> vec2<usize> {
    let (head_pos, _) = map
        .iter()
        .filter_map(|(pos, cell)| match cell {
            MapCell::SnakePart(idx) => Some((pos, idx)),
            _ => None,
        })
        .max_by_key(|&(_, idx)| idx)
        .unwrap();
    head_pos
}

pub fn tail(map: &Map) -> vec2<usize> {
    let (head_pos, _) = map
        .iter()
        .filter_map(|(pos, cell)| match cell {
            MapCell::SnakePart(idx) => Some((pos, idx)),
            _ => None,
        })
        .min_by_key(|&(_, idx)| idx)
        .unwrap();
    head_pos
}

pub fn go_ai(map: &mut Map) -> bool {
    let did_smth = if let Some(pos) = find_closest_food(map) {
        go_to(map, pos)
    } else {
        let tail_pos = tail(map);
        go_to(map, tail_pos)
    };
    if !did_smth {
        if let Some(next) = neighbors(head(map), map)
            .filter(|&pos| matches!(map[pos], MapCell::Player(_) | MapCell::Empty))
            .choose(&mut thread_rng())
        {
            assert!(go_to(map, next));
        } else {
            return false;
        }
    }
    true
}

fn find_closest_food(map: &Map) -> Option<vec2<usize>> {
    let head_pos = head(map);
    let mut d = vec![vec![None::<usize>; map.size().y]; map.size().x];
    let mut q = std::collections::VecDeque::new();
    d[head_pos.x][head_pos.y] = Some(0);
    q.push_back(head_pos);
    while let Some(pos) = q.pop_front() {
        let pos_d = d[pos.x][pos.y].unwrap();
        for new_pos in neighbors(pos, map) {
            match map[new_pos] {
                MapCell::Wall | MapCell::SnakePart(_) => continue,
                MapCell::Player(_) => return Some(new_pos),
                _ => {}
            }
            if d[new_pos.x][new_pos.y].is_none() {
                d[new_pos.x][new_pos.y] = Some(pos_d + 1);
                q.push_back(new_pos);
            }
        }
    }
    None
}

pub fn go_to(map: &mut Map, to: vec2<usize>) -> bool {
    let (head_pos, _) = map
        .iter()
        .filter_map(|(pos, cell)| match cell {
            MapCell::SnakePart(idx) => Some((pos, idx)),
            _ => None,
        })
        .max_by_key(|&(_, idx)| idx)
        .unwrap();
    let mut d = vec![vec![None::<usize>; map.size().y]; map.size().x];
    let mut q = std::collections::VecDeque::new();
    d[to.x][to.y] = Some(0);
    q.push_back(to);
    while let Some(pos) = q.pop_front() {
        let pos_d = d[pos.x][pos.y].unwrap();
        for new_pos in neighbors(pos, map) {
            if let MapCell::Wall | MapCell::SnakePart(_) = map[new_pos] {
                continue;
            }
            if d[new_pos.x][new_pos.y].is_none() {
                d[new_pos.x][new_pos.y] = Some(pos_d + 1);
                q.push_back(new_pos);
            }
        }
    }

    if let Some(next) = neighbors(head_pos, map)
        .filter(|next| d[next.x][next.y].is_some())
        .min_by_key(|next| d[next.x][next.y].unwrap())
    {
        let head_idx = match map[head_pos] {
            MapCell::SnakePart(idx) => idx,
            _ => unreachable!(),
        };
        if !matches!(map[next], MapCell::Player(_)) {
            let tail_pos = tail(map);
            map[tail_pos] = MapCell::Empty;
        }
        map[next] = MapCell::SnakePart(head_idx + 1);
        true
    } else {
        false
    }
}
