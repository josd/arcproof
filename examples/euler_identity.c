
/*
 * euler_identity.c
 *
 * This toy program represents the exact value exp(i*pi) as (-1, 0), then adds
 * 1 to obtain (0, 0). The point is not numerical approximation, but showing
 * the exact algebraic structure of Euler's identity in a tiny C program.
 */

#include <stdbool.h>
#include <stdio.h>

typedef struct {
    int re;
    int im;
} ExactComplex;

static ExactComplex add(ExactComplex a, ExactComplex b) {
    ExactComplex c = {a.re + b.re, a.im + b.im};
    return c;
}

int main(void) {
    ExactComplex exp_ipi = {-1, 0};
    ExactComplex one = {1, 0};
    ExactComplex result = add(exp_ipi, one);
    int modulus_sq = exp_ipi.re * exp_ipi.re + exp_ipi.im * exp_ipi.im;
    bool identity_ok = (result.re == 0 && result.im == 0);
    bool unit_circle_ok = (modulus_sq == 1);

    printf("=== Answer ===\n");
    printf("Euler's identity holds exactly in this exact-arithmetic model: exp(i*pi) + 1 = 0.\n");
    printf("\n=== Reason Why ===\n");
    printf("exp(i*pi) is represented as (-1, 0) and adding (1, 0) gives the exact zero complex number.\n");
    printf("exp(i*pi)   : (%d, %d)\n", exp_ipi.re, exp_ipi.im);
    printf("exp(i*pi)+1 : (%d, %d)\n", result.re, result.im);
    printf("|exp(i*pi)|^2: %d\n", modulus_sq);
    printf("\n=== Check ===\n");
    printf("identity exact: %s\n", identity_ok ? "yes" : "no");
    printf("unit circle   : %s\n", unit_circle_ok ? "yes" : "no");
    return (identity_ok && unit_circle_ok) ? 0 : 1;
}
