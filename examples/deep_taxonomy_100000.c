
/*
 * deep_taxonomy_100000.c
 *
 * This file models a long implication chain. From N(0), each N(i) derives
 * N(i+1), I(i+1), and J(i+1). Reaching N(100000) derives A2, and A2 derives
 * the final goal.
 */

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

#define MAX_N 100000
#define RULE_COUNT 100002
#define EXPECTED_TYPE_FACTS (3 * MAX_N + 2)
#define EXPECTED_DERIVED_FACTS (EXPECTED_TYPE_FACTS + 1)

typedef struct {
    int kind; /* 0=N, 1=I, 2=J, 3=A2 */
    int index;
} Fact;

typedef struct {
    Fact *data;
    size_t head;
    size_t tail;
    size_t cap;
} Queue;

static void die_oom(void) {
    fprintf(stderr, "out of memory\n");
    exit(1);
}

static void q_push(Queue *q, Fact f) {
    if (q->tail == q->cap) {
        size_t new_cap = q->cap ? q->cap * 2 : 1024;
        Fact *p = (Fact *)realloc(q->data, new_cap * sizeof(Fact));
        if (!p) die_oom();
        q->data = p;
        q->cap = new_cap;
    }
    q->data[q->tail++] = f;
}

static bool q_pop(Queue *q, Fact *out) {
    if (q->head == q->tail) return false;
    *out = q->data[q->head++];
    return true;
}

static bool insert_flag(bool *slot) {
    if (*slot) return false;
    *slot = true;
    return true;
}

static void enqueue_class(int kind, int index, Queue *q, bool *n_seen, bool *i_seen, bool *j_seen, bool *a2_seen) {
    bool inserted = false;
    if (kind == 0) inserted = insert_flag(&n_seen[index]);
    else if (kind == 1) inserted = insert_flag(&i_seen[index]);
    else if (kind == 2) inserted = insert_flag(&j_seen[index]);
    else inserted = insert_flag(a2_seen);
    if (inserted) q_push(q, (Fact){kind, index});
}

int main(void) {
    bool *n_seen = (bool *)calloc(MAX_N + 1, sizeof(bool));
    bool *i_seen = (bool *)calloc(MAX_N + 1, sizeof(bool));
    bool *j_seen = (bool *)calloc(MAX_N + 1, sizeof(bool));
    bool a2_seen = false;
    bool goal_seen = false;
    Queue q = {0};
    if (!n_seen || !i_seen || !j_seen) die_oom();

    enqueue_class(0, 0, &q, n_seen, i_seen, j_seen, &a2_seen);

    Fact cur;
    while (q_pop(&q, &cur)) {
        if (cur.kind == 0 && cur.index < MAX_N) {
            int next = cur.index + 1;
            enqueue_class(0, next, &q, n_seen, i_seen, j_seen, &a2_seen);
            enqueue_class(1, next, &q, n_seen, i_seen, j_seen, &a2_seen);
            enqueue_class(2, next, &q, n_seen, i_seen, j_seen, &a2_seen);
        } else if (cur.kind == 0 && cur.index == MAX_N) {
            enqueue_class(3, 0, &q, n_seen, i_seen, j_seen, &a2_seen);
        } else if (cur.kind == 3) {
            goal_seen = true;
        }
    }

    size_t type_facts = 0;
    for (int i = 0; i <= MAX_N; ++i) {
        if (n_seen[i]) type_facts++;
        if (i > 0 && i_seen[i]) type_facts++;
        if (i > 0 && j_seen[i]) type_facts++;
    }
    if (a2_seen) type_facts++;
    size_t derived_facts = type_facts + (goal_seen ? 1U : 0U);
    bool count_ok = (type_facts == EXPECTED_TYPE_FACTS) && (derived_facts == EXPECTED_DERIVED_FACTS);

    printf("=== Answer ===\n");
    printf("The deep taxonomy chain reaches the goal from the seed fact after deriving the full class ladder up to N(100000).\n");
    printf("\n=== Reason Why ===\n");
    printf("Starting from Ind:N(0), each N(i) derives N(i+1), I(i+1), and J(i+1); N(100000) then derives A2 and the goal.\n");
    printf("seed facts    : 1\n");
    printf("rules         : %d\n", RULE_COUNT);
    printf("derived facts : %zu\n", derived_facts);
    printf("type facts    : %zu\n", type_facts);
    printf("\n=== Check ===\n");
    printf("goal reached  : %s\n", goal_seen ? "yes" : "no");
    printf("N(100000) seen: %s\n", n_seen[MAX_N] ? "yes" : "no");
    printf("A2 derived    : %s\n", a2_seen ? "yes" : "no");
    printf("count formula : %s\n", count_ok ? "yes" : "no");

    bool nmax_ok = n_seen[MAX_N];
    free(n_seen); free(i_seen); free(j_seen); free(q.data);
    return (goal_seen && nmax_ok && a2_seen && count_ok) ? 0 : 1;
}
