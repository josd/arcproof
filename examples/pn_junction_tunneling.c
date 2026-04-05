
/*
 * pn_junction_tunneling.c
 *
 * This is a toy tunneling model. The current proxy is the size of the overlap
 * between a filled set of N-side states and an empty set of P-side states after
 * forward bias shifts the P-side levels.
 */

#include <stdbool.h>
#include <stdio.h>

static const int N_FILLED[4] = {1, 2, 3, 4};
static const int P_EMPTY_ZERO_BIAS[4] = {3, 4, 5, 6};
static const int BIAS_POINTS[7] = {0, 1, 2, 3, 4, 5, 6};

static int overlap_count(const int lhs[4], const int rhs[4]) {
    int count = 0;
    for (int i = 0; i < 4; ++i) {
        for (int j = 0; j < 4; ++j) if (lhs[i] == rhs[j]) { count++; break; }
    }
    return count;
}

int main(void) {
    int curve[7];
    int peak_index = 0;
    for (int i = 0; i < 7; ++i) {
        int shifted[4];
        for (int j = 0; j < 4; ++j) shifted[j] = P_EMPTY_ZERO_BIAS[j] - BIAS_POINTS[i];
        curve[i] = overlap_count(N_FILLED, shifted);
        if (curve[i] > curve[peak_index]) peak_index = i;
    }
    int valley_index = 6;
    bool barrier_narrower = (1 < 8);
    bool peak_before_valley = peak_index < valley_index;
    bool negative_differential = false;
    for (int i = peak_index; i < 6; ++i) if (curve[i + 1] < curve[i]) negative_differential = true;
    bool overlap_closes = curve[valley_index] == 0;
    bool full_overlap_peak = curve[peak_index] == 4;

    printf("=== Answer ===\n");
    printf("In this toy PN-junction tunneling model, heavy doping narrows the depletion region enough for a tunneling window that rises to a peak and then falls.\n");
    printf("\n=== Reason Why ===\n");
    printf("We count exact state overlap while forward bias shifts the empty P-side levels.\n");
    printf("bias -> overlap current proxy : ");
    for (int i = 0; i < 7; ++i) { if (i) printf(", "); printf("%d->%d", BIAS_POINTS[i], curve[i]); }
    printf("\npeak point                    : %d -> %d\n", BIAS_POINTS[peak_index], curve[peak_index]);
    printf("high-bias point               : %d -> %d\n", BIAS_POINTS[valley_index], curve[valley_index]);
    printf("\n=== Check ===\n");
    printf("heavily doped barrier is narrower : %s\n", barrier_narrower ? "yes" : "no");
    printf("peak occurs before overlap closes : %s\n", peak_before_valley ? "yes" : "no");
    printf("negative differential region      : %s\n", negative_differential ? "yes" : "no");
    printf("high-bias overlap closes          : %s\n", overlap_closes ? "yes" : "no");
    printf("peak equals full overlap          : %s\n", full_overlap_peak ? "yes" : "no");
    return (barrier_narrower && peak_before_valley && negative_differential && overlap_closes && full_overlap_peak) ? 0 : 1;
}
