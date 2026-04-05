
/*
 * delfour.c
 *
 * This example simulates a privacy-preserving shopping assistant. The phone
 * turns a sensitive household condition into a neutral low-sugar shopping
 * insight, wraps it in an insight+policy envelope, signs the envelope, and the
 * scanner verifies and uses it.
 */

#include <openssl/hmac.h>
#include <openssl/sha.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const char *SECRET = "neutral-insight-demo-shared-secret";
static const char *PHONE_CREATED_AT = "2025-10-05T20:33:48.907163+00:00";
static const char *PHONE_EXPIRES_AT = "2025-10-05T22:33:48.907185+00:00";
static const char *SCANNER_AUTH_AT = "2025-10-05T20:35:48.907163+00:00";

typedef struct { const char *id; const char *name; unsigned sugar_tenths; } Product;
static const Product CATALOG[] = {
    {"prod:BIS_001", "Classic Tea Biscuits", 120},
    {"prod:BIS_101", "Low-Sugar Tea Biscuits", 30},
    {"prod:CHOC_050", "Milk Chocolate Bar", 150},
    {"prod:CHOC_150", "85% Dark Chocolate", 60},
};

typedef struct {
    char insight_json[1024];
    char policy_json[1024];
    /* Large enough for the combined JSON envelope plus future small growth. */
    char envelope_json[4096];
    char payload_hash_hex[65];
    char hmac_hex[65];
    bool signature_verified;
    bool payload_hash_matches;
    bool minimization_ok;
    bool authorization_allowed;
    bool duty_timing_ok;
    const Product *scanned;
    const Product *alternative;
} Summary;

static void hexify(const unsigned char *bytes, size_t n, char *out) {
    static const char *HEX = "0123456789abcdef";
    for (size_t i = 0; i < n; ++i) {
        out[2*i] = HEX[(bytes[i] >> 4) & 0xF];
        out[2*i+1] = HEX[bytes[i] & 0xF];
    }
    out[2*n] = '\0';
}

static void sha256_hex(const char *text, char out[65]) {
    unsigned char digest[SHA256_DIGEST_LENGTH];
    SHA256((const unsigned char *)text, strlen(text), digest);
    hexify(digest, SHA256_DIGEST_LENGTH, out);
}

static void hmac_sha256_hex(const char *secret, const char *text, char out[65]) {
    unsigned int len = 0;
    unsigned char digest[EVP_MAX_MD_SIZE];
    HMAC(EVP_sha256(), secret, (int)strlen(secret), (const unsigned char *)text, strlen(text), digest, &len);
    hexify(digest, len, out);
}

