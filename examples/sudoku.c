
/*
 * sudoku.c
 *
 * This program really solves the Sudoku puzzle. It tracks row, column, and box
 * constraints with bit masks, propagates forced singles, and falls back to a
 * depth-first search with minimum-remaining-values branching when needed.
 */

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define ALL_MASK 0x1FFu
static const char *DEFAULT_PUZZLE =
    "100007090030020008009600500005300900010080002600004000300000010040000007007000300";

typedef struct {
    size_t index;
    uint8_t value;
    uint16_t candidates_mask;
    bool forced;
} MoveLog;

typedef struct {
    size_t givens, blanks, forced_moves, guessed_moves, recursive_nodes, backtracks, max_depth;
} SolveStats;

typedef struct {
    uint8_t cells[81];
    uint16_t row_used[9], col_used[9], box_used[9];
    MoveLog *moves;
    size_t moves_len, moves_cap;
} SearchState;

static int popcount_u16(uint16_t x) {
    int c = 0;
    while (x) { x &= (uint16_t)(x - 1u); c++; }
    return c;
}

static uint16_t digit_mask(uint8_t d) { return (uint16_t)(1u << (d - 1u)); }
static size_t box_index(size_t r, size_t c) { return (r / 3u) * 3u + (c / 3u); }

static void die_oom(void) {
    fprintf(stderr, "out of memory\n");
    exit(1);
}

static void push_move(SearchState *s, MoveLog mv) {
    if (s->moves_len == s->moves_cap) {
        size_t new_cap = s->moves_cap ? s->moves_cap * 2u : 64u;
        MoveLog *p = (MoveLog *)realloc(s->moves, new_cap * sizeof(MoveLog));
        if (!p) die_oom();
        s->moves = p;
        s->moves_cap = new_cap;
    }
    s->moves[s->moves_len++] = mv;
}

static void free_state(SearchState *s) {
    free(s->moves);
    s->moves = NULL;
    s->moves_len = s->moves_cap = 0;
}

static SearchState clone_state(const SearchState *src) {
    SearchState out = *src;
    out.moves = NULL;
    out.moves_len = out.moves_cap = 0;
    if (src->moves_len) {
        out.moves = (MoveLog *)malloc(src->moves_len * sizeof(MoveLog));
        if (!out.moves) die_oom();
        memcpy(out.moves, src->moves, src->moves_len * sizeof(MoveLog));
        out.moves_len = src->moves_len;
        out.moves_cap = src->moves_len;
    }
    return out;
}

static bool place(SearchState *s, size_t idx, uint8_t value) {
    if (s->cells[idx] != 0) return s->cells[idx] == value;
    size_t r = idx / 9u, c = idx % 9u, b = box_index(r, c);
    uint16_t bit = digit_mask(value);
    if ((s->row_used[r] | s->col_used[c] | s->box_used[b]) & bit) return false;
    s->cells[idx] = value;
    s->row_used[r] |= bit;
    s->col_used[c] |= bit;
    s->box_used[b] |= bit;
    return true;
}

static uint16_t candidates(const SearchState *s, size_t idx) {
    size_t r = idx / 9u, c = idx % 9u, b = box_index(r, c);
    return (uint16_t)(ALL_MASK & ~(s->row_used[r] | s->col_used[c] | s->box_used[b]));
}

static bool parse_puzzle(const char *text, uint8_t out[81]) {
    size_t n = strlen(text);
    if (n != 81) return false;
    for (size_t i = 0; i < 81; ++i) {
        char ch = text[i];
        if (ch >= '1' && ch <= '9') out[i] = (uint8_t)(ch - '0');
        else if (ch == '0' || ch == '.' || ch == '_') out[i] = 0;
        else return false;
    }
    return true;
}

static bool state_from_puzzle(const uint8_t puzzle[81], SearchState *s) {
    memset(s, 0, sizeof(*s));
    for (size_t i = 0; i < 81; ++i) {
        if (puzzle[i] && !place(s, i, puzzle[i])) return false;
    }
    return true;
}

