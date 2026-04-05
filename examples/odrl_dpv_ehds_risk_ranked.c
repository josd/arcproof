
/*
 * odrl_dpv_ehds_risk_ranked.c
 *
 * This file turns a small EHDS agreement into concrete permissions, patient
 * needs, and risk rules. A risk is emitted when a permission is missing a
 * safeguard that a rule expects.
 */

#include <stdbool.h>
#include <stdio.h>
#include <string.h>

typedef enum { PROVIDE_SECONDARY_USE_DATA, DOWNLOAD, REMOVE_DIRECT_IDENTIFIERS, PROCESS_ONLY_IN_SECURE_ENVIRONMENT } Action;
typedef enum { PURPOSE, HAS_DATA_PERMIT, RESPECT_OPT_OUT_SECONDARY_USE, STATISTICALLY_ANONYMISED } ConstraintKey;
typedef struct { ConstraintKey key; const char *value; } Constraint;
typedef struct { Action action; } Duty;
typedef struct { const char *id; unsigned importance; const char *description; } Need;
typedef struct { const char *id; const char *text; } Clause;
typedef struct { const char *id; const char *clause_id; Action action; const Constraint *constraints; size_t constraints_len; const Duty *duties; size_t duties_len; } Permission;
typedef enum { MISSING_DATA_PERMIT, MISSING_OPT_OUT, MISSING_SECURE_ENV, MISSING_STAT_ANON } MissingSafeguard;
typedef struct { const char *rule_id; const char *permission_id; const char *clause_id; const char *need_id; unsigned base_score; const char *risk_source; const char *mitigation; MissingSafeguard missing; } RiskRule;
typedef struct { const char *clause_id; const char *permission_id; const char *need_id; Action action; unsigned need_importance; unsigned score_raw; unsigned score; const char *risk_source; const char *mitigation; } RankedRisk;

static const char *action_name(Action a) {
    switch (a) {
        case PROVIDE_SECONDARY_USE_DATA: return "provideSecondaryUseData";
        case DOWNLOAD: return "download";
        case REMOVE_DIRECT_IDENTIFIERS: return "removeDirectIdentifiers";
        case PROCESS_ONLY_IN_SECURE_ENVIRONMENT: return "processOnlyInSecureEnvironment";
    }
    return "?";
}

static const Need NEEDS[] = {
    {"Need_RequireDataPermit", 20, "Secondary use should be authorised via an EHDS Data Permit."},
    {"Need_RespectOptOutSecondaryUse", 25, "Respect the EHDS right to opt out from secondary use."},
    {"Need_SecureProcessingEnvironment", 18, "Secondary-use processing must occur within a secure processing environment."},
    {"Need_StatisticallyAnonymisedSecondaryUse", 15, "Secondary use should use statistically anonymised data."},
};
static const Constraint P1_C[] = {{PURPOSE, "HealthcareScientificResearch"}};
static const Constraint P2_C[] = {{PURPOSE, "TrainTestAndEvaluateHealthAlgorithms"}};
static const Duty P4_D[] = {{REMOVE_DIRECT_IDENTIFIERS}};
static const Permission PERMISSIONS[] = {
    {"PermSecondaryUseDUA", "H1", PROVIDE_SECONDARY_USE_DATA, P1_C, 1, NULL, 0},
    {"PermSecondaryUseAllPatients", "H2", PROVIDE_SECONDARY_USE_DATA, P2_C, 1, NULL, 0},
    {"PermDownloadLocalCopy", "H3", DOWNLOAD, NULL, 0, NULL, 0},
    {"PermProvidePseudonymisedData", "H4", PROVIDE_SECONDARY_USE_DATA, NULL, 0, P4_D, 1},
};
static const RiskRule RULES[] = {
    {"R1", "PermSecondaryUseDUA", "H1", "Need_RequireDataPermit", 80, "Secondary use permitted without EHDS Data Permit.", "Require an EHDS Data Permit before secondary use.", MISSING_DATA_PERMIT},
    {"R2", "PermSecondaryUseAllPatients", "H2", "Need_RespectOptOutSecondaryUse", 75, "Opt-out from secondary use not explicitly respected.", "Exclude records of persons who exercised the EHDS opt-out.", MISSING_OPT_OUT},
    {"R3", "PermDownloadLocalCopy", "H3", "Need_SecureProcessingEnvironment", 70, "Local download permitted; secure processing environment not required.", "Require processing only within a secure processing environment.", MISSING_SECURE_ENV},
    {"R4", "PermProvidePseudonymisedData", "H4", "Need_StatisticallyAnonymisedSecondaryUse", 65, "Statistical anonymisation safeguard missing for secondary use.", "Require statistically anonymised data for secondary use.", MISSING_STAT_ANON},
};

