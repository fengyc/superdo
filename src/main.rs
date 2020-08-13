use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::io;

#[derive(Debug, Default, Clone)]
struct SudokuPos {
    val: u32,
    digits: HashSet<u32>,
}

impl PartialEq<u32> for SudokuPos {
    fn eq(&self, r: &u32) -> bool {
        self.val == *r
    }
}

impl SudokuPos {
    pub fn new_with(val: u32) -> Self {
        let nums = if val == 0 {
            (1..10).collect()
        } else {
            HashSet::default()
        };
        Self { val, digits: nums }
    }
}

#[derive(Debug, Clone)]
struct SudokuBoard {
    board: Vec<Vec<SudokuPos>>,
}

impl SudokuBoard {
    pub fn empty() -> Self {
        let mut line = vec![];
        for _ in 0..9 {
            line.push(SudokuPos::new_with(0));
        }
        let mut board = vec![];
        for _ in 0..9 {
            board.push(line.clone());
        }

        Self { board }
    }

    pub fn new_with(board: &[[u32; 9]; 9]) -> Self {
        let mut b = Self::empty();
        for row in 0..9 {
            for col in 0..9 {
                b.set(board[row][col], row, col);
            }
        }
        b
    }

    pub fn set(&mut self, val: u32, row: usize, col: usize) {
        self.get_mut(row, col).val = val;
        if val != 0 {
            self.get_mut(row, col).digits.clear();

            // 清理行
            for i in 0..9 {
                self.get_mut(row, i).digits.remove(&val);
            }
            // 清理列
            for i in 0..9 {
                self.get_mut(i, col).digits.remove(&val);
            }
            // 清理 3x3 小格
            let row_s = (row / 3) * 3;
            let col_s = (col / 3) * 3;
            for i in 0..3 {
                for j in 0..3 {
                    self.get_mut(row_s + i, col_s + j).digits.remove(&val);
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

                        // 失败
                        if self.board[row][col].digits.len() == 0 {
                            return false;
                        }

                        // 已经只剩下一个数字
                        let pos = self.get_mut(row, col);
                        if pos.val == 0 && pos.digits.len() == 1 {
                            let val = pos.digits.iter().next().unwrap().clone();
                            self.set(val, row, col);
                            has_solution = true;
                            continue;
                        }

                        log::debug!("({}, {}) nums: {:?}", row, col, pos.digits);

                        // 检查是否只有当前位置才能使用的数字
                        let mut counts = HashMap::new();
                        for n in self.get(row, col).digits.iter() {
                            counts.insert(n.clone(), 1);
                        }
                        let counts_cloned = counts.clone();
                        // 辅助函数
                        let count_and_set =
                            |board: &mut SudokuBoard, counts: HashMap<u32, u32>| -> bool {
                                counts.iter().any(|(k, v)| {
                                    if *v == 1 {
                                        log::debug!("({}, {}) solved: {}", row, col, k);
                                        board.set(*k, row, col);
                                        return true;
                                    }
                                    false
                                })
                            };

                        // 当前行是否有唯一只能被当前使用的数字
                        let mut counts = counts_cloned.clone();
                        for i in 0..9 {
                            if i != col {
                                let pos = self.get(row, i);
                                for n in pos.digits.iter() {
                                    if let Some(count) = counts.get_mut(n) {
                                        *count += 1;
                                    }
                                }
                            }
                        }
                        if count_and_set(self, counts) {
                            has_solution = true;
                            continue;
                        }

                        // 当前列上是否有唯一只能被当前使用的数字
                        let mut counts = counts_cloned.clone();
                        for i in 0..9 {
                            if i != row {
                                let pos = self.get(i, col);
                                for n in pos.digits.iter() {
                                    if let Some(count) = counts.get_mut(n) {
                                        *count += 1;
                                    }
                                }
                            }
                        }
                        if count_and_set(self, counts) {
                            has_solution = true;
                            continue;
                        }

                        // 3x3 小格中，剩下数字中，是否有唯一只能被当前位置使用的数字
                        let mut counts = counts_cloned.clone();
                        let row_s = (row / 3) * 3;
                        let col_s = (col / 3) * 3;
                        for a in 0..3 {
                            for b in 0..3 {
                                let pos = self.get(row_s + a, col_s + b);
                                if (row_s + a != row || col_s + b != col) && pos.val == 0 {
                                    for n in pos.digits.iter() {
                                        if let Some(count) = counts.get_mut(n) {
                                            *count += 1;
                                        }
                                    }
                                }
                            }
                        }
                        if count_and_set(self, counts) {
                            has_solution = true;
                            continue;
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
                write!(f, "{}", self.board[row][col].val)?;
                if self.board[row][col].val == 0 {
                    write!(f, "{:?}", self.board[row][col].digits)?;
                }
                write!(f, " ")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl PartialEq<[[u32; 9]; 9]> for SudokuBoard {
    fn eq(&self, a: &[[u32; 9]; 9]) -> bool {
        for row in 0..9 {
            for col in 0..9 {
                if self.get(row, col) != &a[row][col] {
                    return false;
                }
            }
        }
        return true;
    }
}

fn main() {
    env_logger::init();

    // 数独板
    let mut board = [[0; 9]; 9];

    println!("Input sodoku board, digit by digit (left to right, top to down, 0 for unknown digit, separated with or without space)");
    let mut total = 0;
    while total < 81 {
        let mut digits = String::new();
        if let Ok(_) = io::stdin().read_line(&mut digits) {
            for c in digits.chars() {
                if let Some(d) = c.to_digit(10) {
                    board[total / 9][total % 9] = d;
                    total += 1;
                }
            }
        }
    }

    // 第一次计算
    let mut board = SudokuBoard::new_with(&board);
    let mut solved = board.solve();
    if solved {
        println!("\nSolved: {}", solved);
        println!("{}", board);
    }

    // 如果未解决，试数，假设只有单个自由数
    // TODO(fengyc) 支持更加多的自由数
    for row in 0..9 {
        for col in 0..9 {
            let pos = board.get(row, col);
            if pos.val == 0 {
                for n in &pos.digits {
                    let mut board2 = board.clone();
                    board2.set(*n, row, col);
                    let solved2 = board2.solve();
                    if solved2 {
                        solved = solved2;
                        println!("Solved");
                        println!("{}", board2);
                    }
                }
            }
        }
    }

    // 打印最终状态
    if solved {
        println!("Final solved: {}", solved);
    }
}

#[cfg(test)]
mod tests {
    use super::SudokuBoard;

    #[test]
    fn test_sodoku_1() {
        env_logger::init();

        let board = [
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
        let mut board = SudokuBoard::new_with(&board);
        println!("{}", board);

        let solved = board.solve();
        println!("\n{}", board);
        assert_eq!(solved, true);

        let board2 = [
            [7_u32, 4, 8, 6, 1, 3, 9, 2, 5],
            [3, 5, 1, 9, 2, 8, 7, 4, 6],
            [9, 2, 6, 7, 4, 5, 8, 1, 3],
            [2, 8, 4, 3, 5, 9, 6, 7, 1],
            [5, 9, 7, 1, 8, 6, 4, 3, 2],
            [6, 1, 3, 4, 7, 2, 5, 9, 8],
            [4, 3, 5, 2, 6, 7, 1, 8, 9],
            [1, 6, 2, 8, 9, 4, 3, 5, 7],
            [8, 7, 9, 5, 3, 1, 2, 6, 4],
        ];
        assert!(board == board2);
    }

    #[test]
    fn test_sodoku_2() {
        let board = [
            [0, 4, 6, 9, 0, 3, 0, 0, 0],
            [0, 0, 3, 0, 5, 0, 0, 6, 0],
            [9, 0, 0, 0, 0, 2, 0, 0, 3],
            [0, 0, 5, 0, 0, 6, 0, 0, 0],
            [8, 0, 0, 0, 0, 0, 0, 1, 0],
            [0, 1, 0, 7, 8, 0, 2, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 5, 0],
            [0, 8, 1, 3, 0, 0, 0, 0, 7],
            [0, 0, 0, 8, 0, 0, 1, 0, 4],
        ];
        let mut board = SudokuBoard::new_with(&board);
        println!("{}", board);

        let mut solved = board.solve();
        assert_eq!(solved, false);

        // 尝试选择
        let mut board2 = board.clone();
        'outer: for row in 0..9 {
            for col in 0..9 {
                let pos = board.get(row, col);
                if pos.val == 0 {
                    for n in &pos.digits {
                        board2 = board.clone();
                        board2.set(*n, row, col);
                        solved = board2.solve();
                        if solved {
                            break 'outer;
                        }
                    }
                }
            }
        }

        assert_eq!(solved, true);
        let result = [
            [1, 4, 6, 9, 7, 3, 5, 8, 2],
            [7, 2, 3, 4, 5, 8, 9, 6, 1],
            [9, 5, 8, 6, 1, 2, 4, 7, 3],
            [3, 7, 5, 1, 2, 6, 8, 4, 9],
            [8, 9, 2, 5, 3, 4, 7, 1, 6],
            [6, 1, 4, 7, 8, 9, 2, 3, 5],
            [4, 6, 7, 2, 9, 1, 3, 5, 8],
            [2, 8, 1, 3, 4, 5, 6, 9, 7],
            [5, 3, 9, 8, 6, 7, 1, 2, 4],
        ];
        assert!(board2 == result);
        println!("{}", board2);
    }
}
