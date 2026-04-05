
/*
 * fibonacci.c
 *
 * This standalone program computes exact Fibonacci numbers with GMP. It uses a
 * simple iterative recurrence for the main values and a fast-doubling helper as
 * an independent cross-check.
 */

#include <gmp.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void fibonacci_iterative(size_t n, mpz_t out) {
    mpz_t a, b, t;
    mpz_inits(a, b, t, NULL);
    mpz_set_ui(a, 0);
    mpz_set_ui(b, 1);
    for (size_t i = 0; i < n; ++i) {
        mpz_add(t, a, b);
        mpz_set(a, b);
        mpz_set(b, t);
    }
    mpz_set(out, a);
    mpz_clears(a, b, t, NULL);
}

static void fast_doubling(size_t n, mpz_t fn, mpz_t fn1) {
    if (n == 0) {
        mpz_set_ui(fn, 0);
        mpz_set_ui(fn1, 1);
        return;
    }
    mpz_t a, b, c, d, two_b_minus_a;
    mpz_inits(a, b, c, d, two_b_minus_a, NULL);
    fast_doubling(n / 2, a, b);
    mpz_mul_ui(two_b_minus_a, b, 2);
    mpz_sub(two_b_minus_a, two_b_minus_a, a);
    mpz_mul(c, a, two_b_minus_a);               /* F(2k) */
    mpz_mul(d, a, a);
    mpz_addmul(d, b, b);                        /* F(2k+1) */
    if (n % 2 == 0) {
        mpz_set(fn, c);
        mpz_set(fn1, d);
    } else {
        mpz_set(fn, d);
        mpz_add(fn1, c, d);
    }
    mpz_clears(a, b, c, d, two_b_minus_a, NULL);
}

int main(void) {
    const size_t targets[5] = {0, 1, 10, 100, 1000};
    mpz_t vals[5];
    for (int i = 0; i < 5; ++i) { mpz_init(vals[i]); fibonacci_iterative(targets[i], vals[i]); }
    bool f10_ok = mpz_cmp_ui(vals[2], 55) == 0;
    char *f1000_str = mpz_get_str(NULL, 10, vals[4]);
    size_t f1000_digits = strlen(f1000_str);

    bool fast_ok = true;
    for (int i = 0; i < 5; ++i) {
        mpz_t a, b;
        mpz_inits(a, b, NULL);
        fast_doubling(targets[i], a, b);
        if (mpz_cmp(a, vals[i]) != 0) fast_ok = false;
        mpz_clears(a, b, NULL);
    }

    mpz_t f99, f100, f101, left, right;
    mpz_inits(f99, f100, f101, left, right, NULL);
    fibonacci_iterative(99, f99);
    fibonacci_iterative(100, f100);
    fibonacci_iterative(101, f101);
    mpz_mul(left, f101, f99);
    mpz_mul(right, f100, f100);
    mpz_add_ui(right, right, 1);
    bool cassini_ok = mpz_cmp(left, right) == 0;

    printf("=== Answer ===\n");
    printf("The requested Fibonacci values are computed exactly, up to F(1000).\n");
    printf("\n=== Reason Why ===\n");
    printf("The main computation uses the defining recurrence F(n+1)=F(n)+F(n-1), and the results are cross-checked with fast doubling.\n");
    for (int i = 0; i < 5; ++i) {
        char *s = mpz_get_str(NULL, 10, vals[i]);
        printf("value[%zu]          : F(%zu) = %s\n", (size_t)i, targets[i], s);
        free(s);
    }
    printf("digits in F(1000)   : %zu\n", f1000_digits);
    printf("\n=== Check ===\n");
    printf("F(10) = 55          : %s\n", f10_ok ? "yes" : "no");
    printf("fast doubling agrees: %s\n", fast_ok ? "yes" : "no");
    printf("Cassini at n=100    : %s\n", cassini_ok ? "yes" : "no");
    printf("F(1000) has 209 digits: %s\n", (f1000_digits == 209) ? "yes" : "no");

    free(f1000_str);
    mpz_clears(f99, f100, f101, left, right, NULL);
    for (int i = 0; i < 5; ++i) mpz_clear(vals[i]);
    return (f10_ok && fast_ok && cassini_ok && f1000_digits == 209) ? 0 : 1;
}