static const Need *find_need(const char *id) { for (size_t i=0;i<4;++i) if (strcmp(NEEDS[i].id,id)==0) return &NEEDS[i]; return NULL; }
static const Permission *find_perm(const char *id) { for (size_t i=0;i<4;++i) if (strcmp(PERMISSIONS[i].id,id)==0) return &PERMISSIONS[i]; return NULL; }
static bool has_constraint(const Permission *p, ConstraintKey k) { for (size_t i=0;i<p->constraints_len;++i) if (p->constraints[i].key == k) return true; return false; }
static bool has_duty(const Permission *p, Action a) { for (size_t i=0;i<p->duties_len;++i) if (p->duties[i].action == a) return true; return false; }
static bool missing(const Permission *p, MissingSafeguard m) {
    switch (m) {
        case MISSING_DATA_PERMIT: return !has_constraint(p, HAS_DATA_PERMIT);
        case MISSING_OPT_OUT: return !has_constraint(p, RESPECT_OPT_OUT_SECONDARY_USE);
        case MISSING_SECURE_ENV: return !has_duty(p, PROCESS_ONLY_IN_SECURE_ENVIRONMENT);
        case MISSING_STAT_ANON: return !has_constraint(p, STATISTICALLY_ANONYMISED);
    }
    return false;
}
static void sort_risks(RankedRisk *r, size_t n) {
    for (size_t i=0;i<n;++i) for (size_t j=i+1;j<n;++j) {
        if (r[j].score > r[i].score || (r[j].score == r[i].score && strcmp(r[j].clause_id, r[i].clause_id) < 0)) {
            RankedRisk t = r[i]; r[i] = r[j]; r[j] = t;
        }
    }
}

int main(void) {
    RankedRisk risks[8];
    size_t risk_len = 0;
    for (size_t i=0;i<4;++i) {
        const RiskRule *rule = &RULES[i];
        const Permission *p = find_perm(rule->permission_id);
        const Need *need = find_need(rule->need_id);
        if (p && need && missing(p, rule->missing)) {
            unsigned raw = rule->base_score + need->importance;
            unsigned score = raw > 100 ? 100 : raw;
            risks[risk_len++] = (RankedRisk){rule->clause_id, rule->permission_id, rule->need_id, p->action, need->importance, raw, score, rule->risk_source, rule->mitigation};
        }
    }
    sort_risks(risks, risk_len);
    bool score_formula_ok = true, sorted_ok = true, top_pair_ok = false, mitigations_ok = true;
    for (size_t i=0;i<risk_len;++i) {
        const RiskRule *rule = &RULES[i];
        const Need *need = find_need(rule->need_id);
        if (risk_len > 0 && risks[i].score != ((rule->base_score + need->importance > 100) ? 100 : rule->base_score + need->importance)) score_formula_ok = false;
        if (risks[i].mitigation == NULL || risks[i].mitigation[0] == '\0') mitigations_ok = false;
        if (i + 1 < risk_len && risks[i].score < risks[i + 1].score) sorted_ok = false;
    }
    top_pair_ok = risk_len >= 2 && strcmp(risks[0].clause_id, "H1") == 0 && risks[0].score == 100 && strcmp(risks[1].clause_id, "H2") == 0 && risks[1].score == 100;

    printf("=== Answer ===\n");
    printf("The EHDS secondary-use agreement yields four ranked risks; H1 and H2 normalize to score 100, followed by H3 at 88 and H4 at 80.\n");
    printf("\n=== Reason Why ===\n");
    printf("The agreement instantiates concrete clauses, permissions, patient needs, and rule applications. A risk appears when a permission is missing a required safeguard.\n");
    for (size_t i=0;i<risk_len;++i) {
        printf("Risk #%zu\n", i + 1);
        printf("  clause        : %s\n", risks[i].clause_id);
        printf("  permission    : %s\n", risks[i].permission_id);
        printf("  action        : %s\n", action_name(risks[i].action));
        printf("  violated need : %s\n", risks[i].need_id);
        printf("  score raw     : %u\n", risks[i].score_raw);
        printf("  score         : %u\n", risks[i].score);
        printf("  source        : %s\n", risks[i].risk_source);
        printf("  mitigation    : %s\n", risks[i].mitigation);
    }
    printf("\n=== Check ===\n");
    printf("risk count = 4          : %s\n", risk_len == 4 ? "yes" : "no");
    printf("score formula recomputes: %s\n", score_formula_ok ? "yes" : "no");
    printf("ranking sorted desc     : %s\n", sorted_ok ? "yes" : "no");
    printf("expected top pair       : %s\n", top_pair_ok ? "yes" : "no");
    printf("every risk has mitigation: %s\n", mitigations_ok ? "yes" : "no");
    return (risk_len == 4 && score_formula_ok && sorted_ok && top_pair_ok && mitigations_ok) ? 0 : 1;
}
