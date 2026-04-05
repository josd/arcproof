
/*
 * gps.c
 *
 * Standalone C port of the Rust GPS example.
 *
 * The program starts from four direct route descriptions and derives all
 * composed routes from Gent to Oostende that satisfy the original route
 * constraints. Duration and cost add across legs; belief and comfort combine
 * multiplicatively in parts-per-million.
 */

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef enum { GENT, BRUGGE, KORTRIJK, OOSTENDE } City;
typedef enum {
    DRIVE_GENT_BRUGGE,
    DRIVE_GENT_KORTRIJK,
    DRIVE_KORTRIJK_BRUGGE,
    DRIVE_BRUGGE_OOSTENDE
} Action;

typedef struct {
    City from;
    City to;
    Action action;
    uint32_t duration_seconds;
    uint32_t cost_milli;
    uint32_t belief_ppm;
    uint32_t comfort_ppm;
} Description;

typedef struct {
    uint32_t max_duration_seconds;
    uint32_t max_cost_milli;
    uint32_t min_belief_ppm;
    uint32_t min_comfort_ppm;
    size_t max_stages;
} Constraints;

typedef struct {
    City from;
    City to;
    Action actions[8];
    size_t actions_len;
    uint32_t duration_seconds;
    uint32_t cost_milli;
    uint32_t belief_ppm;
    uint32_t comfort_ppm;
} Route;

static const Description DESCRIPTIONS[] = {
    {GENT, BRUGGE, DRIVE_GENT_BRUGGE, 1500U, 6U, 960000U, 990000U},
    {GENT, KORTRIJK, DRIVE_GENT_KORTRIJK, 1600U, 7U, 960000U, 990000U},
    {KORTRIJK, BRUGGE, DRIVE_KORTRIJK_BRUGGE, 1600U, 7U, 960000U, 990000U},
    {BRUGGE, OOSTENDE, DRIVE_BRUGGE_OOSTENDE, 900U, 4U, 980000U, 1000000U},
};

static const Constraints GOAL = {5000U, 5000U, 200000U, 400000U, 1U};

static uint32_t multiply_ppm(uint32_t left, uint32_t right) {
    return (uint32_t)(((uint64_t)left * (uint64_t)right) / 1000000ULL);
}

static const char *action_name(Action action) {
    switch (action) {
        case DRIVE_GENT_BRUGGE: return "drive_gent_brugge";
        case DRIVE_GENT_KORTRIJK: return "drive_gent_kortrijk";
        case DRIVE_KORTRIJK_BRUGGE: return "drive_kortrijk_brugge";
        case DRIVE_BRUGGE_OOSTENDE: return "drive_brugge_oostende";
    }
    return "?";
}

static size_t stage_count(const Route *route) {
    return route->actions_len > 0U ? 1U : 0U;
}

static bool route_satisfies(const Route *route, const Constraints *c) {
    return route->duration_seconds <= c->max_duration_seconds &&
           route->cost_milli <= c->max_cost_milli &&
           route->belief_ppm >= c->min_belief_ppm &&
           route->comfort_ppm >= c->min_comfort_ppm &&
           stage_count(route) <= c->max_stages;
}

static bool route_equals(const Route *a, const Route *b) {
    return a->from == b->from &&
           a->to == b->to &&
           a->actions_len == b->actions_len &&
           a->duration_seconds == b->duration_seconds &&
           a->cost_milli == b->cost_milli &&
           a->belief_ppm == b->belief_ppm &&
           a->comfort_ppm == b->comfort_ppm &&
           memcmp(a->actions, b->actions, a->actions_len * sizeof(Action)) == 0;
}

static int compare_routes(const void *lhs, const void *rhs) {
    const Route *left = (const Route *)lhs;
    const Route *right = (const Route *)rhs;

    if (left->actions_len < right->actions_len) return -1;
    if (left->actions_len > right->actions_len) return 1;

    for (size_t i = 0; i < left->actions_len && i < right->actions_len; ++i) {
        if (left->actions[i] < right->actions[i]) return -1;
        if (left->actions[i] > right->actions[i]) return 1;
    }
    return 0;
}