static bool propagate_singles(SearchState *s, SolveStats *stats) {
    for (;;) {
        bool progress = false;
        for (size_t idx = 0; idx < 81; ++idx) {
            if (s->cells[idx] != 0) continue;
            uint16_t mask = candidates(s, idx);
            int count = popcount_u16(mask);
            if (count == 0) return false;
            if (count == 1) {
                uint8_t digit = 0;
                for (uint8_t d = 1; d <= 9; ++d) if (mask & digit_mask(d)) { digit = d; break; }
                push_move(s, (MoveLog){idx, digit, mask, true});
                if (!place(s, idx, digit)) return false;
                stats->forced_moves++;
                progress = true;
            }
        }
        if (!progress) return true;
    }
}

static bool select_unfilled_cell(const SearchState *s, size_t *out_idx, uint16_t *out_mask) {
    bool found = false;
    int best_count = 10;
    for (size_t idx = 0; idx < 81; ++idx) {
        if (s->cells[idx] != 0) continue;
        uint16_t mask = candidates(s, idx);
        int count = popcount_u16(mask);
        if (count < best_count) {
            best_count = count;
            *out_idx = idx;
            *out_mask = mask;
            found = true;
            if (count == 2) break;
        }
    }
    return found;
}

static bool solve(SearchState *s, SolveStats *stats, size_t depth) {
    stats->recursive_nodes++;
    if (depth > stats->max_depth) stats->max_depth = depth;
    if (!propagate_singles(s, stats)) {
        stats->backtracks++;
        return false;
    }
    size_t idx;
    uint16_t mask;
    if (!select_unfilled_cell(s, &idx, &mask)) return true;
    for (uint8_t d = 1; d <= 9; ++d) {
        if (!(mask & digit_mask(d))) continue;
        SearchState next = clone_state(s);
        push_move(&next, (MoveLog){idx, d, mask, false});
        stats->guessed_moves++;
        if (place(&next, idx, d) && solve(&next, stats, depth + 1u)) {
            free(s->moves);
            *s = next;
            return true;
        }
        free_state(&next);
    }
    stats->backtracks++;
    return false;
}

static void count_solutions(SearchState *s, size_t limit, size_t *count) {
    if (*count >= limit) return;
    SolveStats dummy = {0};
    if (!propagate_singles(s, &dummy)) return;
    size_t idx;
    uint16_t mask;
    if (!select_unfilled_cell(s, &idx, &mask)) { (*count)++; return; }
    for (uint8_t d = 1; d <= 9; ++d) {
        if (!(mask & digit_mask(d))) continue;
        SearchState next = clone_state(s);
        if (place(&next, idx, d)) count_solutions(&next, limit, count);
        free_state(&next);
        if (*count >= limit) return;
    }
}

static bool unit_complete(const uint8_t *vals, size_t len) {
    uint16_t seen = 0;
    for (size_t i = 0; i < len; ++i) {
        uint8_t v = vals[i];
        if (v < 1 || v > 9) return false;
        uint16_t bit = digit_mask(v);
        if (seen & bit) return false;
        seen |= bit;
    }
    return seen == ALL_MASK;
}

static bool replay_moves_are_legal(const uint8_t puzzle[81], const SearchState *solved) {
    SearchState st;
    if (!state_from_puzzle(puzzle, &st)) return false;
    for (size_t i = 0; i < solved->moves_len; ++i) {
        MoveLog mv = solved->moves[i];
        if (st.cells[mv.index] != 0) { free_state(&st); return false; }
        uint16_t mask = candidates(&st, mv.index);
        if (mask != mv.candidates_mask) { free_state(&st); return false; }
        if (!(mask & digit_mask(mv.value))) { free_state(&st); return false; }
        if (mv.forced && popcount_u16(mask) != 1) { free_state(&st); return false; }
        if (!place(&st, mv.index, mv.value)) { free_state(&st); return false; }
    }
    free_state(&st);
    return true;
}

