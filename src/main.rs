use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use clap::Parser;
use env_logger::Env;

#[cfg(windows)]
const EOL: &'static str = "\r\n";
#[cfg(not(windows))]
const EOL: &str = "\n";

/// 数独位置
#[derive(Debug, Default, Clone)]
struct SudokuPos {
    /// 当前值，非 0 表示已有确定数字
    val: u32,
    /// 候选数字
    digits: HashSet<u32>,
}

impl PartialEq<u32> for SudokuPos {
    fn eq(&self, r: &u32) -> bool {
        self.val == *r
    }
}

impl SudokuPos {
    /// 创建一个新的位置，数值非 0 时为已有确定数字
    pub fn new_with(val: u32) -> Self {
        let digits = if val == 0 {
            (1..10).collect()
        } else {
            HashSet::default()
        };
        Self { val, digits }
    }
}

/// 数独棋盘， 9*9
#[derive(Debug, Clone)]
struct SudokuBoard {
    board: Vec<Vec<SudokuPos>>,
}

impl SudokuBoard {
    /// 创建一个空白的数独棋盘
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

    /// 创建一个已初始化的数独棋盘
    pub fn new_with(board: &[[u32; 9]; 9]) -> Self {
        let mut b = Self::empty();
        for row in 0..9 {
            for col in 0..9 {
                b.set(board[row][col], row, col);
            }
        }
        b
    }

    /// 设置某个位置的数值
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

    /// 获取某个位置
    pub fn get(&self, row: usize, col: usize) -> &SudokuPos {
        &self.board[row][col]
    }

    /// 获取某个位置，可变形式
    pub fn get_mut(&mut self, row: usize, col: usize) -> &mut SudokuPos {
        &mut self.board[row][col]
    }

    /// 是否有自由位置耗尽，此时无解
    pub fn exhausted(&self) -> bool {
        for row in &self.board {
            for col in row {
                if col.val == 0 && col.digits.is_empty() {
                    return true;
                }
            }
        }
        false
    }

    /// 进行数独求解
    pub fn solve(&mut self) -> bool {
        loop {
            let mut has_empty = false; // 是否还有空白的位置
            let mut has_changes = false; // 本次求解是否产生变化

            for row in 0..9 {
                for col in 0..9 {
                    if self.board[row][col].val == 0 {
                        has_empty = true;

                        // 失败
                        if self.board[row][col].digits.is_empty() {
                            return false;
                        }

                        // 已经只剩下一个数字
                        let pos = self.get_mut(row, col);
                        if pos.val == 0 && pos.digits.len() == 1 {
                            let val = *pos.digits.iter().next().unwrap();
                            self.set(val, row, col);
                            has_changes = true;
                            continue;
                        }

                        // 记录下日志，当前位置剩下的可用数字
                        log::debug!("({},{}) digits: {:?}", row, col, pos.digits);

                        // 检查是否只有当前位置才能使用的数字，进行数字统计
                        let mut digit_stats = HashMap::new();
                        for n in self.get(row, col).digits.iter() {
                            digit_stats.insert(*n, 1);
                        }
                        let digit_stats_cloned = digit_stats.clone();
                        // 辅助函数，如果只有计数为 1 中进行一次更新
                        let count_and_set =
                            |board: &mut SudokuBoard, stats: HashMap<u32, u32>| -> bool {
                                stats.iter().any(|(k, v)| {
                                    if *v == 1 {
                                        log::debug!("({},{}) solved: {}", row, col, k);
                                        board.set(*k, row, col);
                                        return true;
                                    }
                                    false
                                })
                            };

                        // 当前行统计，是否有唯一只能被当前使用的数字
                        let mut row_digit_stats = digit_stats_cloned.clone();
                        for i in 0..9 {
                            if i != col {
                                let pos = self.get(row, i);
                                for n in pos.digits.iter() {
                                    if let Some(count) = row_digit_stats.get_mut(n) {
                                        *count += 1;
                                    }
                                }
                            }
                        }
                        if count_and_set(self, row_digit_stats) {
                            has_changes = true;
                            continue;
                        }

                        // 当前列统计，是否有唯一只能被当前使用的数字
                        let mut col_digit_stats = digit_stats_cloned.clone();
                        for i in 0..9 {
                            if i != row {
                                let pos = self.get(i, col);
                                for n in pos.digits.iter() {
                                    if let Some(count) = col_digit_stats.get_mut(n) {
                                        *count += 1;
                                    }
                                }
                            }
                        }
                        if count_and_set(self, col_digit_stats) {
                            has_changes = true;
                            continue;
                        }

                        // 3x3 小格统计，剩下数字中，是否有唯一只能被当前位置使用的数字
                        let mut grid_digit_stats = digit_stats_cloned.clone();
                        let row_s = (row / 3) * 3;
                        let col_s = (col / 3) * 3;
                        for a in 0..3 {
                            for b in 0..3 {
                                let pos = self.get(row_s + a, col_s + b);
                                if (row_s + a != row || col_s + b != col) && pos.val == 0 {
                                    for n in pos.digits.iter() {
                                        if let Some(count) = grid_digit_stats.get_mut(n) {
                                            *count += 1;
                                        }
                                    }
                                }
                            }
                        }
                        if count_and_set(self, grid_digit_stats) {
                            has_changes = true;
                            continue;
                        }
                    }
                }
            }

            // 已填满
            if !has_empty {
                return true;
            }
            // 未填满，但是本次运行未有找到合适的方案
            if !has_changes {
                return false;
            }
        }
    }
}

