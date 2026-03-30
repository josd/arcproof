use std::fmt::Write as _;
use std::io;

use crate::report::{CaseReport, ReportItem};

const ALL: u16 = 0x1FF; // bits 0..8 represent digits 1..9
const DEFAULT_PUZZLE_NAME: &str = "classic";
const DEFAULT_PUZZLE: &str =
    "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

#[derive(Clone, Copy, Debug)]
struct MoveLog {
    index: usize,
    value: u8,
    candidates_mask: u16,
    forced: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct SolveStats {
    givens: usize,
    blanks: usize,
    forced_moves: usize,
    guessed_moves: usize,
    recursive_nodes: usize,
    backtracks: usize,
    max_depth: usize,
}

#[derive(Clone, Debug)]
struct SearchState {
    cells: [u8; 81],
    row_used: [u16; 9],
    col_used: [u16; 9],
    box_used: [u16; 9],
    moves: Vec<MoveLog>,
}

#[derive(Clone, Debug)]
struct SolveResult {
    unique: Option<bool>,
    solution: [u8; 81],
    stats: SolveStats,
    moves: Vec<MoveLog>,
}

#[derive(Clone, Debug)]
struct ArcReport {
    answer: String,
    reason_why: String,
    checks: Vec<String>,
    puzzle_text: String,
    solution_text: String,
}

pub fn report() -> io::Result<CaseReport> {
    let puzzle = parse_puzzle(DEFAULT_PUZZLE)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let report = build_arc_report(&puzzle);
    ensure_checks(&report)?;

    Ok(CaseReport::new("sudoku")
        .with_answer(vec![
            ReportItem::text(report.answer),
            ReportItem::field("case", "sudoku"),
            ReportItem::field("default puzzle", DEFAULT_PUZZLE_NAME),
            ReportItem::text(format!("Puzzle
{}", report.puzzle_text.trim_end())),
            ReportItem::text(format!("Completed grid
{}", report.solution_text.trim_end())),
        ])
        .with_reason_why(vec![ReportItem::text(report.reason_why)])
        .with_check(
            report
                .checks
                .into_iter()
                .map(ReportItem::text)
                .collect::<Vec<_>>(),
        ))
}

pub fn run_and_print() -> io::Result<()> {
    let puzzle = parse_puzzle(DEFAULT_PUZZLE)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let report = build_arc_report(&puzzle);
    ensure_checks(&report)?;

    println!("=== Answer ===");
    println!("{}", report.answer);
    println!("case              : sudoku");
    println!("default puzzle    : {DEFAULT_PUZZLE_NAME}");
    println!();
    println!("Puzzle");
    print!("{}", report.puzzle_text);
    println!();
    println!("Completed grid");
    print!("{}", report.solution_text);
    println!();
    println!("=== Reason Why ===");
    println!("{}", report.reason_why);
    println!();
    println!("=== Check ===");
    for check in &report.checks {
        println!("{check}");
    }

    Ok(())
}

fn ensure_checks(report: &ArcReport) -> io::Result<()> {
    let bad = report.checks.iter().any(|line| {
        let lower = line.to_ascii_lowercase();
        lower.contains("failed") || lower.contains("not unique") || lower.contains("no valid")
    });
    if bad {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "sudoku check failed: one or more independent validations did not hold",
        ));
    }
    Ok(())
}

fn build_arc_report(puzzle: &[u8; 81]) -> ArcReport {
    let initial = match SearchState::from_puzzle(puzzle) {
        Ok(state) => state,
        Err(err) => {
            return ArcReport {
                answer: "The puzzle is invalid and cannot be solved as a standard Sudoku.".to_string(),
                reason_why: err,
                checks: vec!["C1 failed - the given clues already violate Sudoku rules.".to_string()],
                puzzle_text: format_board(puzzle),
                solution_text: format_board(puzzle),
            };
        }
    };

    let mut stats = SolveStats {
        givens: puzzle.iter().filter(|&&v| v != 0).count(),
        blanks: puzzle.iter().filter(|&&v| v == 0).count(),
        ..SolveStats::default()
    };

    let solved = solve(initial.clone(), &mut stats, 0);

    match solved {
        Some(final_state) => {
            let unique = is_unique_solution(&initial, 2);
            let solution = final_state.cells;
            let result = SolveResult {
                unique: Some(unique),
                solution,
                stats,
                moves: final_state.moves.clone(),
            };
            render_success_report(puzzle, &result)
        }
        None => ArcReport {
            answer: "No valid Sudoku solution exists for the supplied puzzle.".to_string(),
            reason_why: format!(
                "The solver explored {} search nodes with minimum-remaining-values branching and backtracked {} times, but every branch eventually contradicted the row, column, or box constraints.",
                stats.recursive_nodes, stats.backtracks
            ),
            checks: vec![
                "C1 OK - the given clues are internally consistent.".to_string(),
                "C2 OK - every explored assignment respected row, column, and box legality.".to_string(),
                "C3 failed - exhaustive search found no complete legal grid.".to_string(),
            ],
            puzzle_text: format_board(puzzle),
            solution_text: format_board(puzzle),
        },
    }
}

fn render_success_report(puzzle: &[u8; 81], result: &SolveResult) -> ArcReport {
    let givens = result.stats.givens;
    let filled = result.stats.blanks;
    let forced = result.stats.forced_moves;
    let guessed = result.stats.guessed_moves;
    let unique_text = match result.unique {
        Some(true) => "The solver also confirmed that the solution is unique.",
        Some(false) => "The solver found a valid solution, but there is more than one.",
        None => "The solver did not check uniqueness.",
    };

    let first_moves = summarize_moves(&result.moves, 8);

    let answer = if result.unique == Some(true) {
        "The puzzle is solved, and the completed grid is the unique valid Sudoku solution."
            .to_string()
    } else {
        "The puzzle is solved, and the completed grid is a valid Sudoku solution.".to_string()
    };

    let reason_why = format!(
        "The solver starts from {givens} clues and fills the remaining {filled} cells by combining constraint propagation with depth-first search. At each step it chooses the empty cell with the fewest legal digits, places forced singles immediately, and only guesses when more than one candidate remains. Across the search it made {forced} forced placements and tried {guessed} guesses, visited {} search nodes overall, and backtracked {} times before reaching the completed grid. {unique_text} Early steps: {first_moves}",
        result.stats.recursive_nodes,
        result.stats.backtracks,
    );

    let checks = build_checks(puzzle, result);

    ArcReport {
        answer,
        reason_why,
        checks,
        puzzle_text: format_board(puzzle),
        solution_text: format_board(&result.solution),
    }
}

fn build_checks(puzzle: &[u8; 81], result: &SolveResult) -> Vec<String> {
    let solution = &result.solution;
    let mut out = Vec::new();

    let givens_preserved = puzzle
        .iter()
        .zip(solution.iter())
        .all(|(&p, &s)| p == 0 || p == s);
    out.push(format!(
        "C1 {} - every given clue is preserved in the final grid.",
        ok(givens_preserved)
    ));

    let no_blanks = solution.iter().all(|&v| (1..=9).contains(&v));
    out.push(format!(
        "C2 {} - the final grid contains only digits 1 through 9, with no blanks left.",
        ok(no_blanks)
    ));

    let rows_ok = (0..9).all(|r| unit_is_complete((0..9).map(|c| solution[r * 9 + c])));
    out.push(format!(
        "C3 {} - each row contains every digit exactly once.",
        ok(rows_ok)
    ));

    let cols_ok = (0..9).all(|c| unit_is_complete((0..9).map(|r| solution[r * 9 + c])));
    out.push(format!(
        "C4 {} - each column contains every digit exactly once.",
        ok(cols_ok)
    ));

    let boxes_ok = (0..9).all(|b| {
        let br = (b / 3) * 3;
        let bc = (b % 3) * 3;
        unit_is_complete(
            (0..3).flat_map(|dr| (0..3).map(move |dc| solution[(br + dr) * 9 + (bc + dc)])),
        )
    });
    out.push(format!(
        "C5 {} - each 3×3 box contains every digit exactly once.",
        ok(boxes_ok)
    ));

    let legal_moves = replay_moves_are_legal(puzzle, &result.moves);
    out.push(format!(
        "C6 {} - replaying the recorded placements from the original puzzle remains legal at every step.",
        ok(legal_moves)
    ));

    let proof_path_guess_count = result.moves.iter().filter(|m| !m.forced).count();
    let search_story_ok = result.stats.recursive_nodes >= 1
        && result.stats.max_depth <= result.stats.blanks
        && result.moves.len() == result.stats.blanks
        && proof_path_guess_count <= result.stats.guessed_moves;
    out.push(format!(
        "C7 {} - the search statistics and the successful proof path are internally consistent.",
        ok(search_story_ok)
    ));

    let uniqueness_text = match result.unique {
        Some(true) => {
            "C8 OK - a second search found no alternative solution, so the solution is unique.".to_string()
        }
        Some(false) => {
            "C8 INFO - a second search found another solution, so the puzzle is not unique.".to_string()
        }
        None => "C8 INFO - uniqueness was not checked.".to_string(),
    };
    out.push(uniqueness_text);

    out
}

fn replay_moves_are_legal(puzzle: &[u8; 81], moves: &[MoveLog]) -> bool {
    let Ok(mut state) = SearchState::from_puzzle(puzzle) else {
        return false;
    };

    for mv in moves {
        if state.cells[mv.index] != 0 {
            return false;
        }
        let mask_now = state.candidates(mv.index);
        if mask_now != mv.candidates_mask {
            return false;
        }
        if mask_now & digit_mask(mv.value) == 0 {
            return false;
        }
        if mv.forced && popcount(mask_now) != 1 {
            return false;
        }
        if !state.place(mv.index, mv.value) {
            return false;
        }
    }
    true
}

fn solve(state: SearchState, stats: &mut SolveStats, depth: usize) -> Option<SearchState> {
    stats.recursive_nodes += 1;
    stats.max_depth = stats.max_depth.max(depth);

    let mut current = state;

    if !propagate_singles(&mut current, stats) {
        stats.backtracks += 1;
        return None;
    }

    let Some((idx, mask)) = select_unfilled_cell(&current) else {
        return Some(current);
    };

    for digit in mask_to_digits(mask) {
        let mut next = current.clone();
        let candidates_mask = next.candidates(idx);
        debug_assert!(candidates_mask & digit_mask(digit) != 0);
        next.moves.push(MoveLog {
            index: idx,
            value: digit,
            candidates_mask,
            forced: false,
        });
        stats.guessed_moves += 1;
        if !next.place(idx, digit) {
            continue;
        }
        if let Some(solved) = solve(next, stats, depth + 1) {
            return Some(solved);
        }
    }

    stats.backtracks += 1;
    None
}

fn propagate_singles(state: &mut SearchState, stats: &mut SolveStats) -> bool {
    loop {
        let mut progress = false;
        for idx in 0..81 {
            if state.cells[idx] != 0 {
                continue;
            }
            let mask = state.candidates(idx);
            let count = popcount(mask);
            if count == 0 {
                return false;
            }
            if count == 1 {
                let digit = mask_to_digits(mask)[0];
                state.moves.push(MoveLog {
                    index: idx,
                    value: digit,
                    candidates_mask: mask,
                    forced: true,
                });
                if !state.place(idx, digit) {
                    return false;
                }
                stats.forced_moves += 1;
                progress = true;
            }
        }
        if !progress {
            return true;
        }
    }
}

fn select_unfilled_cell(state: &SearchState) -> Option<(usize, u16)> {
    let mut best: Option<(usize, u16, usize)> = None;
    for idx in 0..81 {
        if state.cells[idx] != 0 {
            continue;
        }
        let mask = state.candidates(idx);
        let count = popcount(mask);
        match best {
            None => best = Some((idx, mask, count)),
            Some((_, _, best_count)) if count < best_count => best = Some((idx, mask, count)),
            _ => {}
        }
        if count == 2 {
            break;
        }
    }
    best.map(|(idx, mask, _)| (idx, mask))
}

fn is_unique_solution(initial: &SearchState, limit: usize) -> bool {
    let mut count = 0usize;
    count_solutions(initial.clone(), limit, &mut count);
    count == 1
}

fn count_solutions(state: SearchState, limit: usize, count: &mut usize) {
    if *count >= limit {
        return;
    }

    let mut current = state;
    let mut dummy_stats = SolveStats::default();
    if !propagate_singles(&mut current, &mut dummy_stats) {
        return;
    }

    let Some((idx, mask)) = select_unfilled_cell(&current) else {
        *count += 1;
        return;
    };

    for digit in mask_to_digits(mask) {
        if *count >= limit {
            return;
        }
        let mut next = current.clone();
        if next.place(idx, digit) {
            count_solutions(next, limit, count);
        }
    }
}

impl SearchState {
    fn from_puzzle(cells: &[u8; 81]) -> Result<Self, String> {
        let mut state = SearchState {
            cells: [0; 81],
            row_used: [0; 9],
            col_used: [0; 9],
            box_used: [0; 9],
            moves: Vec::new(),
        };

        for (idx, &value) in cells.iter().enumerate() {
            if value == 0 {
                continue;
            }
            if !(1..=9).contains(&value) {
                return Err(format!(
                    "Cell {} contains {value}, but only digits 1-9 or 0/. are allowed.",
                    idx + 1
                ));
            }
            if !state.place(idx, value) {
                let row = idx / 9 + 1;
                let col = idx % 9 + 1;
                return Err(format!(
                    "The given clues already conflict at row {row}, column {col}.",
                ));
            }
        }

        Ok(state)
    }

    fn candidates(&self, idx: usize) -> u16 {
        let row = idx / 9;
        let col = idx % 9;
        let bx = box_index(row, col);
        ALL & !(self.row_used[row] | self.col_used[col] | self.box_used[bx])
    }

    fn place(&mut self, idx: usize, value: u8) -> bool {
        if self.cells[idx] != 0 {
            return self.cells[idx] == value;
        }
        let row = idx / 9;
        let col = idx % 9;
        let bx = box_index(row, col);
        let bit = digit_mask(value);
        if (self.row_used[row] | self.col_used[col] | self.box_used[bx]) & bit != 0 {
            return false;
        }
        self.cells[idx] = value;
        self.row_used[row] |= bit;
        self.col_used[col] |= bit;
        self.box_used[bx] |= bit;
        true
    }
}

fn parse_puzzle(input: &str) -> Result<[u8; 81], String> {
    let filtered: Vec<char> = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '|' && *c != '+')
        .collect();
    if filtered.len() != 81 {
        return Err(format!(
            "Expected exactly 81 cells after removing whitespace, but found {}.",
            filtered.len()
        ));
    }

    let mut cells = [0u8; 81];
    for (i, ch) in filtered.into_iter().enumerate() {
        cells[i] = match ch {
            '1'..='9' => ch as u8 - b'0',
            '0' | '.' | '_' => 0,
            _ => return Err(format!("Unexpected character '{}' at position {}.", ch, i + 1)),
        };
    }
    Ok(cells)
}

fn format_board(cells: &[u8; 81]) -> String {
    let mut out = String::new();
    for r in 0..9 {
        if r > 0 && r % 3 == 0 {
            let _ = writeln!(out);
        }
        for c in 0..9 {
            if c > 0 && c % 3 == 0 {
                let _ = write!(out, "| ");
            }
            let v = cells[r * 9 + c];
            if v == 0 {
                let _ = write!(out, ". ");
            } else {
                let _ = write!(out, "{} ", v);
            }
        }
        let _ = writeln!(out);
    }
    out
}

fn summarize_moves(moves: &[MoveLog], limit: usize) -> String {
    if moves.is_empty() {
        return "no placements were needed".to_string();
    }
    let mut parts = Vec::new();
    for mv in moves.iter().take(limit) {
        let row = mv.index / 9 + 1;
        let col = mv.index % 9 + 1;
        let mode = if mv.forced { "forced" } else { "guess" };
        parts.push(format!("r{row}c{col}={}: {mode}", mv.value));
    }
    if moves.len() > limit {
        parts.push(format!("… and {} more placements", moves.len() - limit));
    }
    parts.join(", ")
}

fn unit_is_complete<I>(iter: I) -> bool
where
    I: IntoIterator<Item = u8>,
{
    let mut seen = 0u16;
    for v in iter {
        if !(1..=9).contains(&v) {
            return false;
        }
        let bit = digit_mask(v);
        if seen & bit != 0 {
            return false;
        }
        seen |= bit;
    }
    seen == ALL
}

fn ok(value: bool) -> &'static str {
    if value { "OK" } else { "failed" }
}

fn box_index(row: usize, col: usize) -> usize {
    (row / 3) * 3 + (col / 3)
}

fn digit_mask(value: u8) -> u16 {
    1u16 << (value - 1)
}

fn popcount(mask: u16) -> usize {
    mask.count_ones() as usize
}

fn mask_to_digits(mask: u16) -> Vec<u8> {
    (1..=9).filter(|&d| mask & digit_mask(d) != 0).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solves_default_puzzle() {
        let puzzle = parse_puzzle(DEFAULT_PUZZLE).unwrap();
        let report = build_arc_report(&puzzle);
        assert!(report.answer.contains("solved"));
        assert!(report.solution_text.contains("5 3 4 | 6 7 8 | 9 1 2"));
        assert!(report.solution_text.contains("3 4 5 | 2 8 6 | 1 7 9"));
    }

    #[test]
    fn rejects_invalid_puzzle() {
        let bad = parse_puzzle(&format!("11{}", "0".repeat(79))).unwrap();
        let report = build_arc_report(&bad);
        assert!(report.answer.contains("invalid"));
    }
}