static bool route_matches_descriptions(const Route *route) {
    City current = route->from;
    uint32_t duration_seconds = 0U;
    uint32_t cost_milli = 0U;
    uint32_t belief_ppm = 1000000U;
    uint32_t comfort_ppm = 1000000U;

    for (size_t i = 0; i < route->actions_len; ++i) {
        bool found = false;
        for (size_t j = 0; j < sizeof(DESCRIPTIONS) / sizeof(DESCRIPTIONS[0]); ++j) {
            const Description *d = &DESCRIPTIONS[j];
            if (d->from == current && d->action == route->actions[i]) {
                current = d->to;
                duration_seconds += d->duration_seconds;
                cost_milli += d->cost_milli;
                belief_ppm = multiply_ppm(belief_ppm, d->belief_ppm);
                comfort_ppm = multiply_ppm(comfort_ppm, d->comfort_ppm);
                found = true;
                break;
            }
        }
        if (!found) {
            return false;
        }
    }

    return current == route->to &&
           duration_seconds == route->duration_seconds &&
           cost_milli == route->cost_milli &&
           belief_ppm == route->belief_ppm &&
           comfort_ppm == route->comfort_ppm;
}

static size_t infer_goal_routes(Route *out_routes, size_t capacity) {
    Route known[64];
    size_t known_count = 0U;
    size_t agenda_head = 0U;

    for (size_t i = 0; i < sizeof(DESCRIPTIONS) / sizeof(DESCRIPTIONS[0]); ++i) {
        Route route;
        memset(&route, 0, sizeof(route));
        route.from = DESCRIPTIONS[i].from;
        route.to = DESCRIPTIONS[i].to;
        route.actions[0] = DESCRIPTIONS[i].action;
        route.actions_len = 1U;
        route.duration_seconds = DESCRIPTIONS[i].duration_seconds;
        route.cost_milli = DESCRIPTIONS[i].cost_milli;
        route.belief_ppm = DESCRIPTIONS[i].belief_ppm;
        route.comfort_ppm = DESCRIPTIONS[i].comfort_ppm;
        known[known_count++] = route;
    }

    while (agenda_head < known_count) {
        Route rest = known[agenda_head++];
        for (size_t i = 0; i < sizeof(DESCRIPTIONS) / sizeof(DESCRIPTIONS[0]); ++i) {
            const Description *d = &DESCRIPTIONS[i];
            if (d->to == rest.from) {
                Route route;
                memset(&route, 0, sizeof(route));
                route.from = d->from;
                route.to = rest.to;
                route.actions[0] = d->action;
                memcpy(&route.actions[1], rest.actions, rest.actions_len * sizeof(Action));
                route.actions_len = rest.actions_len + 1U;
                route.duration_seconds = d->duration_seconds + rest.duration_seconds;
                route.cost_milli = d->cost_milli + rest.cost_milli;
                route.belief_ppm = multiply_ppm(d->belief_ppm, rest.belief_ppm);
                route.comfort_ppm = multiply_ppm(d->comfort_ppm, rest.comfort_ppm);

                bool duplicate = false;
                for (size_t k = 0; k < known_count; ++k) {
                    if (route_equals(&known[k], &route)) {
                        duplicate = true;
                        break;
                    }
                }
                if (!duplicate) {
                    if (known_count >= sizeof(known) / sizeof(known[0])) {
                        fprintf(stderr, "gps: internal route buffer exhausted\n");
                        exit(1);
                    }
                    known[known_count++] = route;
                }
            }
        }
    }

    size_t out_count = 0U;
    for (size_t i = 0; i < known_count; ++i) {
        if (known[i].from == GENT && known[i].to == OOSTENDE && route_satisfies(&known[i], &GOAL)) {
            if (out_count >= capacity) {
                fprintf(stderr, "gps: output route buffer exhausted\n");
                exit(1);
            }
            out_routes[out_count++] = known[i];
        }
    }

    qsort(out_routes, out_count, sizeof(Route), compare_routes);
    return out_count;
}

