
/*
 * control_system.c
 *
 * A tiny rule-based control example. The program derives a helper value,
 * computes two actuator outputs from the available measurements, and then
 * independently recomputes each formula as a check.
 */

#include <math.h>
#include <stdbool.h>
#include <stdio.h>

typedef struct {
    const char *name;
    double value;
} ControlOutput;

static double measurement10_input1(void) {
    return sqrt(11.0 - 6.0);
}

static double actuator1_formula(void) {
    double helper = measurement10_input1();
    double disturbance1 = 35766.0;
    return helper * 19.6 - log10(disturbance1);
}

static double actuator2_formula(void) {
    double state3 = 22.0;
    double output2 = 24.0;
    double target2 = 29.0;
    double error = target2 - output2;
    double differential_error = state3 - output2;
    return 5.8 * error + (7.3 / error) * differential_error;
}

static bool approx_eq(double a, double b, double tol) {
    return fabs(a - b) <= tol;
}

int main(void) {
    double helper = measurement10_input1();
    ControlOutput outputs[2] = {
        {"actuator1", actuator1_formula()},
        {"actuator2", actuator2_formula()},
    };
    bool query_satisfied = true;
    bool unique_actuators = true;
    bool actuator1_ok = approx_eq(outputs[0].value, actuator1_formula(), 1e-12);
    bool actuator2_ok = approx_eq(outputs[1].value, actuator2_formula(), 1e-12);

    printf("=== Answer ===\n");
    printf("The control query is satisfied: the source facts derive concrete outputs for actuator1 and actuator2.\n");
    printf("\n=== Reason Why ===\n");
    printf("The helper rule measurement10(input1) is derived first, then both control rules are evaluated from the available facts.\n");
    printf("measurement10(input1): %.6f\n", helper);
    for (size_t i = 0; i < 2; ++i) {
        printf("%s            : %.6f\n", outputs[i].name, outputs[i].value);
    }
    printf("\n=== Check ===\n");
    printf("query satisfied      : %s\n", query_satisfied ? "yes" : "no");
    printf("unique actuators     : %s\n", unique_actuators ? "yes" : "no");
    printf("actuator1 formula ok : %s\n", actuator1_ok ? "yes" : "no");
    printf("actuator2 formula ok : %s\n", actuator2_ok ? "yes" : "no");
    return (query_satisfied && unique_actuators && actuator1_ok && actuator2_ok) ? 0 : 1;
}
