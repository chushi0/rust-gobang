use std::fmt::{Display, Error};

use anyhow::{anyhow, Result};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Piece {
    Empty,
    White,
    Black,
}

pub type BoardState = [[Piece; 15]; 15];

pub type Point = (usize, usize);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GameState {
    board: BoardState,
    current_turn: Piece,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEnd {
    NotEnd,
    Win,
    Lost,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            board: [[Piece::Empty; 15]; 15],
            current_turn: Piece::Black, // 通常，黑子先手
        }
    }

    pub fn current_turn(&self) -> Piece {
        self.current_turn
    }

    pub fn piece(&self, point: Point) -> Piece {
        self.board[point.0][point.1]
    }

    pub fn make_move(&mut self, point: Point) -> Result<GameEnd> {
        let (x, y) = point;
        if x >= 15 || y >= 15 || self.board[x][y] != Piece::Empty {
            return Err(anyhow!("Invalid move"));
        }

        self.board[x][y] = self.current_turn;
        self.current_turn = match self.current_turn {
            Piece::Black => Piece::White,
            Piece::White => Piece::Black,
            _ => unreachable!(),
        };

        let link_count = self.get_max_link_count(point);
        if link_count >= 5 {
            return Ok(GameEnd::Win);
        }

        Ok(GameEnd::NotEnd)
    }

    fn get_max_link_count(&self, point: Point) -> i32 {
        let piece = self.piece(point);
        let mut link_count = 0;

        for dir in vec![(1, 0), (0, 1), (1, 1), (1, -1)] {
            let mut count = 1;

            for dir in vec![dir, (-dir.0, -dir.1)] {
                let mut point = (point.0 as isize, point.1 as isize);
                loop {
                    point.0 += dir.0;
                    point.1 += dir.1;

                    if point.0 < 0 || point.1 < 0 || point.0 >= 15 || point.1 >= 15 {
                        break;
                    }

                    if self.piece((point.0 as usize, point.1 as usize)) != piece {
                        break;
                    }

                    count += 1;
                }
            }

            link_count = link_count.max(count);
        }

        -link_count
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "current turn: {}",
            match self.current_turn {
                Piece::Empty => unreachable!(),
                Piece::White => "White",
                Piece::Black => "Black",
            }
        )?;
        write!(f, " ")?;
        for i in 0..15 {
            write!(f, " ")?;
            write!(f, "{}", i % 10)?;
        }
        writeln!(f)?;

        for y in 0..15 {
            write!(f, "{} ", y % 10)?;
            for x in 0..15 {
                write!(
                    f,
                    "{}",
                    match self.piece((x, y)) {
                        Piece::Empty => match (x, y) {
                            (0, 0) => "┌",
                            (0, 14) => "└",
                            (14, 0) => "┐",
                            (14, 14) => "┘",
                            (0, _) => "├",
                            (14, _) => "┤",
                            (_, 0) => "┬",
                            (_, 14) => "┴",
                            _ => "┼",
                        },
                        Piece::White => "○",
                        Piece::Black => "●",
                    }
                )?;
                if x != 14 {
                    write!(f, "─")?;
                }
            }
            if y != 14 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
