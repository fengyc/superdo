use std::collections::HashMap;
use std::fmt;
use std::mem;
use std::{
    collections::HashSet,
    io::{self},
};

#[derive(Debug, Default, Clone)]
struct SudokuPos {
    val: u32,
    nums: HashSet<u32>,
}

impl SudokuPos {
    pub fn new_with(val: u32) -> Self {
        let nums = if val == 0 {
            (1..10).collect()
        } else {
            HashSet::default()
        };
        Self { val, nums }
    }

    pub fn set(&mut self, val: u32) {
        self.val = val;
        if self.val != 0 {
            self.nums.clear()
        }
    }
}

struct SudokuBoard {
    board: Vec<Vec<SudokuPos>>,
}

impl SudokuBoard {
    pub fn empty() -> Self {
        let mut line = vec![];
        for _ in 0..9 {
            line.push(SudokuPos::new_with(0));
        }
        let board = vec![line.clone()];

        Self { board }
    }

    pub fn new_with(board: [[u32; 9]; 9]) -> Self {
        let mut b = Self::empty();
        for row in 0..9 {
            for col in 0..9 {
                b.set(board[row][col], row, col);
            }
        }
        b
    }

    pub fn set(&mut self, val: u32, row: usize, col: usize) {
        self.board[row][col].set(val);
        if val != 0 {
            // 清理行
            for i in 0..9 {
                self.board[row][i].nums.remove(&val);
            }
            // 清理列
            for i in 0..9 {
                self.board[i][col].nums.remove(&val);
            }
            // 清理 3x3 小格
            let row_s = (row / 3) * 3;
            let col_s = (col / 3) * 3;
            for i in 0..3 {
                for j in 0..3 {
                    self.board[row_s + i][col_s + j].nums.remove(&val);
                }
            }
        }
    }

    pub fn get(&self, row: usize, col: usize) -> &SudokuPos {
        &self.board[row][col]
    }

    pub fn get_mut(&mut self, row: usize, col: usize) -> &mut SudokuPos {
        &mut self.board[row][col]
    }

    pub fn solve(&mut self) -> bool {
        loop {
            let mut has_empty = false;
            let mut has_solution = false;

            for row in 0..9 {
                for col in 0..9 {
                    if self.board[row][col].val == 0 {
                        has_empty = true;

                        // 去掉当前所在行已有数字
                        for x in 0..9 {
                            if self.board[row][x].val != 0 {
                                self.board[row][col].nums.remove(&(self.board[row][x].val));
                                has_solution = true;
                            }
                        }
                        log::debug!("({}, {}) nums: {:?}", row, col, self.board[row][col].nums);

                        // 去掉当前所在列已有数字
                        for y in 0..9 {
                            if self.board[y][col].val != 0 {
                                self.board[row][col].nums.remove(&self.board[y][col].val);
                                has_solution = true;
                            }
                        }
                        log::debug!("({}, {}) nums: {:?}", row, col, self.board[row][col].nums);

                        // 去掉当前所在的 3x3 小格已有数字
                        let x = (col / 3) * 3;
                        let y = (row / 3) * 3;
                        for a in 0..3 {
                            for b in 0..3 {
                                if self.board[y + a][x + b].val != 0 {
                                    self.board[row][col]
                                        .nums
                                        .remove(&self.board[y + a][x + b].val);
                                    has_solution = true;
                                }
                            }
                        }
                        log::debug!("({}, {}) nums: {:?}", row, col, self.board[row][col].nums);

                        // 已经只剩下一个数字
                        if self.board[row][col].nums.len() == 1 {
                            self.board[row][col].val =
                                *self.board[row][col].nums.iter().next().unwrap();
                            self.board[row][col].nums.clear();
                            continue;
                        }

                        // 还有多个数字，3x3 小格中，剩下数字中，是否有唯一只能被当前位置使用的数字
                        let mut counts = HashMap::new();
                        for n in self.board[row][col].nums.iter() {
                            counts.insert(n.clone(), 1);
                        }
                        for a in 0..3 {
                            for b in 0..3 {
                                if self.board[y + a][x + b].val == 0 && row != y + a && col != x + b
                                {
                                    for n in self.board[y + a][x + b].nums.iter() {
                                        if let Some(count) = counts.get_mut(n) {
                                            *count += 1;
                                        }
                                    }
                                }
                            }
                        }
                        for (n, c) in counts {
                            if c == 1 {
                                log::debug!("({}, {}) only num: {}", row, col, n);
                                self.board[row][col].val == n;
                                self.board[row][col].nums.clear();
                                has_solution = true;
                            }
                        }
                    }
                }
            }

            // 已填满，或无解决方案
            if !has_empty {
                return true;
            }
            if !has_solution {
                return false;
            }
        }
    }
}

impl fmt::Display for SudokuBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..9 {
            for col in 0..9 {
                write!(f, "{}", self.board[row][col].val);
                if self.board[row][col].val == 0 {
                    write!(f, "{:?}", self.board[row][col].nums);
                }
                write!(f, " ");
            }
            write!(f, "\n");
        }
        Ok(())
    }
}

fn main() {
    env_logger::init();

    // 数独板
    let mut board = SudokuBoard::empty();

    println!("Input sodoku board, digit by digit (left to right, top to down, 0 for unknown digit, separated with or without space)");
    let mut total = 0;
    while total < 81 {
        let mut digits = String::new();
        if let Ok(n) = io::stdin().read_line(&mut digits) {
            for c in digits.chars() {
                if let Some(d) = c.to_digit(10) {
                    board.set(d, total / 9, total % 9);
                    total += 1;
                }
            }
        }
    }

    let solved = board.solve();

    // 打印最终状态
    println!("\nSolved: {}", solved);
    println!("{}", board);
}

#[test]
fn test_sodoku_1() {
    let mut board = [
        [0, 4, 0, 6, 1, 0, 9, 2, 5],
        [0, 5, 1, 0, 0, 0, 7, 4, 6],
        [9, 2, 6, 0, 0, 0, 8, 1, 3],
        [0, 8, 0, 0, 5, 0, 0, 7, 1],
        [0, 9, 0, 1, 0, 0, 0, 3, 2],
        [0, 1, 3, 4, 7, 0, 5, 9, 8],
        [0, 0, 0, 0, 0, 0, 1, 8, 9],
        [1, 6, 2, 8, 0, 0, 3, 5, 7],
        [8, 0, 9, 0, 0, 1, 2, 6, 4],
    ];
    let solved = sudoku(&mut board);
    assert_eq!(solved, true);
    assert_eq!(
        board,
        [
            [7, 4, 8, 6, 1, 3, 9, 2, 5],
            [3, 5, 1, 9, 2, 8, 7, 4, 6],
            [9, 2, 6, 7, 4, 5, 8, 1, 3],
            [2, 8, 4, 3, 5, 9, 6, 7, 1],
            [5, 9, 7, 1, 8, 6, 4, 3, 2],
            [6, 1, 3, 4, 7, 2, 5, 9, 8],
            [4, 3, 5, 2, 6, 7, 1, 8, 9],
            [1, 6, 2, 8, 9, 4, 3, 5, 7],
            [8, 7, 9, 5, 3, 1, 2, 6, 4],
        ]
    )
}
