use std::cmp::Reverse;
use std::io;

use crate::report::{CaseReport, ReportItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    ProvideSecondaryUseData,
    Download,
    RemoveDirectIdentifiers,
    ProcessOnlyInSecureEnvironment,
}

impl Action {
    fn as_str(self) -> &'static str {
        match self {
            Action::ProvideSecondaryUseData => "provideSecondaryUseData",
            Action::Download => "download",
            Action::RemoveDirectIdentifiers => "removeDirectIdentifiers",
            Action::ProcessOnlyInSecureEnvironment => "processOnlyInSecureEnvironment",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstraintKey {
    Purpose,
    HasDataPermit,
    RespectOptOutSecondaryUse,
    StatisticallyAnonymised,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstraintValue {
    Reference(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Constraint {
    key: ConstraintKey,
    value: ConstraintValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Duty {
    action: Action,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Need {
    id: &'static str,
    importance: u32,
    related_right: Option<&'static str>,
    description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Clause {
    id: &'static str,
    text: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Permission {
    id: &'static str,
    clause_id: &'static str,
    action: Action,
    constraints: Vec<Constraint>,
    duties: Vec<Duty>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Agreement {
    holder: &'static str,
    access_body: &'static str,
    user: &'static str,
    dataset: &'static str,
    process_context: &'static str,
    clauses: Vec<Clause>,
    permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MissingSafeguard {
    DataPermitConstraint,
    OptOutConstraint,
    SecureEnvironmentDuty,
    StatisticalAnonymisationConstraint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RiskRule {
    rule_id: &'static str,
    permission_id: &'static str,
    clause_id: &'static str,
    need_id: &'static str,
    base_score: u32,
    risk_source_description: &'static str,
    finding_template: &'static str,
    mitigation: &'static str,
    missing_safeguard: MissingSafeguard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Severity {
    High,
    Moderate,
    Low,
}

impl Severity {
    fn as_str(self) -> &'static str {
        match self {
            Severity::High => "HighSeverity / HighRisk",
            Severity::Moderate => "ModerateSeverity / ModerateRisk",
            Severity::Low => "LowSeverity / LowRisk",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RankedRisk {
    rule_id: &'static str,
    permission_id: &'static str,
    clause_id: &'static str,
    action: Action,
    violated_need_id: &'static str,
    need_importance: u32,
    score_raw: u32,
    score: u32,
    severity: Severity,
    description: String,
    risk_source_description: &'static str,
    mitigation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Evaluation {
    agreement: Agreement,
    needs: Vec<Need>,
    ranked_risks: Vec<RankedRisk>,
    score_formula_ok: bool,
    ranking_sorted_desc: bool,
    every_expected_clause_flagged: bool,
    all_scores_are_high: bool,
    every_risk_has_mitigation: bool,
    independent_top_pair_ok: bool,
}

pub fn report() -> io::Result<CaseReport> {
    let evaluation = evaluate()?;
    ensure_checks(&evaluation)?;

    let top_pair = format!(
        "{}={}, {}={}",
        evaluation.ranked_risks[0].clause_id,
        evaluation.ranked_risks[0].score,
        evaluation.ranked_risks[1].clause_id,
        evaluation.ranked_risks[1].score
    );

    let mut reason = vec![
        ReportItem::text(
            "The agreement instantiates concrete clauses, permissions, patient needs, and rule applications, producing four ranked EHDS risks with explicit mitigations and independent checks.",
        ),
        ReportItem::field("health data holder", evaluation.agreement.holder),
        ReportItem::field("health data access body", evaluation.agreement.access_body),
        ReportItem::field("health data user", evaluation.agreement.user),
        ReportItem::field("dataset", evaluation.agreement.dataset),
        ReportItem::field("process context", evaluation.agreement.process_context),
        ReportItem::field("patient needs", evaluation.needs.len().to_string()),
        ReportItem::field("risks found", evaluation.ranked_risks.len().to_string()),
        ReportItem::field("top pair", top_pair),
    ];

    for (index, risk) in evaluation.ranked_risks.iter().enumerate() {
        reason.push(ReportItem::text(format!("Risk #{}", index + 1)));
        reason.push(ReportItem::field("clause", risk.clause_id));
        reason.push(ReportItem::field("permission", risk.permission_id));
        reason.push(ReportItem::field("action", risk.action.as_str()));
        reason.push(ReportItem::field("violated need", risk.violated_need_id));
        reason.push(ReportItem::field("need importance", risk.need_importance.to_string()));
        reason.push(ReportItem::field("score raw", risk.score_raw.to_string()));
        reason.push(ReportItem::field("score", risk.score.to_string()));
        reason.push(ReportItem::field("severity", risk.severity.as_str()));
        reason.push(ReportItem::field("risk source", risk.risk_source_description));
        reason.push(ReportItem::field("why", &risk.description));
        reason.push(ReportItem::field("mitigation", risk.mitigation));
    }

    Ok(CaseReport::new("odrl-dpv-ehds-risk-ranked")
        .with_answer(vec![
            ReportItem::text(
                "The EHDS secondary-use agreement yields four ranked risks; H1 and H2 normalize to score 100, followed by H3 at 88 and H4 at 80.",
            ),
            ReportItem::field("case", "odrl-dpv-ehds-risk-ranked"),
            ReportItem::field("agreement", "EHDS Secondary Use Agreement (example)"),
            ReportItem::field("ranked risks", evaluation.ranked_risks.len().to_string()),
        ])
        .with_reason_why(reason)
        .with_check(vec![
            ReportItem::field("score formula recomputes", yes_no(evaluation.score_formula_ok)),
            ReportItem::field("ranking sorted descending", yes_no(evaluation.ranking_sorted_desc)),
            ReportItem::field(
                "all expected clauses flagged",
                yes_no(evaluation.every_expected_clause_flagged),
            ),
            ReportItem::field("all severities high", yes_no(evaluation.all_scores_are_high)),
            ReportItem::field(
                "every risk has mitigation",
                yes_no(evaluation.every_risk_has_mitigation),
            ),
            ReportItem::field(
                "independent top pair check",
                yes_no(evaluation.independent_top_pair_ok),
            ),
        ]))
}

pub fn run_and_print() -> io::Result<()> {
    let evaluation = evaluate()?;
    ensure_checks(&evaluation)?;

    println!("=== Answer ===");
    println!(
        "The EHDS secondary-use agreement yields four ranked risks; H1 and H2 normalize to score 100, followed by H3 at 88 and H4 at 80."
    );
    println!("case                 : odrl-dpv-ehds-risk-ranked");
    println!("agreement            : EHDS Secondary Use Agreement (example)");
    println!("ranked risks         : {}", evaluation.ranked_risks.len());
    println!();
    println!("=== Reason Why ===");
    println!(
        "The agreement instantiates concrete clauses, permissions, patient needs, and rule applications, producing four ranked EHDS risks with explicit mitigations and independent checks."
    );
    println!("health data holder   : {}", evaluation.agreement.holder);
    println!("health data access body: {}", evaluation.agreement.access_body);
    println!("health data user     : {}", evaluation.agreement.user);
    println!("dataset              : {}", evaluation.agreement.dataset);
    println!("process context      : {}", evaluation.agreement.process_context);
    println!("patient needs        : {}", evaluation.needs.len());
    println!("risks found          : {}", evaluation.ranked_risks.len());
    println!(
        "top pair             : {}={}, {}={}",
        evaluation.ranked_risks[0].clause_id,
        evaluation.ranked_risks[0].score,
        evaluation.ranked_risks[1].clause_id,
        evaluation.ranked_risks[1].score
    );

    for (index, risk) in evaluation.ranked_risks.iter().enumerate() {
        println!();
        println!("Risk #{}", index + 1);
        println!(" clause              : {}", risk.clause_id);
        println!(" permission          : {}", risk.permission_id);
        println!(" action              : {}", risk.action.as_str());
        println!(" violated need       : {}", risk.violated_need_id);
        println!(" need importance     : {}", risk.need_importance);
        println!(" score raw           : {}", risk.score_raw);
        println!(" score               : {}", risk.score);
        println!(" severity            : {}", risk.severity.as_str());
        println!(" risk source         : {}", risk.risk_source_description);
        println!(" why                 : {}", risk.description);
        println!(" mitigation          : {}", risk.mitigation);
    }

    println!();
    println!("=== Check ===");
    println!(
        "score formula recomputes  : {}",
        yes_no(evaluation.score_formula_ok)
    );
    println!(
        "ranking sorted descending : {}",
        yes_no(evaluation.ranking_sorted_desc)
    );
    println!(
        "all expected clauses flagged : {}",
        yes_no(evaluation.every_expected_clause_flagged)
    );
    println!(
        "all severities high      : {}",
        yes_no(evaluation.all_scores_are_high)
    );
    println!(
        "every risk has mitigation: {}",
        yes_no(evaluation.every_risk_has_mitigation)
    );
    println!(
        "independent top pair check: {}",
        yes_no(evaluation.independent_top_pair_ok)
    );

    Ok(())
}

fn evaluate() -> io::Result<Evaluation> {
    let needs = needs();
    let agreement = agreement();
    let rules = rules();
    let mut ranked_risks = derive_risks(&agreement, &needs, &rules)?;
    ranked_risks.sort_by_key(|risk| (Reverse(risk.score), risk.clause_id));

    let score_formula_ok = ranked_risks.iter().all(|risk| {
        let expected_raw = rule_by_id(&rules, risk.rule_id)
            .map(|rule| rule.base_score)
            .unwrap_or(0)
            + risk.need_importance;
        risk.score_raw == expected_raw && risk.score == expected_raw.min(100)
    });
    let ranking_sorted_desc = ranked_risks.windows(2).all(|pair| pair[0].score >= pair[1].score);
    let every_expected_clause_flagged = ranked_risks
        .iter()
        .map(|risk| risk.clause_id)
        .collect::<Vec<_>>()
        == vec!["H1", "H2", "H3", "H4"];
    let all_scores_are_high = ranked_risks.iter().all(|risk| risk.severity == Severity::High);
    let every_risk_has_mitigation = ranked_risks.iter().all(|risk| !risk.mitigation.is_empty());
    let independently_ranked = independently_rank_scores(&agreement, &needs)?;
    let independent_top_pair_ok = independently_ranked
        .iter()
        .take(2)
        .copied()
        .eq([("H1", 100), ("H2", 100)].into_iter());

    Ok(Evaluation {
        agreement,
        needs,
        ranked_risks,
        score_formula_ok,
        ranking_sorted_desc,
        every_expected_clause_flagged,
        all_scores_are_high,
        every_risk_has_mitigation,
        independent_top_pair_ok,
    })
}

fn derive_risks(
    agreement: &Agreement,
    needs: &[Need],
    rules: &[RiskRule],
) -> io::Result<Vec<RankedRisk>> {
    let mut out = Vec::new();

    for rule in rules {
        let need = need_by_id(needs, rule.need_id)?;
        let permission = permission_by_id(&agreement.permissions, rule.permission_id)?;
        let clause = clause_by_id(&agreement.clauses, rule.clause_id)?;

        if !matches_missing_safeguard(permission, rule.missing_safeguard) {
            continue;
        }

        let score_raw = rule.base_score + need.importance;
        let score = score_raw.min(100);
        out.push(RankedRisk {
            rule_id: rule.rule_id,
            permission_id: permission.id,
            clause_id: clause.id,
            action: permission.action,
            violated_need_id: need.id,
            need_importance: need.importance,
            score_raw,
            score,
            severity: classify(score),
            description: rule.finding_template.replace("{}", clause.id).replacen("{}", clause.text, 1),
            risk_source_description: rule.risk_source_description,
            mitigation: rule.mitigation,
        });
    }

    Ok(out)
}

fn independently_rank_scores(
    agreement: &Agreement,
    needs: &[Need],
) -> io::Result<Vec<(&'static str, u32)>> {
    let mut pairs = vec![
        (
            "H1",
            independent_score(
                permission_by_id(&agreement.permissions, "PermSecondaryUseDUA")?,
                need_by_id(needs, "Need_RequireDataPermit")?.importance,
                80,
                MissingSafeguard::DataPermitConstraint,
            ),
        ),
        (
            "H2",
            independent_score(
                permission_by_id(&agreement.permissions, "PermSecondaryUseAllPatients")?,
                need_by_id(needs, "Need_RespectOptOutSecondaryUse")?.importance,
                75,
                MissingSafeguard::OptOutConstraint,
            ),
        ),
        (
            "H3",
            independent_score(
                permission_by_id(&agreement.permissions, "PermDownloadLocalCopy")?,
                need_by_id(needs, "Need_SecureProcessingEnvironment")?.importance,
                70,
                MissingSafeguard::SecureEnvironmentDuty,
            ),
        ),
        (
            "H4",
            independent_score(
                permission_by_id(&agreement.permissions, "PermProvidePseudonymisedData")?,
                need_by_id(needs, "Need_StatisticallyAnonymisedSecondaryUse")?.importance,
                65,
                MissingSafeguard::StatisticalAnonymisationConstraint,
            ),
        ),
    ];
    pairs.sort_by_key(|(clause_id, score)| (Reverse(*score), *clause_id));
    Ok(pairs)
}

fn independent_score(
    permission: &Permission,
    importance: u32,
    base: u32,
    missing_safeguard: MissingSafeguard,
) -> u32 {
    let penalty = if matches_missing_safeguard(permission, missing_safeguard) {
        base
    } else {
        0
    };
    (penalty + importance).min(100)
}

fn matches_missing_safeguard(permission: &Permission, safeguard: MissingSafeguard) -> bool {
    match safeguard {
        MissingSafeguard::DataPermitConstraint => {
            !has_constraint(permission, ConstraintKey::HasDataPermit)
        }
        MissingSafeguard::OptOutConstraint => {
            !has_constraint(permission, ConstraintKey::RespectOptOutSecondaryUse)
        }
        MissingSafeguard::SecureEnvironmentDuty => {
            !has_duty(permission, Action::ProcessOnlyInSecureEnvironment)
        }
        MissingSafeguard::StatisticalAnonymisationConstraint => {
            !has_constraint(permission, ConstraintKey::StatisticallyAnonymised)
        }
    }
}

fn has_constraint(permission: &Permission, key: ConstraintKey) -> bool {
    permission.constraints.iter().any(|constraint| constraint.key == key)
}

fn has_duty(permission: &Permission, action: Action) -> bool {
    permission.duties.iter().any(|duty| duty.action == action)
}

fn classify(score: u32) -> Severity {
    if score > 79 {
        Severity::High
    } else if score > 49 {
        Severity::Moderate
    } else {
        Severity::Low
    }
}

fn needs() -> Vec<Need> {
    vec![
        Need {
            id: "Need_RequireDataPermit",
            importance: 20,
            related_right: None,
            description: "Secondary use should be authorised via an EHDS Data Permit issued by a Health Data Access Body.",
        },
        Need {
            id: "Need_RespectOptOutSecondaryUse",
            importance: 25,
            related_right: Some("A71"),
            description: "Respect the EHDS right to opt out from secondary use.",
        },
        Need {
            id: "Need_SecureProcessingEnvironment",
            importance: 18,
            related_right: Some("A68-11"),
            description: "Secondary-use processing must occur within a secure processing environment (and not via local downloads).",
        },
        Need {
            id: "Need_StatisticallyAnonymisedSecondaryUse",
            importance: 15,
            related_right: None,
            description: "Secondary use should use statistically anonymised electronic health data (e.g., via an EHDS Health Data Request).",
        },
    ]
}

fn agreement() -> Agreement {
    Agreement {
        holder: "St. Example Hospital (Health Data Holder)",
        access_body: "Example Health Data Access Body (BE)",
        user: "Example University Lab (Health Data User)",
        dataset: "Combined dataset (EHR + genomics + clinical trials data)",
        process_context: "Secondary use under AgreementEHDS1",
        clauses: vec![
            Clause {
                id: "H1",
                text: "Hospital may provide electronic health data for secondary use based on a bilateral data use agreement with the applicant.",
            },
            Clause {
                id: "H2",
                text: "Secondary use may include all patient records for training and evaluating health-related algorithms.",
            },
            Clause {
                id: "H3",
                text: "The applicant may download a complete local copy of the dataset to its own infrastructure for analysis.",
            },
            Clause {
                id: "H4",
                text: "The dataset will be provided in pseudonymised form by removing direct identifiers.",
            },
        ],
        permissions: vec![
            Permission {
                id: "PermSecondaryUseDUA",
                clause_id: "H1",
                action: Action::ProvideSecondaryUseData,
                constraints: vec![Constraint {
                    key: ConstraintKey::Purpose,
                    value: ConstraintValue::Reference("HealthcareScientificResearch"),
                }],
                duties: vec![],
            },
            Permission {
                id: "PermSecondaryUseAllPatients",
                clause_id: "H2",
                action: Action::ProvideSecondaryUseData,
                constraints: vec![Constraint {
                    key: ConstraintKey::Purpose,
                    value: ConstraintValue::Reference("TrainTestAndEvaluateHealthAlgorithms"),
                }],
                duties: vec![],
            },
            Permission {
                id: "PermDownloadLocalCopy",
                clause_id: "H3",
                action: Action::Download,
                constraints: vec![],
                duties: vec![],
            },
            Permission {
                id: "PermProvidePseudonymisedData",
                clause_id: "H4",
                action: Action::ProvideSecondaryUseData,
                constraints: vec![],
                duties: vec![Duty {
                    action: Action::RemoveDirectIdentifiers,
                }],
            },
        ],
    }
}

fn rules() -> Vec<RiskRule> {
    vec![
        RiskRule {
            rule_id: "R1",
            permission_id: "PermSecondaryUseDUA",
            clause_id: "H1",
            need_id: "Need_RequireDataPermit",
            base_score: 80,
            risk_source_description: "Secondary use permitted without EHDS Data Permit.",
            finding_template: "Risk: secondary use is permitted without an EHDS Data Permit safeguard. Clause {}: {}",
            mitigation: "Require an EHDS Data Permit issued by a Health Data Access Body before secondary use.",
            missing_safeguard: MissingSafeguard::DataPermitConstraint,
        },
        RiskRule {
            rule_id: "R2",
            permission_id: "PermSecondaryUseAllPatients",
            clause_id: "H2",
            need_id: "Need_RespectOptOutSecondaryUse",
            base_score: 75,
            risk_source_description: "Opt-out from secondary use not explicitly respected.",
            finding_template: "Risk: secondary use may include patients who opted out (EHDS A71). Clause {}: {}",
            mitigation: "Add an explicit safeguard that excludes records of persons who exercised the EHDS opt-out from secondary use.",
            missing_safeguard: MissingSafeguard::OptOutConstraint,
        },
        RiskRule {
            rule_id: "R3",
            permission_id: "PermDownloadLocalCopy",
            clause_id: "H3",
            need_id: "Need_SecureProcessingEnvironment",
            base_score: 70,
            risk_source_description: "Local download permitted; secure processing environment not required.",
            finding_template: "Risk: the agreement permits local downloads rather than processing within a secure processing environment. Clause {}: {}",
            mitigation: "Require processing only within a secure processing environment and prohibit local downloads of raw datasets.",
            missing_safeguard: MissingSafeguard::SecureEnvironmentDuty,
        },
        RiskRule {
            rule_id: "R4",
            permission_id: "PermProvidePseudonymisedData",
            clause_id: "H4",
            need_id: "Need_StatisticallyAnonymisedSecondaryUse",
            base_score: 65,
            risk_source_description: "Statistical anonymisation safeguard missing for secondary use.",
            finding_template: "Risk: secondary-use dataset is only described as pseudonymised, without a safeguard requiring statistically anonymised data for secondary use. Clause {}: {}",
            mitigation: "Require an EHDS Health Data Request for statistically anonymised data and add a constraint that secondary-use data must be statistically anonymised.",
            missing_safeguard: MissingSafeguard::StatisticalAnonymisationConstraint,
        },
    ]
}

fn need_by_id<'a>(needs: &'a [Need], id: &str) -> io::Result<&'a Need> {
    needs.iter().find(|need| need.id == id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("need not found: {id}"))
    })
}

fn clause_by_id<'a>(clauses: &'a [Clause], id: &str) -> io::Result<&'a Clause> {
    clauses.iter().find(|clause| clause.id == id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("clause not found: {id}"))
    })
}

fn permission_by_id<'a>(permissions: &'a [Permission], id: &str) -> io::Result<&'a Permission> {
    permissions.iter().find(|permission| permission.id == id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("permission not found: {id}"))
    })
}

fn rule_by_id<'a>(rules: &'a [RiskRule], id: &str) -> Option<&'a RiskRule> {
    rules.iter().find(|rule| rule.rule_id == id)
}

fn ensure_checks(evaluation: &Evaluation) -> io::Result<()> {
    if evaluation.ranked_risks.len() != 4
        || !evaluation.score_formula_ok
        || !evaluation.ranking_sorted_desc
        || !evaluation.every_expected_clause_flagged
        || !evaluation.all_scores_are_high
        || !evaluation.every_risk_has_mitigation
        || !evaluation.independent_top_pair_ok
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "odrl-dpv-ehds-risk-ranked check failed: a score, ranking, clause coverage, severity, mitigation, or independent check failed",
        ));
    }

    Ok(())
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reproduces_expected_clause_order_and_scores() {
        let evaluation = evaluate().expect("evaluation should succeed");
        let pairs: Vec<_> = evaluation
            .ranked_risks
            .iter()
            .map(|risk| (risk.clause_id, risk.score))
            .collect();
        assert_eq!(pairs, vec![("H1", 100), ("H2", 100), ("H3", 88), ("H4", 80)]);
    }

    #[test]
    fn all_ranked_risks_are_high_and_mitigated() {
        let evaluation = evaluate().expect("evaluation should succeed");
        assert!(evaluation.all_scores_are_high);
        assert!(evaluation.every_risk_has_mitigation);
    }

    #[test]
    fn independent_ranking_matches_main_ranking_prefix() {
        let evaluation = evaluate().expect("evaluation should succeed");
        assert!(evaluation.independent_top_pair_ok);
        assert_eq!(
            independently_rank_scores(&evaluation.agreement, &evaluation.needs)
                .expect("independent ranking should succeed"),
            vec![("H1", 100), ("H2", 100), ("H3", 88), ("H4", 80)]
        );
    }
}