impl fmt::Display for SudokuBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self
            .board
            .iter()
            .map(|row| {
                row.iter()
                    .map(|p| format!("{}", p.val))
                    .collect::<Vec<String>>()
                    .join("")
            })
            .collect::<Vec<String>>()
            .join(EOL);
        write!(f, "{}", s)
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
        true
    }
}

/// 求解上下文
struct ResolveCtx {
    /// 结果分隔符
    sep: String,
    /// 是否求解所有结果
    all: bool,
    /// 结果总数
    total: AtomicUsize,
}

/// 进行求解
fn resolve(ctx: Arc<ResolveCtx>, board: SudokuBoard, q: Vec<(usize, usize, u32)>) {
    if ctx.total.load(Ordering::Relaxed) > 0 && !ctx.all {
        return;
    }
    let mut board = board;
    let solved = board.solve();
    if solved {
        ctx.total.fetch_add(1, Ordering::Relaxed);
        log::debug!("q: {:?}", q);
        println!("{}\n{}", ctx.sep, board);
    } else if !board.exhausted() {
        // 固定某个自由参数
        let (free_row, free_col, _) = q.last().cloned().unwrap_or((0, 0, 0));
        let free_pos = free_row * 9 + free_col;
        let mut found_free = false;
        for row in free_row..9 {
            for col in 0..9 {
                let cur_pos = row * 9 + col;
                if cur_pos < free_pos {
                    continue;
                }
                let pos = board.get(row, col);
                if pos.val == 0 {
                    // 找到一个自由参数
                    found_free = true;
                    log::debug!("free pos: ({},{})={} {:?}", row, col, pos.val, pos.digits);
                    for digit in pos.digits.clone() {
                        let mut board2 = board.clone();
                        board2.set(digit, row, col);
                        let ctx_cloned = ctx.clone();
                        let mut q2 = q.clone();
                        q2.push((row, col, digit));
                        rayon::spawn(move || {
                            resolve(ctx_cloned.clone(), board2, q2);
                        });
                    }
                }
                if found_free {
                    break;
                }
            }
            if found_free {
                break;
            }
        }
    }
}

/// 回溯法找一个解
fn brute_force(board: &mut [[u32; 9]; 9], empty: &[[bool; 9]; 9], stack: &mut Vec<(usize, usize)>) -> bool {
    // 无法回溯或缺少初始值
    if stack.is_empty() {
        return false;
    }
    
    let (mut row,mut col) = stack.pop().unwrap(); // 当前行列

    // 为 (row,col) 查找下一个能用的值
    let next_digit = |board: &[[u32; 9]; 9], row: usize, col: usize| -> Option<u32> {
        // 注意，使用 +1 来越过旧值
        'next_digit: for digit in (board[row][col] + 1)..10 {
            // 同一行唯一
            for i in 0..9 {
                if board[row][i] == digit {
                    continue 'next_digit;
                }
            }
            // 同一列唯一
            for i in 0..9 {
                if board[i][col] == digit {
                    continue 'next_digit;
                }
            }
            // 3x3 小格唯一
            let row_grid = (row / 3) * 3;
            let col_grid = (col / 3) * 3;
            for row_x in 0..3 {
                for col_x in 0..3 {
                    if board[row_grid + row_x][col_grid + col_x] == digit {
                        continue 'next_digit;
                    }
                }
            }
            return Some(digit);
        }
        None
    };

    'back_trace: while row < 9 {
        while col < 9 {
            // 空位处理
            if board[row][col] == 0 || empty[row][col] {
                match next_digit(board, row, col) {
                    Some(digit) => {
                        // 找到可能值，压栈
                        board[row][col] = digit;
                        stack.push((row, col));
                    }
                    None => {
                        // 当前复位，向前回溯
                        board[row][col] = 0;
                        match stack.pop() {
                            Some((prev_row, prev_col)) => {
                                row = prev_row;
                                col = prev_col;
                                continue 'back_trace;
                            }
                            None => {
                                // 已不能回溯，失败
                                return false;
                            }
                        }
                    }
                }
            }
            // 成功找到结果，打印出来，然后向前回溯
            if row == 8 && col == 8 && board[row][col] != 0 {
                return true;
            }
            col += 1;
        }
        row += 1;
        col = 0;
    }
    false
}

