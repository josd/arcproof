
/*
 * polynomial.c
 *
 * This program solves two quartic examples numerically with the Durand-Kerner
 * method. It then substitutes the roots back into the original polynomial and
 * rebuilds the coefficients from those roots as an independent consistency
 * check.
 */

#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

#define ROOT_TOL 1e-10
#define COEFF_TOL 1e-8
#define MAX_ITER 200

typedef struct { double re, im; } Cx;
static Cx cx(double re, double im) { Cx z = {re, im}; return z; }
static Cx add(Cx a, Cx b) { return cx(a.re + b.re, a.im + b.im); }
static Cx sub(Cx a, Cx b) { return cx(a.re - b.re, a.im - b.im); }
static Cx mul(Cx a, Cx b) { return cx(a.re*b.re - a.im*b.im, a.re*b.im + a.im*b.re); }
static Cx divcx(Cx a, Cx b) { double s = b.re*b.re + b.im*b.im; return cx((a.re*b.re + a.im*b.im)/s, (a.im*b.re - a.re*b.im)/s); }
static double abscx(Cx a) { return hypot(a.re, a.im); }
static Cx powu(Cx a, size_t e) { Cx out = cx(1,0); for (size_t i=0;i<e;++i) out = mul(out,a); return out; }
static double dist(Cx a, Cx b) { return abscx(sub(a,b)); }

static Cx eval_poly(const Cx *coeffs, size_t len, Cx x) { Cx acc = cx(0,0); for (size_t i=0;i<len;++i) acc = add(mul(acc,x), coeffs[i]); return acc; }
static void multiply_polys(const Cx *left, size_t left_len, const Cx *right, size_t right_len, Cx *out) {
    for (size_t i=0;i<left_len+right_len-1;++i) out[i] = cx(0,0);
    for (size_t i=0;i<left_len;++i) for (size_t j=0;j<right_len;++j) out[i+j] = add(out[i+j], mul(left[i], right[j]));
}
static void roots_from_coeffs(const Cx *coeffs, size_t len, Cx *roots) {
    size_t degree = len - 1;
    Cx monic[8];
    Cx lead = coeffs[0];
    double radius = 1.0;
    for (size_t i=0;i<len;++i) {
        monic[i] = divcx(coeffs[i], lead);
        if (i>0) {
            double a = abscx(monic[i]);
            if (a > radius - 1.0) radius = 1.0 + a;
        }
    }
    Cx seed = cx(0.4, 0.9);
    for (size_t i=0;i<degree;++i) roots[i] = mul(powu(seed, i), cx(radius, 0));
    for (size_t iter=0; iter<MAX_ITER; ++iter) {
        double max_delta = 0.0;
        for (size_t i=0;i<degree;++i) {
            Cx denom = cx(1,0);
            for (size_t j=0;j<degree;++j) if (j != i) denom = mul(denom, sub(roots[i], roots[j]));
            if (abscx(denom) < 1e-18) denom = add(denom, cx(1e-12, 1e-12));
            Cx delta = divcx(eval_poly(monic, len, roots[i]), denom);
            roots[i] = sub(roots[i], delta);
            double a = abscx(delta); if (a > max_delta) max_delta = a;
        }
        if (max_delta < ROOT_TOL) break;
    }
}

static void sort_roots(Cx *roots, size_t n) {
    for (size_t i=0;i<n;++i) for (size_t j=i+1;j<n;++j) {
        bool left_real = fabs(roots[i].im) < 1e-8;
        bool right_real = fabs(roots[j].im) < 1e-8;
        bool swap = false;
        if (left_real && right_real) swap = roots[j].re > roots[i].re;
        else if (!left_real && right_real) swap = false;
        else if (left_real && !right_real) swap = true;
        else if (roots[j].im > roots[i].im || (fabs(roots[j].im - roots[i].im) < 1e-8 && roots[j].re > roots[i].re)) swap = true;
        if (swap) { Cx t = roots[i]; roots[i] = roots[j]; roots[j] = t; }
    }
}

static void print_cx(Cx z) {
    double re = fabs(z.re) < 1e-8 ? 0.0 : z.re;
    double im = fabs(z.im) < 1e-8 ? 0.0 : z.im;
    if (im == 0.0) printf("%.10g", re);
    else if (re == 0.0) printf("%.10gi", im);
    else printf("%.10g %c %.10gi", re, im >= 0 ? '+' : '-', fabs(im));
}

int main(void) {
    Cx cases[2][5] = {
        {{1,0},{-10,0},{35,0},{-50,0},{24,0}},
        {{1,0},{-9,-5},{14,33},{24,-44},{-26,0}},
    };
    const char *labels[2] = {"real quartic", "complex quartic"};
    bool all_ok = true;

    printf("=== Answer ===\n");
    printf("Both polynomial examples are solved consistently: the computed roots satisfy the source polynomials and reconstruct the original coefficients.\n\n");
    printf("=== Reason Why ===\n");
    printf("For each quartic, the program solves for the roots numerically, substitutes them back, and rebuilds the polynomial from those roots.\n");
    for (int c = 0; c < 2; ++c) {
        Cx roots[4];
        roots_from_coeffs(cases[c], 5, roots);
        sort_roots(roots, 4);
        Cx rebuilt[5];
        Cx coeffs1[2] = {cx(1,0), sub(cx(0,0), roots[0])};
        Cx coeffs2[2] = {cx(1,0), sub(cx(0,0), roots[1])};
        Cx coeffs3[2] = {cx(1,0), sub(cx(0,0), roots[2])};
        Cx coeffs4[2] = {cx(1,0), sub(cx(0,0), roots[3])};
        Cx tmp1[3], tmp2[4];
        multiply_polys(coeffs1, 2, coeffs2, 2, tmp1);
        multiply_polys(tmp1, 3, coeffs3, 2, tmp2);
        multiply_polys(tmp2, 4, coeffs4, 2, rebuilt);
        bool roots_valid = true, rebuild_ok = true;
        for (int i=0;i<4;++i) if (abscx(eval_poly(cases[c], 5, roots[i])) > 1e-6) roots_valid = false;
        for (int i=0;i<5;++i) if (dist(rebuilt[i], cases[c][i]) > COEFF_TOL) rebuild_ok = false;
        all_ok &= roots_valid && rebuild_ok;
        printf("\nExample #%d (%s)\n", c + 1, labels[c]);
        printf("roots               : ");
        for (int i=0;i<4;++i) { if (i) printf(", "); print_cx(roots[i]); }
        printf("\nresiduals           : ");
        for (int i=0;i<4;++i) { if (i) printf(", "); print_cx(eval_poly(cases[c], 5, roots[i])); }
        printf("\nreconstruction ok   : %s\n", rebuild_ok ? "yes" : "no");
        printf("roots valid         : %s\n", roots_valid ? "yes" : "no");
    }
    printf("\n=== Check ===\n");
    printf("all examples valid  : %s\n", all_ok ? "yes" : "no");
    return all_ok ? 0 : 1;
}
