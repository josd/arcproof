
/*
 * collatz_1000.c
 *
 * This standalone program checks the Collatz process for every starting value
 * from 1 through 10,000. It does not trust a precomputed report. Instead it
 * generates traces, memoizes stopping times, and verifies several witnesses.
 */

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define MAX_START 10000ULL
#define SAMPLE_START 27ULL

typedef struct {
    uint64_t *data;
    size_t len;
    size_t cap;
} U64Vec;

typedef struct {
    size_t starts_checked;
    bool all_reach_one;
    size_t max_steps;
    uint64_t max_steps_start;
    uint64_t highest_peak;
    uint64_t peak_start;
    size_t sample_trace_steps;
    uint64_t sample_trace_peak;
    bool sample_trace_rule_valid;
    bool max_steps_witness_verified;
    bool peak_witness_verified;
} CollatzReport;

static void die_oom(void) {
    fprintf(stderr, "out of memory\n");
    exit(1);
}

static void vec_push(U64Vec *v, uint64_t x) {
    if (v->len == v->cap) {
        size_t new_cap = v->cap ? v->cap * 2 : 32;
        uint64_t *p = (uint64_t *)realloc(v->data, new_cap * sizeof(uint64_t));
        if (!p) die_oom();
        v->data = p;
        v->cap = new_cap;
    }
    v->data[v->len++] = x;
}

static void vec_free(U64Vec *v) {
    free(v->data);
    v->data = NULL;
    v->len = v->cap = 0;
}

static uint64_t collatz_step(uint64_t n) {
    return (n % 2ULL == 0ULL) ? (n / 2ULL) : (3ULL * n + 1ULL);
}

static U64Vec collatz_trace(uint64_t start) {
    U64Vec trace = {0};
    uint64_t cur = start;
    vec_push(&trace, cur);
    while (cur != 1ULL) {
        cur = collatz_step(cur);
        vec_push(&trace, cur);
    }
    return trace;
}

static bool trace_follows_rule(const U64Vec *trace) {
    if (trace->len == 0 || trace->data[trace->len - 1] != 1ULL) return false;
    for (size_t i = 0; i + 1 < trace->len; ++i) {
        if (collatz_step(trace->data[i]) != trace->data[i + 1]) return false;
    }
    return true;
}

static CollatzReport evaluate(void) {
    size_t *memo = (size_t *)calloc((size_t)MAX_START + 1U, sizeof(size_t));
    bool *known = (bool *)calloc((size_t)MAX_START + 1U, sizeof(bool));
    if (!memo || !known) die_oom();
    known[1] = true;
    memo[1] = 0;

    CollatzReport r = {0};
    r.all_reach_one = true;
    r.max_steps_start = 1;
    r.highest_peak = 1;
    r.peak_start = 1;

    for (uint64_t start = 1; start <= MAX_START; ++start) {
        r.starts_checked++;
        U64Vec trace = collatz_trace(start);
        if (trace.len == 0 || trace.data[trace.len - 1] != 1ULL) r.all_reach_one = false;

        uint64_t peak = start;
        for (size_t i = 0; i < trace.len; ++i) {
            if (trace.data[i] > peak) peak = trace.data[i];
        }

        U64Vec path = {0};
        uint64_t cur = start;
        while (!(cur <= MAX_START && known[cur])) {
            vec_push(&path, cur);
            cur = collatz_step(cur);
        }
        size_t steps = memo[cur];
        for (size_t i = path.len; i > 0; --i) {
            steps += 1;
            uint64_t v = path.data[i - 1];
            if (v <= MAX_START) {
                known[v] = true;
                memo[v] = steps;
            }
        }
        vec_free(&path);

        if (steps > r.max_steps) {
            r.max_steps = steps;
            r.max_steps_start = start;
        }
        if (peak > r.highest_peak) {
            r.highest_peak = peak;
            r.peak_start = start;
        }
        vec_free(&trace);
    }

    U64Vec sample = collatz_trace(SAMPLE_START);
    U64Vec hardest = collatz_trace(r.max_steps_start);
    U64Vec highest = collatz_trace(r.peak_start);
    r.sample_trace_steps = sample.len ? sample.len - 1 : 0;
    r.sample_trace_peak = SAMPLE_START;
    for (size_t i = 0; i < sample.len; ++i) if (sample.data[i] > r.sample_trace_peak) r.sample_trace_peak = sample.data[i];
    r.sample_trace_rule_valid = trace_follows_rule(&sample);
    r.max_steps_witness_verified = hardest.len && hardest.len - 1 == r.max_steps;
    uint64_t peak_check = r.peak_start;
    for (size_t i = 0; i < highest.len; ++i) if (highest.data[i] > peak_check) peak_check = highest.data[i];
    r.peak_witness_verified = peak_check == r.highest_peak;
    vec_free(&sample);
    vec_free(&hardest);
    vec_free(&highest);
    free(memo);
    free(known);
    return r;
}

int main(void) {
    CollatzReport r = evaluate();
    printf("=== Answer ===\n");
    printf("For starts 1..=%llu, every tested value reaches 1 under the Collatz map.\n", (unsigned long long)MAX_START);
    printf("\n=== Reason Why ===\n");
    printf("The program applies the standard Collatz rule, memoizes stopping times, and tracks the hardest witnesses.\n");
    printf("starts checked      : %zu\n", r.starts_checked);
    printf("max steps           : %zu\n", r.max_steps);
    printf("max-steps start     : %llu\n", (unsigned long long)r.max_steps_start);
    printf("highest peak        : %llu\n", (unsigned long long)r.highest_peak);
    printf("peak start          : %llu\n", (unsigned long long)r.peak_start);
    printf("trace(27) steps     : %zu\n", r.sample_trace_steps);
    printf("trace(27) peak      : %llu\n", (unsigned long long)r.sample_trace_peak);
    printf("\n=== Check ===\n");
    printf("all reach 1         : %s\n", r.all_reach_one ? "yes" : "no");
    printf("trace(27) valid     : %s\n", r.sample_trace_rule_valid ? "yes" : "no");
    printf("max-steps witness ok: %s\n", r.max_steps_witness_verified ? "yes" : "no");
    printf("peak witness ok     : %s\n", r.peak_witness_verified ? "yes" : "no");
    return (r.all_reach_one && r.sample_trace_rule_valid && r.max_steps_witness_verified && r.peak_witness_verified) ? 0 : 1;
}