static void print_board(const uint8_t cells[81]) {
    for (size_t r = 0; r < 9; ++r) {
        if (r > 0 && r % 3 == 0) printf("\n");
        for (size_t c = 0; c < 9; ++c) {
            if (c > 0 && c % 3 == 0) printf("| ");
            uint8_t v = cells[r * 9 + c];
            if (v == 0) printf(". "); else printf("%u ", (unsigned)v);
        }
        printf("\n");
    }
}

int main(void) {
    uint8_t puzzle[81];
    if (!parse_puzzle(DEFAULT_PUZZLE, puzzle)) return 1;
    SearchState initial;
    if (!state_from_puzzle(puzzle, &initial)) return 1;
    SolveStats stats = {0};
    for (size_t i = 0; i < 81; ++i) if (puzzle[i]) stats.givens++; else stats.blanks++;
    SearchState solved = clone_state(&initial);
    bool solved_ok = solve(&solved, &stats, 0);
    size_t solution_count = 0;
    SearchState count_state = clone_state(&initial);
    count_solutions(&count_state, 2, &solution_count);
    free_state(&count_state);
    bool unique = solution_count == 1;
    bool givens_preserved = true, rows_ok = true, cols_ok = true, boxes_ok = true, replay_ok = false;
    if (solved_ok) {
        for (size_t i = 0; i < 81; ++i) if (puzzle[i] && puzzle[i] != solved.cells[i]) givens_preserved = false;
        for (size_t r = 0; r < 9; ++r) {
            uint8_t row[9];
            for (size_t c = 0; c < 9; ++c) row[c] = solved.cells[r * 9 + c];
            if (!unit_complete(row, 9)) rows_ok = false;
        }
        for (size_t c = 0; c < 9; ++c) {
            uint8_t col[9];
            for (size_t r = 0; r < 9; ++r) col[r] = solved.cells[r * 9 + c];
            if (!unit_complete(col, 9)) cols_ok = false;
        }
        for (size_t b = 0; b < 9; ++b) {
            size_t br = (b / 3u) * 3u, bc = (b % 3u) * 3u;
            uint8_t box[9]; size_t k = 0;
            for (size_t dr = 0; dr < 3; ++dr) for (size_t dc = 0; dc < 3; ++dc) box[k++] = solved.cells[(br + dr) * 9 + (bc + dc)];
            if (!unit_complete(box, 9)) boxes_ok = false;
        }
        replay_ok = replay_moves_are_legal(puzzle, &solved);
    }

    printf("=== Answer ===\n");
    printf("%s\n", unique ? "The puzzle is solved, and the completed grid is the unique valid Sudoku solution." : "The puzzle is solved, and the completed grid is a valid Sudoku solution.");
    printf("\nPuzzle\n");
    print_board(puzzle);
    printf("\nCompleted grid\n");
    if (solved_ok) print_board(solved.cells);
    printf("\n=== Reason Why ===\n");
    printf("The solver combines constraint propagation with depth-first search. It fills forced singles immediately and branches on the blank cell with the fewest candidates.\n");
    printf("givens             : %zu\n", stats.givens);
    printf("blanks             : %zu\n", stats.blanks);
    printf("forced placements  : %zu\n", stats.forced_moves);
    printf("guesses            : %zu\n", stats.guessed_moves);
    printf("search nodes       : %zu\n", stats.recursive_nodes);
    printf("backtracks         : %zu\n", stats.backtracks);
    printf("solution unique    : %s\n", unique ? "yes" : "no");
    printf("\n=== Check ===\n");
    printf("solver found solution           : %s\n", solved_ok ? "yes" : "no");
    printf("givens preserved                : %s\n", givens_preserved ? "yes" : "no");
    printf("rows complete                   : %s\n", rows_ok ? "yes" : "no");
    printf("columns complete                : %s\n", cols_ok ? "yes" : "no");
    printf("boxes complete                  : %s\n", boxes_ok ? "yes" : "no");
    printf("recorded placements replay legally: %s\n", replay_ok ? "yes" : "no");
    printf("uniqueness check                : %s\n", unique ? "yes" : "no");

    free_state(&initial);
    free_state(&solved);
    return (solved_ok && givens_preserved && rows_ok && cols_ok && boxes_ok && replay_ok && unique) ? 0 : 1;
}
