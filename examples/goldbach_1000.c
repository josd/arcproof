
/*
 * goldbach_1000.c
 *
 * The program generates primes up to 1000 with a sieve, checks every even
 * target from 4 through 1000, counts the unordered decompositions p+q=n, and
 * summarizes the hardest and richest cases.
 */

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

#define LIMIT 1000

static bool *sieve(size_t limit) {
    bool *prime = (bool *)malloc((limit + 1) * sizeof(bool));
    if (!prime) return NULL;
    for (size_t i = 0; i <= limit; ++i) prime[i] = true;
    prime[0] = prime[1] = false;
    for (size_t p = 2; p * p <= limit; ++p) {
        if (!prime[p]) continue;
        for (size_t m = p * p; m <= limit; m += p) prime[m] = false;
    }
    return prime;
}

static size_t collect_primes(const bool *prime, size_t limit, size_t **out) {
    size_t count = 0;
    for (size_t i = 2; i <= limit; ++i) if (prime[i]) count++;
    size_t *vals = (size_t *)malloc(count * sizeof(size_t));
    if (!vals) return 0;
    size_t k = 0;
    for (size_t i = 2; i <= limit; ++i) if (prime[i]) vals[k++] = i;
    *out = vals;
    return count;
}

static size_t goldbach_pairs(size_t target, const size_t *primes, size_t prime_count, const bool *prime) {
    size_t count = 0;
    for (size_t i = 0; i < prime_count; ++i) {
        size_t p = primes[i];
        if (p > target / 2) break;
        size_t q = target - p;
        if (prime[q]) count++;
    }
    return count;
}

int main(void) {
    bool *prime = sieve(LIMIT);
    size_t *primes = NULL;
    if (!prime) return 1;
    size_t prime_count = collect_primes(prime, LIMIT, &primes);
    if (!primes) return 1;

    size_t total_decompositions = 0;
    size_t fewest = (size_t)-1, most = 0, richest_target = 4;
    size_t hardest[512];
    size_t hardest_len = 0;
    bool all_represented = true;

    for (size_t target = 4; target <= LIMIT; target += 2) {
        size_t count = goldbach_pairs(target, primes, prime_count, prime);
        total_decompositions += count;
        if (count == 0) all_represented = false;
        if (count < fewest) {
            fewest = count;
            hardest_len = 0;
            hardest[hardest_len++] = target;
        } else if (count == fewest) {
            hardest[hardest_len++] = target;
        }
        if (count > most) {
            most = count;
            richest_target = target;
        }
    }

    size_t best_a = 0, best_b = 0, best_diff = (size_t)-1;
    for (size_t i = 0; i < prime_count; ++i) {
        size_t p = primes[i];
        if (p > LIMIT / 2) break;
        size_t q = LIMIT - p;
        if (prime[q] && q >= p) {
            size_t diff = q - p;
            if (diff < best_diff) { best_diff = diff; best_a = p; best_b = q; }
        }
    }

    bool prime_count_ok = prime_count == 168;
    bool balanced_pair_ok = (best_a + best_b == LIMIT && prime[best_a] && prime[best_b]);

    printf("=== Answer ===\n");
    printf("Every even integer from 4 through 1000 has at least one Goldbach decomposition in the tested range.\n");
    printf("\n=== Reason Why ===\n");
    printf("The program builds a prime table, enumerates unordered pairs p+q=n for each even target, and summarizes sparse and rich cases.\n");
    printf("even targets checked : %d\n", (LIMIT - 4) / 2 + 1);
    printf("total decompositions : %zu\n", total_decompositions);
    printf("fewest decompositions: %zu\n", fewest);
    printf("hardest targets      : ");
    for (size_t i = 0; i < hardest_len; ++i) {
        if (i) printf(", ");
        printf("%zu", hardest[i]);
    }
    printf("\n");
    printf("most decompositions  : %zu\n", most);
    printf("richest target       : %zu\n", richest_target);
    printf("balanced pair(1000)  : %zu + %zu\n", best_a, best_b);
    printf("\n=== Check ===\n");
    printf("all represented      : %s\n", all_represented ? "yes" : "no");
    printf("prime count known    : %s\n", prime_count_ok ? "yes" : "no");
    printf("balanced pair valid  : %s\n", balanced_pair_ok ? "yes" : "no");

    free(prime);
    free(primes);
    return (all_represented && prime_count_ok && balanced_pair_ok) ? 0 : 1;
}