static void format_decimal(char *buffer, size_t buffer_size, uint64_t value, uint64_t scale, unsigned digits) {
    uint64_t fractional_scale = 1U;
    for (unsigned i = 0; i < digits; ++i) {
        fractional_scale *= 10U;
    }

    uint64_t scaled = value * fractional_scale;
    uint64_t rounded = (scaled + (scale / 2U)) / scale;
    uint64_t whole = rounded / fractional_scale;
    uint64_t fractional = rounded % fractional_scale;

    (void)snprintf(buffer, buffer_size, "%llu.%0*llu",
                   (unsigned long long)whole,
                   (int)digits,
                   (unsigned long long)fractional);
}

static void print_route(size_t index, const Route *route) {
    char cost[32];
    char cost_limit[32];
    char belief[32];
    char belief_limit[32];
    char comfort[32];
    char comfort_limit[32];

    format_decimal(cost, sizeof(cost), route->cost_milli, 1000U, 3U);
    format_decimal(cost_limit, sizeof(cost_limit), GOAL.max_cost_milli, 1000U, 1U);
    format_decimal(belief, sizeof(belief), route->belief_ppm, 1000000U, 3U);
    format_decimal(belief_limit, sizeof(belief_limit), GOAL.min_belief_ppm, 1000000U, 1U);
    format_decimal(comfort, sizeof(comfort), route->comfort_ppm, 1000000U, 3U);
    format_decimal(comfort_limit, sizeof(comfort_limit), GOAL.min_comfort_ppm, 1000000U, 1U);

    printf("Route #%zu\n", index);
    printf(" Steps    : %zu\n", route->actions_len);
    printf(" Duration : %u s (\342\211\244 %u)\n", route->duration_seconds, GOAL.max_duration_seconds);
    printf(" Cost     : %s (\342\211\244 %s)\n", cost, cost_limit);
    printf(" Belief   : %s (\342\211\245 %s)\n", belief, belief_limit);
    printf(" Comfort  : %s (\342\211\245 %s)\n", comfort, comfort_limit);
    printf(" Stages   : %zu (\342\211\244 %zu)\n", stage_count(route), GOAL.max_stages);
    for (size_t i = 0; i < route->actions_len; ++i) {
        printf("   %zu. %s\n", i + 1U, action_name(route->actions[i]));
    }
}

int main(void) {
    Route routes[16];
    size_t route_count = infer_goal_routes(routes, sizeof(routes) / sizeof(routes[0]));

    bool all_routes_satisfy_constraints = true;
    bool all_routes_hit_goal_endpoints = true;
    bool all_metrics_recompute = true;
    for (size_t i = 0; i < route_count; ++i) {
        all_routes_satisfy_constraints = all_routes_satisfy_constraints && route_satisfies(&routes[i], &GOAL);
        all_routes_hit_goal_endpoints = all_routes_hit_goal_endpoints && routes[i].from == GENT && routes[i].to == OOSTENDE;
        all_metrics_recompute = all_metrics_recompute && route_matches_descriptions(&routes[i]);
    }

    printf("=== Answer ===\n");
    printf("The GPS case finds all goal routes from Gent to Oostende that satisfy the route constraints.\n");
    printf("case      : gps\n");
    printf("routes    : %zu\n", route_count);
    printf("\n=== Reason Why ===\n");
    printf("Routes are built compositionally from direct descriptions, with duration and cost added and belief and comfort combined multiplicatively.\n");
    for (size_t i = 0; i < route_count; ++i) {
        print_route(i + 1U, &routes[i]);
        if (i + 1U != route_count) {
            printf("\n");
        }
    }

    printf("\n=== Check ===\n");
    printf("all routes satisfy constraints : %s\n", all_routes_satisfy_constraints ? "yes" : "no");
    printf("all routes hit goal endpoints  : %s\n", all_routes_hit_goal_endpoints ? "yes" : "no");
    printf("metrics recompute from steps   : %s\n", all_metrics_recompute ? "yes" : "no");
    printf("expected route count (= 2)     : %s\n", route_count == 2U ? "yes" : "no");

    return (route_count == 2U && all_routes_satisfy_constraints && all_routes_hit_goal_endpoints && all_metrics_recompute) ? 0 : 1;
}