fn resolve_2(board: &mut [[u32; 9]; 9]) {
    // 空位
    let mut empty = [[false; 9]; 9];
    for i in 0..9 {
        for j in 0..9 {
            empty[i][j] = board[i][j] == 0;
        }
    }
    // 回溯栈
    let mut stack = Vec::with_capacity(81);
    stack.push((0, 0));

    // 打印
    let dump_board = |board: &[[u32; 9]; 9]| -> String {
        board
            .map(|row| row.map(|d| d.to_string()).join(""))
            .join(EOL)
    };

    loop {
        let resolve = brute_force(board, &empty, &mut stack);
        if resolve {
            println!("---------\n{}", dump_board(board));
        } else {
            break;
        }
    }
    println!();
}

#[derive(Parser, Debug)]
#[command(
    version,
    about = "A sudoku puzzle solver.\n\n\
            Input the sudoku puzzle digit by digit (left to right, top to down, \
                0 for unknown digit, whitespace and other characters are ignored).\n\n\
            Output is a list of solutions separated by the chosen separator, then \
                followed by a blank line.",
    long_about = None
)]
struct Args {
    /// Show debug log
    #[arg(short, long)]
    debug: bool,

    /// Find all solutions
    #[arg(short, long)]
    all: bool,

    /// Solution separator
    #[arg(long, default_value = "---------")]
    sep: String,

    /// Max number of threads
    #[arg(long, default_value_t = num_cpus::get())]
    threads: usize,
}

fn main() {
    let args = Args::parse();

    // 日志初始化
    let log_level = if args.debug { "debug" } else { "info" };
    env_logger::init_from_env(Env::default().default_filter_or(log_level));

    // 线程池
    let num_threads = if args.threads > 0 {
        args.threads
    } else {
        num_cpus::get()
    };
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    // 结果格式
    let all = args.all;
    let sep = args.sep;

    // 数独板
    let mut board = [[0; 9]; 9];
    let mut count = 0;
    for line in io::stdin().lines() {
        for c in line.unwrap().chars().filter(|c| c.is_ascii_digit()) {
            // 读取
            board[count / 9][count % 9] = c.to_digit(10).unwrap();
            count += 1;
            // 进行求解
            if count == 81 {
                let ctx = Arc::new(ResolveCtx {
                    sep: sep.clone(),
                    all,
                    total: AtomicUsize::new(0),
                });
                let board = SudokuBoard::new_with(&board);
                let _ = thread_pool.install(|| resolve(ctx, board, vec![]));
                count = 0;
                println!();
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::brute_force;

    use super::SudokuBoard;

    #[ctor::ctor]
    fn init() {
        env_logger::init();
    }

    #[test]
    fn test_sudoku_1() {
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
    fn test_sudoku_2() {
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

    #[test]
    fn test_sudoku_3() {
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
        let mut empty = [[false; 9]; 9];
        for i in 0..9 {
            for j in 0..9 {
                empty[i][j] = board[i][j] == 0;
            }
        }
        let mut stack = Vec::with_capacity(81);
        stack.push((0, 0));
        let resolved = brute_force(&mut board, &empty, &mut stack);
        assert!(resolved);

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
        assert_eq!(board, board2);

        let resolved = brute_force(&mut board, &empty, &mut stack);
        assert!(!resolved);
    }
}
