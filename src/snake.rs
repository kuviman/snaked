use super::*;

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

pub struct AiState {
    target_pos: Option<vec2<usize>>,
}

impl AiState {
    pub fn new() -> Self {
        Self { target_pos: None }
    }
}

pub fn go_ai(
    config: &Config,
    map: &mut Map,
    state: &mut AiState,
    remove_tail: bool,
) -> Result<Option<Item>, ()> {
    if let Some(pos) = find_closest_food(config, map) {
        state.target_pos = Some(pos);
    } else if state.target_pos.is_none() {
        state.target_pos = Some(vec2(
            thread_rng().gen_range(0..map.size().x),
            thread_rng().gen_range(0..map.size().y),
        ));
    }
    if let Ok(item) = go_to(map, state.target_pos.unwrap(), remove_tail) {
        return Ok(item);
    }

    state.target_pos = None;
    let tail_pos = tail(map);
    if let Ok(item) = go_to(map, tail_pos, remove_tail) {
        return Ok(item);
    }
    if let Some(next) = map
        .neighbors(head(map))
        .filter(|&pos| !matches!(map[pos], MapCell::Wall | MapCell::SnakePart(_)))
        .choose(&mut thread_rng())
    {
        return Ok(go_to(map, next, remove_tail).unwrap());
    } else {
        return Err(());
    }
}

fn find_closest_food(config: &Config, map: &Map) -> Option<vec2<usize>> {
    let head_pos = head(map);
    let mut d = vec![vec![None::<usize>; map.size().y]; map.size().x];
    let mut q = std::collections::VecDeque::new();
    d[head_pos.x][head_pos.y] = Some(0);
    q.push_back(head_pos);
    while let Some(pos) = q.pop_front() {
        let pos_d = d[pos.x][pos.y].unwrap();
        if pos_d >= config.snake_vision {
            continue;
        }
        for new_pos in map.neighbors(pos) {
            if let MapCell::Player(_) | MapCell::Item(_) = map[new_pos] {
                return Some(new_pos);
            }
            if d[new_pos.x][new_pos.y].is_none() {
                d[new_pos.x][new_pos.y] = Some(pos_d + 1);
                q.push_back(new_pos);
            }
        }
    }
    None
}

pub fn go_to(map: &mut Map, to: vec2<usize>, remove_tail: bool) -> Result<Option<Item>, ()> {
    let head_pos = head(map);
    let tail_pos = tail(map);
    if to != tail_pos && matches!(map[to], MapCell::Wall | MapCell::SnakePart(_)) {
        return Err(());
    }
    let mut d = vec![vec![None::<usize>; map.size().y]; map.size().x];
    let mut q = std::collections::VecDeque::new();
    d[to.x][to.y] = Some(0);
    q.push_back(to);
    while let Some(pos) = q.pop_front() {
        let pos_d = d[pos.x][pos.y].unwrap();
        for new_pos in map.neighbors(pos) {
            if let MapCell::Wall | MapCell::SnakePart(_) = map[new_pos] {
                continue;
            }
            if d[new_pos.x][new_pos.y].is_none() {
                d[new_pos.x][new_pos.y] = Some(pos_d + 1);
                q.push_back(new_pos);
            }
        }
    }

    if let Some(next) = map
        .neighbors(head_pos)
        .filter(|next| d[next.x][next.y].is_some())
        .min_by_key(|next| d[next.x][next.y].unwrap())
    {
        let head_idx = match map[head_pos] {
            MapCell::SnakePart(idx) => idx,
            _ => unreachable!(),
        };
        let mut eaten_item = None;
        match mem::replace(&mut map[next], MapCell::SnakePart(head_idx + 1)) {
            MapCell::Player(_) => {}
            MapCell::Item(item) => eaten_item = Some(item),
            MapCell::Empty | MapCell::SnakePart(_) => {
                if remove_tail {
                    map[tail_pos] = MapCell::Empty;
                }
            }
            MapCell::Wall => unreachable!(),
        }
        Ok(eaten_item)
    } else {
        Err(())
    }
}
