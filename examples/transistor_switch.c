
/*
 * transistor_switch.c
 *
 * A small NPN low-side switch model. With a low input, the transistor stays in
 * cutoff. With a high input, the collector current becomes load-limited and the
 * transistor reaches saturation.
 */

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

#define VCC_MV 5000U
#define VIN_LOW_MV 0U
#define VIN_HIGH_MV 5000U
#define VBE_ON_MV 700U
#define VCE_SAT_MV 200U
#define RB_OHMS 10000U
#define RL_OHMS 1000U
#define BETA 100U

typedef struct {
    uint32_t input_mv, base_current_ua, collector_gain_limit_ua, collector_load_limit_ua;
    uint32_t collector_current_ua, load_voltage_mv, collector_emitter_voltage_mv;
    bool cutoff, saturation;
} SwitchState;

static SwitchState evaluate_state(uint32_t input_mv) {
    if (input_mv <= VBE_ON_MV) {
        SwitchState s = {input_mv, 0, 0, 0, 0, 0, VCC_MV, true, false};
        return s;
    }
    uint32_t ib = ((input_mv - VBE_ON_MV) * 1000U) / RB_OHMS;
    uint32_t gain_limit = ib * BETA;
    uint32_t load_limit = ((VCC_MV - VCE_SAT_MV) * 1000U) / RL_OHMS;
    uint32_t ic = gain_limit < load_limit ? gain_limit : load_limit;
    bool sat = gain_limit >= load_limit;
    uint32_t load_v = (ic * RL_OHMS) / 1000U;
    uint32_t vce = sat ? VCE_SAT_MV : (VCC_MV - load_v);
    SwitchState s = {input_mv, ib, gain_limit, load_limit, ic, load_v, vce, false, sat};
    return s;
}

static const char *state_name(SwitchState s) {
    return s.cutoff ? "cutoff / OFF" : (s.saturation ? "saturation / ON" : "active / linear");
}

int main(void) {
    SwitchState low = evaluate_state(VIN_LOW_MV);
    SwitchState high = evaluate_state(VIN_HIGH_MV);
    bool low_cutoff = low.cutoff && low.collector_current_ua == 0 && low.collector_emitter_voltage_mv == VCC_MV;
    bool high_sat = high.saturation && high.collector_emitter_voltage_mv == VCE_SAT_MV;
    bool switching_cleanly = low.cutoff && high.saturation;
    bool load_limited = high.collector_current_ua == high.collector_load_limit_ua && high.collector_gain_limit_ua > high.collector_load_limit_ua;
    bool ohm_ok = high.load_voltage_mv == (high.collector_current_ua * RL_OHMS) / 1000U;

    printf("=== Answer ===\n");
    printf("In this toy transistor-switch model, a low input leaves the transistor OFF and a high input drives it ON in saturation.\n");
    printf("\n=== Reason Why ===\n");
    printf("low input state   : %s\n", state_name(low));
    printf("high input state  : %s\n", state_name(high));
    printf("high base current : %u uA\n", high.base_current_ua);
    printf("high collector Ic : %u uA\n", high.collector_current_ua);
    printf("load-limited Ic   : %u uA\n", high.collector_load_limit_ua);
    printf("\n=== Check ===\n");
    printf("low input cutoff                : %s\n", low_cutoff ? "yes" : "no");
    printf("high input saturation           : %s\n", high_sat ? "yes" : "no");
    printf("switching states differ         : %s\n", switching_cleanly ? "yes" : "no");
    printf("on-state current is load-limited: %s\n", load_limited ? "yes" : "no");
    printf("load voltage matches Ohm's law  : %s\n", ohm_ok ? "yes" : "no");
    return (low_cutoff && high_sat && switching_cleanly && load_limited && ohm_ok) ? 0 : 1;
}