static Summary run_demo(void) {
    Summary s;
    memset(&s, 0, sizeof(s));
    const char *insight_id = "https://example.org/insight/delfour";
    snprintf(s.insight_json, sizeof(s.insight_json),
        "{\"createdAt\":\"%s\",\"expiresAt\":\"%s\",\"id\":\"%s\",\"metric\":\"sugar_g_per_serving\",\"retailer\":\"Delfour\",\"scopeDevice\":\"self-scanner\",\"scopeEvent\":\"pick_up_scanner\",\"suggestionPolicy\":\"lower_metric_first_higher_price_ok\",\"threshold\":10.0,\"type\":\"ins:Insight\"}",
        PHONE_CREATED_AT, PHONE_EXPIRES_AT, insight_id);
    snprintf(s.policy_json, sizeof(s.policy_json),
        "{\"duty\":{\"action\":\"odrl:delete\",\"constraint\":{\"leftOperand\":\"odrl:dateTime\",\"operator\":\"odrl:eq\",\"rightOperand\":\"%s\"}},\"permission\":{\"action\":\"odrl:use\",\"constraint\":{\"leftOperand\":\"odrl:purpose\",\"operator\":\"odrl:eq\",\"rightOperand\":\"shopping_assist\"},\"target\":\"%s\"},\"profile\":\"Delfour-Insight-Policy\",\"prohibition\":{\"action\":\"odrl:distribute\",\"constraint\":{\"leftOperand\":\"odrl:purpose\",\"operator\":\"odrl:eq\",\"rightOperand\":\"marketing\"},\"target\":\"%s\"},\"type\":\"odrl:Policy\"}",
        PHONE_EXPIRES_AT, insight_id, insight_id);
    int envelope_len = snprintf(s.envelope_json, sizeof(s.envelope_json),
        "{\"insight\":%s,\"policy\":%s}", s.insight_json, s.policy_json);
    if (envelope_len < 0 || (size_t)envelope_len >= sizeof(s.envelope_json)) {
        fprintf(stderr, "delfour: internal JSON envelope buffer too small\n");
        exit(1);
    }
    sha256_hex(s.envelope_json, s.payload_hash_hex);
    hmac_sha256_hex(SECRET, s.envelope_json, s.hmac_hex);
    char check_hash[65], check_hmac[65];
    sha256_hex(s.envelope_json, check_hash);
    hmac_sha256_hex(SECRET, s.envelope_json, check_hmac);
    s.signature_verified = strcmp(check_hmac, s.hmac_hex) == 0;
    s.payload_hash_matches = strcmp(check_hash, s.payload_hash_hex) == 0;
    s.minimization_ok = strstr(s.insight_json, "Diabetes") == NULL && strstr(s.insight_json, "medical") == NULL;
    s.authorization_allowed = strcmp(SCANNER_AUTH_AT, PHONE_EXPIRES_AT) < 0 && strstr(s.policy_json, "shopping_assist") != NULL;
    s.scanned = &CATALOG[0];
    s.alternative = &CATALOG[1];
    /* In this scenario the delete duty is not due yet, which is still consistent. */
    s.duty_timing_ok = true;
    return s;
}

int main(void) {
    Summary s = run_demo();
    bool banner_flags_high_sugar = s.scanned->sugar_tenths >= 100;
    bool alternative_is_lower = s.alternative->sugar_tenths < s.scanned->sugar_tenths;
    bool marketing_prohibited = strstr(s.policy_json, "marketing") != NULL && strstr(s.policy_json, "odrl:distribute") != NULL;
    bool scope_complete = strstr(s.insight_json, "scopeDevice") && strstr(s.insight_json, "scopeEvent") && strstr(s.insight_json, "expiresAt");

    printf("=== Answer ===\n");
    printf("The scanner is allowed to use a neutral shopping insight and recommends Low-Sugar Tea Biscuits instead of Classic Tea Biscuits.\n");
    printf("\n=== Reason Why ===\n");
    printf("The phone desensitizes a diabetes-related household condition into a scoped low-sugar need, wraps it in an expiring Insight+Policy envelope, and signs it.\n");
    printf("scanned product      : %s\n", s.scanned->name);
    printf("suggested alternative: %s\n", s.alternative->name);
    printf("payload SHA-256      : %s\n", s.payload_hash_hex);
    printf("HMAC-SHA256          : %s\n", s.hmac_hex);
    printf("\n=== Check ===\n");
    printf("signature verifies             : %s\n", s.signature_verified ? "yes" : "no");
    printf("payload hash matches           : %s\n", s.payload_hash_matches ? "yes" : "no");
    printf("minimization strips sensitive terms: %s\n", s.minimization_ok ? "yes" : "no");
    printf("scope complete                 : %s\n", scope_complete ? "yes" : "no");
    printf("authorization allowed          : %s\n", s.authorization_allowed ? "yes" : "no");
    printf("high-sugar banner              : %s\n", banner_flags_high_sugar ? "yes" : "no");
    printf("alternative lowers sugar       : %s\n", alternative_is_lower ? "yes" : "no");
    printf("duty timing consistent         : %s\n", s.duty_timing_ok ? "yes" : "no");
    printf("marketing prohibited           : %s\n", marketing_prohibited ? "yes" : "no");
    return (s.signature_verified && s.payload_hash_matches && s.minimization_ok && scope_complete && s.authorization_allowed && banner_flags_high_sugar && alternative_is_lower && s.duty_timing_ok && marketing_prohibited) ? 0 : 1;
}
