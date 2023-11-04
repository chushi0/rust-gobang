use crate::game::*;
use lazy_static::lazy_static;
use rand::Rng;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU32, Mutex},
    vec,
};

const MAX_DEPTH: i32 = 3;

fn minimax(state: &GameState, depth: i32, alpha: i32, beta: i32, maximizing_player: bool) -> i32 {
    if depth == 0 {
        return evaluate(&state);
    }

    let mut alpha = alpha;
    let mut beta = beta;
    if maximizing_player {
        let mut max_eval = i32::MIN;
        for action in generate_moves(&state) {
            let mut new_state = state.clone();
            let result = new_state.make_move(action).expect("move to possible moves");
            let eval = match result {
                GameEnd::NotEnd => minimax(&new_state, depth - 1, alpha, beta, false),
                GameEnd::Win => i32::MAX,
                GameEnd::Lost => i32::MIN,
            };
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for action in generate_moves(&state) {
            let mut new_state = state.clone();
            let result = new_state.make_move(action).expect("move to possible moves");
            let eval = match result {
                GameEnd::NotEnd => minimax(&new_state, depth - 1, alpha, beta, true),
                GameEnd::Win => i32::MIN,
                GameEnd::Lost => i32::MAX,
            };
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        min_eval
    }
}

pub fn best_move(state: &GameState) -> (Option<Point>, i32) {
    COUNTER.store(0, std::sync::atomic::Ordering::Relaxed);
    EVALUATE_CACHE.lock().expect("should obtain lock").clear();

    let mut best_eval = i32::MIN;
    let mut best_moves = Vec::new();

    let actions: Vec<((usize, usize), i32)> = generate_moves(state)
        .into_par_iter()
        .map(|action| {
            let mut state_snapshot = state.clone();
            state_snapshot
                .make_move(action)
                .expect("move to possible moves");
            let eval = minimax(&state_snapshot, MAX_DEPTH, i32::MIN, i32::MAX, false);

            (action, eval)
        })
        .collect();

    for (action, eval) in actions {
        if eval > best_eval {
            best_eval = eval;
            best_moves = vec![action];
        } else if eval == best_eval {
            best_moves.push(action);
        }
    }

    let best_move = match best_moves.len() {
        0 => None,
        1 => Some(best_moves[0]),
        len => Some(best_moves[rand::thread_rng().gen_range(0..len)]),
    };

    let counter = COUNTER.load(std::sync::atomic::Ordering::Relaxed);
    println!("evaluate count: {counter}");

    (best_move, best_eval)
}

fn generate_moves(state: &GameState) -> Vec<Point> {
    let mut positions = Vec::new();

    for x in 0..15 {
        for y in 0..15 {
            if state.piece((x, y)) != Piece::Empty {
                continue;
            }

            let mut nearby = false;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let px = x as isize + dx;
                    let py = y as isize + dy;
                    if px < 0 || py < 0 || px >= 15 || py >= 15 {
                        continue;
                    }

                    if state.piece((px as usize, py as usize)) != Piece::Empty {
                        nearby = true;
                        break;
                    }
                }
            }

            if nearby {
                positions.push(((x, y), evaluate_location(state, (x, y))));
            }
        }
    }

    positions.sort_by(|pos1, pos2| pos1.1.cmp(&pos2.1));
    positions.into_iter().map(|(point, _)| point).collect()
}

lazy_static! {
    static ref EVALUATE_CACHE: Mutex<HashMap<GameState, i32>> = Mutex::new(HashMap::new());
}

static COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn evaluate(state: &GameState) -> i32 {
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if let Some(score) = EVALUATE_CACHE
        .lock()
        .expect("should obtain lock")
        .get(state)
        .map(|score| *score)
    {
        return score;
    }

    let mut score = 0;

    for (piece_state, count) in sequence(state, state.current_turn()) {
        score += match piece_state {
            PieceState::Long => 1000000,
            PieceState::Five => 1000000,
            PieceState::RushFour => 2000,
            PieceState::LiveFour => 10000,
            PieceState::LiveThree => 2000,
            PieceState::SleepThree => 100,
            PieceState::LiveTwo => 5,
            PieceState::SleepTwo => 1,
        } * count;
    }

    for (piece_state, count) in sequence(
        state,
        match state.current_turn() {
            Piece::Empty => unreachable!(),
            Piece::White => Piece::Black,
            Piece::Black => Piece::White,
        },
    ) {
        score -= match piece_state {
            PieceState::Long => 1000000,
            PieceState::Five => 1000000,
            PieceState::RushFour => 5000,
            PieceState::LiveFour => 10000,
            PieceState::LiveThree => 200,
            PieceState::SleepThree => 100,
            PieceState::LiveTwo => 5,
            PieceState::SleepTwo => 1,
        } * count;
    }

    EVALUATE_CACHE
        .lock()
        .expect("should obtain lock")
        .insert(state.clone(), score);

    score
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum PieceState {
    Long,
    Five,
    RushFour,
    LiveFour,
    LiveThree,
    SleepThree,
    LiveTwo,
    SleepTwo,
}

fn sequence(game_state: &GameState, my_piece: Piece) -> Vec<(PieceState, i32)> {
    vec![(1, 0), (0, 1), (1, 1), (1, -1)]
        .into_iter()
        .flat_map(|dir| {
            let mut vec = Vec::new();
            vec.push(((0, 0), dir));
            if dir.1 > 0 {
                for x in 1..15 {
                    vec.push(((x, 0), dir));
                }
            } else if dir.1 < 0 {
                for x in 1..15 {
                    vec.push(((x, 14), dir));
                }
            }
            if dir.0 != 0 {
                for y in 1..15 {
                    vec.push(((0, y), dir));
                }
            }
            vec
        })
        .map(|(point, dir)| {
            let mut point = point;
            let mut sequence = Vec::new();
            sequence.push(2);
            while next_point_valid(&mut point, &dir) {
                let piece = game_state.piece(point);
                if piece == Piece::Empty {
                    sequence.push(0);
                } else if piece == my_piece {
                    sequence.push(1);
                } else {
                    sequence.push(2);
                }
            }
            sequence.push(2);
            sequence
        })
        .filter(|sequence| sequence.len() > 7)
        .filter(|sequence| {
            for i in sequence {
                if *i != 0 {
                    return true;
                }
            }
            false
        })
        .flat_map(|sequence| {
            let patterns: Vec<(Vec<i32>, PieceState)> = with_inv(vec![
                (vec![1, 1, 1, 1, 1, 1], PieceState::Long),
                (vec![1, 1, 1, 1, 1], PieceState::Five),
                (vec![0, 1, 1, 1, 1, 0], PieceState::LiveFour),
                (vec![0, 1, 1, 1, 0, 0], PieceState::LiveThree),
                (vec![0, 1, 0, 1, 1, 0], PieceState::LiveThree),
                (vec![0, 0, 1, 1, 0, 0], PieceState::LiveTwo),
                (vec![2, 1, 1, 1, 1, 0], PieceState::RushFour),
                (vec![1, 1, 0, 1, 1], PieceState::RushFour),
                (vec![1, 0, 1, 1, 1], PieceState::RushFour),
                (vec![1, 0, 0, 1, 1], PieceState::SleepThree),
                (vec![2, 1, 1, 1, 0, 0], PieceState::SleepThree),
                (vec![2, 1, 1, 0, 1, 0], PieceState::SleepThree),
                (vec![2, 1, 0, 1, 1, 0], PieceState::SleepThree),
                (vec![2, 1, 1, 0, 0, 0], PieceState::SleepTwo),
                (vec![2, 1, 0, 1, 0, 0], PieceState::SleepTwo),
                (vec![2, 1, 0, 0, 1, 0], PieceState::SleepTwo),
                (vec![2, 1, 0, 0, 0, 1], PieceState::SleepTwo),
                (vec![2, 0, 1, 1, 0, 0, 2], PieceState::SleepTwo),
                (vec![2, 0, 1, 0, 1, 0, 2], PieceState::SleepTwo),
            ]);

            count_subarrays(&sequence, &patterns)
                .iter()
                .map(|(state, count)| (*state, *count))
                .collect::<Vec<(PieceState, i32)>>()
        })
        .collect()
}

fn next_point_valid(point: &mut Point, dir: &(i32, i32)) -> bool {
    let (x, y) = *point;
    let (dx, dy) = *dir;
    let (x, y) = (x as isize + dx as isize, y as isize + dy as isize);

    if x < 0 || y < 0 || x >= 15 || y >= 15 {
        return false;
    }
    point.0 = x as usize;
    point.1 = y as usize;

    true
}

fn count_subarrays(
    array: &[i32],
    subarrays: &[(Vec<i32>, PieceState)],
) -> HashMap<PieceState, i32> {
    let array_len = array.len();
    let mut count = HashMap::new();
    let mut len = usize::MAX;

    for (subarray, _) in subarrays {
        if subarray.len() < len {
            len = subarray.len();
        }
    }

    if array_len < len {
        return count;
    }

    let mut i = 0;
    while i < array_len - len + 1 {
        let mut found = false;
        for (subarray, state) in subarrays {
            let subarray_len = subarray.len();
            if i + subarray_len <= array_len && &array[i..i + subarray_len] == subarray.as_slice() {
                let c = count.get(state).unwrap_or(&0) + 1;
                count.insert(*state, c);
                i += subarray_len;
                found = true;
                break;
            }
        }
        if !found {
            i += 1;
        }
    }

    count
}

fn loopback(sequence: &Vec<i32>) -> bool {
    let mut left = 0;
    let mut right = sequence.len() - 1;
    while left < right {
        if sequence[left] != sequence[right] {
            return false;
        }
        left += 1;
        right -= 1;
    }
    return true;
}

fn with_inv(patterns: Vec<(Vec<i32>, PieceState)>) -> Vec<(Vec<i32>, PieceState)> {
    let mut result = Vec::new();

    for (pattern, state) in patterns {
        result.push((pattern.clone(), state));
        if loopback(&pattern) {
            continue;
        }
        result.push((inv(&pattern), state))
    }

    result
}

fn inv(sequence: &Vec<i32>) -> Vec<i32> {
    let mut list = Vec::with_capacity(sequence.len());

    let mut i = sequence.len();
    while i > 0 {
        i -= 1;
        list.push(sequence[i]);
    }

    list
}

fn evaluate_location(game_state: &GameState, point: Point) -> i32 {
    let evaluate_impl = |piece, dir: (isize, isize)| {
        let mut count = 1;

        for dir in vec![dir, (-dir.0, -dir.1)] {
            let mut point = (point.0 as isize, point.1 as isize);
            loop {
                point.0 += dir.0;
                point.1 += dir.1;

                if point.0 < 0 || point.1 < 0 || point.0 >= 15 || point.1 >= 15 {
                    break;
                }

                if game_state.piece((point.0 as usize, point.1 as usize)) != piece {
                    break;
                }

                count += 1;
            }
        }

        count
    };

    let mut score = 0;

    for piece in vec![Piece::White, Piece::Black] {
        for dir in vec![(1, 0), (0, 1), (1, 1), (1, -1)] {
            let count = evaluate_impl(piece, dir);
            score += if count > 5 {
                100000
            } else if count == 5 {
                100000
            } else if count == 4 {
                1000
            } else if count == 3 {
                10
            } else {
                0
            };
        }
    }

    -score
}
