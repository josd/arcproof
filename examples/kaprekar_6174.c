
/*
 * kaprekar_6174.c
 *
 * For each non-repdigit four-digit number, this program repeatedly sorts the
 * digits descending and ascending and subtracts them. It checks that every
 * valid start reaches 6174 within seven steps.
 */

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void digits4(uint16_t n, int d[4]) {
    d[0] = (n / 1000) % 10;
    d[1] = (n / 100) % 10;
    d[2] = (n / 10) % 10;
    d[3] = n % 10;
}

static void sort4(int d[4], bool descending) {
    for (int i = 0; i < 4; ++i) for (int j = i + 1; j < 4; ++j) {
        if ((descending && d[j] > d[i]) || (!descending && d[j] < d[i])) {
            int t = d[i]; d[i] = d[j]; d[j] = t;
        }
    }
}

static uint16_t build4(const int d[4]) {
    return (uint16_t)(d[0] * 1000 + d[1] * 100 + d[2] * 10 + d[3]);
}

static bool has_two_distinct_digits(uint16_t n) {
    int d[4]; digits4(n, d);
    for (int i = 1; i < 4; ++i) if (d[i] != d[0]) return true;
    return false;
}

static uint16_t kaprekar_step(uint16_t n) {
    int hi[4], lo[4];
    digits4(n, hi);
    memcpy(lo, hi, sizeof(hi));
    sort4(hi, true);
    sort4(lo, false);
    return (uint16_t)(build4(hi) - build4(lo));
}

static size_t kaprekar_trace(uint16_t start, uint16_t *out, size_t cap) {
    size_t len = 0;
    uint16_t cur = start;
    while (len < cap) {
        out[len++] = cur;
        if (cur == 6174) break;
        cur = kaprekar_step(cur);
    }
    return len;
}

int main(void) {
    size_t valid_starts = 0, repdigits = 0, max_iterations = 0, worst_case_starts = 0;
    size_t hist[8] = {0};
    uint16_t worst_trace[16] = {0}, leading_trace[16] = {0};
    size_t worst_len = 0, leading_len = kaprekar_trace(2111, leading_trace, 16);
    bool all_reach = true, bound_ok = true;

    for (uint16_t start = 0; start <= 9999; ++start) {
        if (!has_two_distinct_digits(start)) { repdigits++; continue; }
        uint16_t trace[16];
        size_t len = kaprekar_trace(start, trace, 16);
        size_t steps = len ? len - 1 : 0;
        valid_starts++;
        if (trace[len - 1] != 6174) all_reach = false;
        if (steps > 7) bound_ok = false;
        if (steps < 8) hist[steps]++;
        if (steps > max_iterations) {
            max_iterations = steps;
            worst_case_starts = 1;
            memcpy(worst_trace, trace, len * sizeof(uint16_t));
            worst_len = len;
        } else if (steps == max_iterations) {
            worst_case_starts++;
        }
    }

    bool fixed_point_ok = kaprekar_step(6174) == 6174;
    bool histogram_ok = true;
    size_t hist_total = 0;
    for (size_t i = 0; i < 8; ++i) hist_total += hist[i];
    histogram_ok = hist_total == valid_starts;

    printf("=== Answer ===\n");
    printf("Every valid four-digit start tested reaches 6174, and all of them do so within seven iterations.\n");
    printf("\n=== Reason Why ===\n");
    printf("The program applies Kaprekar's routine to every non-repdigit start, records the iteration count, and keeps witness traces.\n");
    printf("valid starts checked: %zu\n", valid_starts);
    printf("repdigits excluded  : %zu\n", repdigits);
    printf("max iterations      : %zu\n", max_iterations);
    printf("worst-case starts   : %zu\n", worst_case_starts);
    printf("worst trace         : ");
    for (size_t i = 0; i < worst_len; ++i) { if (i) printf(" -> "); printf("%04u", worst_trace[i]); }
    printf("\nleading-zero trace  : ");
    for (size_t i = 0; i < leading_len; ++i) { if (i) printf(" -> "); printf("%04u", leading_trace[i]); }
    printf("\n\n=== Check ===\n");
    printf("6174 fixed point    : %s\n", fixed_point_ok ? "yes" : "no");
    printf("all starts reach it : %s\n", all_reach ? "yes" : "no");
    printf("bound <= 7 verified : %s\n", bound_ok ? "yes" : "no");
    printf("histogram total ok  : %s\n", histogram_ok ? "yes" : "no");
    return (fixed_point_ok && all_reach && bound_ok && histogram_ok) ? 0 : 1;
}
