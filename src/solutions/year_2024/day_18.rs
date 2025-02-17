use std::collections::BinaryHeap;

use crate::HashSet;

pub fn part_one(input: &str) -> u32 {
    solve_part_one::<1024, 71, 71>(input)
}

pub fn part_two(input: &str) -> String {
    solve_part_two::<1024, 71, 71>(input)
}

fn solve_part_one<const BYTES: u32, const WIDTH: usize, const HEIGHT: usize>(input: &str) -> u32 {
    let mut map = Map::<WIDTH, HEIGHT>::new(input, BYTES);
    map.part_one().unwrap_or(0)
}

fn solve_part_two<const BYTES: u32, const WIDTH: usize, const HEIGHT: usize>(
    input: &str,
) -> String {
    let mut map = Map::<WIDTH, HEIGHT>::new(input, BYTES);
    let (x, y) = map.part_two().unwrap();

    format!("{x},{y}")
}

struct Map<const WIDTH: usize, const HEIGHT: usize> {
    walls: Vec<u32>,
    cutoff: u32,
    max_cutoff: u32,
    end: (i32, i32),
}

impl<const WIDTH: usize, const HEIGHT: usize> Map<WIDTH, HEIGHT> {
    fn new(input: &str, cutoff: u32) -> Self {
        let mut walls = vec![u32::MAX; WIDTH * HEIGHT];
        let mut max_cutoff = 0;
        let end = ((WIDTH - 1) as i32, (HEIGHT - 1) as i32);

        let bytes = input
            .trim()
            .lines()
            .map(|l| l.split_once(',').unwrap())
            .map(|(l, r)| (l.parse::<usize>().unwrap(), r.parse::<usize>().unwrap()))
            .enumerate();

        for (id, (x, y)) in bytes {
            let ind = y * WIDTH + x;
            walls[ind] = id as u32;
            max_cutoff = id as u32;
        }

        Self {
            walls,
            cutoff,
            max_cutoff,
            end,
        }
    }

    fn get(&self, (x, y): (i32, i32)) -> Option<bool> {
        if x < 0 || y < 0 {
            return None;
        }
        let x = x as usize;
        let y = y as usize;
        if x >= WIDTH || y >= HEIGHT {
            return None;
        }
        let ind = y * WIDTH + x;
        self.walls.get(ind).copied().map(|n| n < self.cutoff)
    }

    fn find(&self, cutoff: u32) -> Option<(usize, usize)> {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let ind = y * WIDTH + x;

                if self.walls[ind] == cutoff {
                    return Some((x, y));
                }
            }
        }

        None
    }

    fn search(&self, heap: &mut SearchHeap, visited: &mut HashSet<(i32, i32)>) -> Option<u32> {
        visited.clear();
        heap.clear();
        heap.push((0, (0, 0)));

        while let Some((cost, point)) = heap.pop() {
            if point == self.end {
                return Some(cost);
            }

            if !visited.insert(point) {
                continue;
            }

            heap.extend(
                Directions::all()
                    .iter()
                    .map(|d| d.apply(point))
                    .filter(|&p| !self.get(p).unwrap_or(true))
                    .filter(|p| !visited.contains(p))
                    .map(|p| (cost + 1, p)),
            );
        }

        None
    }

    fn part_one(&mut self) -> Option<u32> {
        let mut visited = HashSet::new();
        let mut heap = SearchHeap::new(self.end);

        self.search(&mut heap, &mut visited)
    }

    fn part_two(&mut self) -> Option<(usize, usize)> {
        let mut visited = HashSet::new();
        let mut heap = SearchHeap::new(self.end);

        let mut min = self.cutoff;
        let mut max = self.max_cutoff;

        let mut pivot;

        loop {
            pivot = ((max - min) / 2) + min;
            self.cutoff = pivot;
            if self.search(&mut heap, &mut visited).is_some() {
                min = pivot;
            } else {
                max = pivot;
            }

            if min + 1 == max {
                return self.find(min);
            }
        }
    }
}

struct SearchHeap {
    heap: BinaryHeap<Search>,
    end: (i32, i32),
}

impl SearchHeap {
    fn new(end: (i32, i32)) -> Self {
        Self {
            heap: BinaryHeap::new(),
            end,
        }
    }

    fn clear(&mut self) {
        self.heap.clear()
    }

    fn pop(&mut self) -> Option<(u32, (i32, i32))> {
        self.heap
            .pop()
            .map(|Search { cost, point, .. }| (cost, point))
    }

    fn push(&mut self, item: (u32, (i32, i32))) {
        self.extend(Some(item))
    }

    fn extend<I: IntoIterator<Item = (u32, (i32, i32))>>(&mut self, iter: I) {
        let iter = iter.into_iter().map(|(cost, point)| Search {
            distance: distance(point, self.end),
            cost,
            point,
        });
        self.heap.extend(iter)
    }
}

fn distance((x1, y1): (i32, i32), (x2, y2): (i32, i32)) -> u32 {
    x1.abs_diff(x2) + y1.abs_diff(y2)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Search {
    distance: u32,
    cost: u32,
    point: (i32, i32),
}

impl Search {
    fn estimate(&self) -> u32 {
        self.cost + self.distance
    }
}

impl std::cmp::Ord for Search {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let ord = match self.estimate().cmp(&other.estimate()) {
            std::cmp::Ordering::Equal => self.distance.cmp(&other.distance),
            c => c,
        };

        ord.reverse()
    }
}

impl std::cmp::PartialOrd for Search {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Copy, Clone)]
enum Directions {
    Up,
    Down,
    Left,
    Right,
}

impl Directions {
    fn all() -> [Self; 4] {
        [
            Directions::Up,
            Directions::Down,
            Directions::Left,
            Directions::Right,
        ]
    }

    fn apply(&self, (x, y): (i32, i32)) -> (i32, i32) {
        match self {
            Directions::Up => (x, y - 1),
            Directions::Down => (x, y + 1),
            Directions::Left => (x - 1, y),
            Directions::Right => (x + 1, y),
        }
    }
}

#[test]
fn test() {
    let input = r#"5,4
4,2
4,5
3,0
2,1
6,3
2,4
1,5
0,6
3,3
2,6
5,1
1,2
5,5
2,5
6,5
1,4
0,4
6,4
1,1
6,1
1,0
0,5
1,6
2,0
"#;

    assert_eq!(22, solve_part_one::<12, 7, 7>(input));
    assert_eq!("6,1", solve_part_two::<12, 7, 7>(input));
}
