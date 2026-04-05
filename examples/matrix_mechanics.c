
/*
 * matrix_mechanics.c
 *
 * A tiny exact 2x2 matrix example inspired by early quantum mechanics. It
 * computes HX, XH, and the commutator [H,X] to show that matrix order matters.
 */

#include <stdbool.h>
#include <stdio.h>

typedef struct { int a11, a12, a21, a22; } M2;

static M2 m2(int a11, int a12, int a21, int a22) { M2 m = {a11, a12, a21, a22}; return m; }
static M2 mul(M2 a, M2 b) { return m2(a.a11*b.a11 + a.a12*b.a21, a.a11*b.a12 + a.a12*b.a22, a.a21*b.a11 + a.a22*b.a21, a.a21*b.a12 + a.a22*b.a22); }
static M2 sub(M2 a, M2 b) { return m2(a.a11-b.a11, a.a12-b.a12, a.a21-b.a21, a.a22-b.a22); }
static int trace(M2 a) { return a.a11 + a.a22; }
static int det(M2 a) { return a.a11 * a.a22 - a.a12 * a.a21; }

int main(void) {
    M2 H = m2(1, 0, 0, 2);
    M2 X = m2(0, 1, 1, 0);
    M2 HX = mul(H, X);
    M2 XH = mul(X, H);
    M2 C = sub(HX, XH);
    bool commutator_nonzero = (C.a11 || C.a12 || C.a21 || C.a22);
    bool spectrum_ok = (trace(H) == 3 && det(H) == 2);
    M2 XX = mul(X, X);
    bool involution = (XX.a11 == 1 && XX.a12 == 0 && XX.a21 == 0 && XX.a22 == 1);

    printf("=== Answer ===\n");
    printf("In this toy matrix-mechanics model, the Hamiltonian has two discrete energy levels and does not commute with a second observable.\n");
    printf("\n=== Reason Why ===\n");
    printf("H  = [[%d,%d],[%d,%d]]\n", H.a11, H.a12, H.a21, H.a22);
    printf("X  = [[%d,%d],[%d,%d]]\n", X.a11, X.a12, X.a21, X.a22);
    printf("HX = [[%d,%d],[%d,%d]]\n", HX.a11, HX.a12, HX.a21, HX.a22);
    printf("XH = [[%d,%d],[%d,%d]]\n", XH.a11, XH.a12, XH.a21, XH.a22);
    printf("[H,X] = [[%d,%d],[%d,%d]]\n", C.a11, C.a12, C.a21, C.a22);
    printf("\n=== Check ===\n");
    printf("trace/determinant match energy levels: %s\n", spectrum_ok ? "yes" : "no");
    printf("X^2 = I                           : %s\n", involution ? "yes" : "no");
    printf("[H,X] != 0                        : %s\n", commutator_nonzero ? "yes" : "no");
    return (spectrum_ok && involution && commutator_nonzero) ? 0 : 1;
}
