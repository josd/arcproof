use std::io;

use crate::report::{CaseReport, ReportItem};

const VCC_MV: u32 = 5_000;
const VIN_LOW_MV: u32 = 0;
const VIN_HIGH_MV: u32 = 5_000;
const VBE_ON_MV: u32 = 700;
const VCE_SAT_MV: u32 = 200;
const BASE_RESISTANCE_OHMS: u32 = 10_000;
const LOAD_RESISTANCE_OHMS: u32 = 1_000;
const TRANSISTOR_BETA: u32 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SwitchState {
    input_mv: u32,
    base_current_ua: u32,
    collector_gain_limit_ua: u32,
    collector_load_limit_ua: u32,
    collector_current_ua: u32,
    load_voltage_mv: u32,
    collector_emitter_voltage_mv: u32,
    cutoff: bool,
    saturation: bool,
}

struct TransistorSwitchReport {
    low_input: SwitchState,
    high_input: SwitchState,
    switches_cleanly: bool,
    low_input_stays_in_cutoff: bool,
    high_input_reaches_saturation: bool,
    on_state_is_load_limited: bool,
    load_voltage_matches_resistor_drop: bool,
}

pub fn report() -> io::Result<CaseReport> {
    let report = evaluate();

    if !report.switches_cleanly
        || !report.low_input_stays_in_cutoff
        || !report.high_input_reaches_saturation
        || !report.on_state_is_load_limited
        || !report.load_voltage_matches_resistor_drop
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "transistor-switch check failed: cutoff/saturation or load-line checks did not hold",
        ));
    }

    Ok(CaseReport::new("transistor-switch")
        .with_answer(vec![
            ReportItem::text(
                "In this toy transistor-switch model, a low input leaves the transistor in cutoff (OFF) and a high input drives it into saturation (ON), so the load behaves like an on/off branch rather than a linear amplifier.",
            ),
            ReportItem::field("case", "transistor-switch"),
            ReportItem::field("low input state", state_name(report.low_input)),
            ReportItem::field("high input state", state_name(report.high_input)),
            ReportItem::field(
                "on-state load current",
                format_ma_from_ua(report.high_input.collector_current_ua),
            ),
        ])
        .with_reason_why(vec![
            ReportItem::text(
                "We model an NPN low-side switch with exact millivolt and microamp arithmetic. The base current comes from (Vin - Vbe)/Rb when the base-emitter junction is forward biased, and the collector current is the smaller of beta * Ib and the load-limited current (Vcc - Vce,sat)/Rl.",
            ),
            ReportItem::field("supply voltage", format_v_from_mv(VCC_MV)),
            ReportItem::field("base resistor", format!("{} ohms", BASE_RESISTANCE_OHMS)),
            ReportItem::field("load resistor", format!("{} ohms", LOAD_RESISTANCE_OHMS)),
            ReportItem::field("transistor beta proxy", TRANSISTOR_BETA.to_string()),
            ReportItem::field("low input", format_state_summary(report.low_input)),
            ReportItem::field("high input", format_state_summary(report.high_input)),
            ReportItem::field(
                "high-input gain limit",
                format_ma_from_ua(report.high_input.collector_gain_limit_ua),
            ),
            ReportItem::field(
                "high-input load limit",
                format_ma_from_ua(report.high_input.collector_load_limit_ua),
            ),
        ])
        .with_check(vec![
            ReportItem::field(
                "low input stays in cutoff",
                yes_no(report.low_input_stays_in_cutoff),
            ),
            ReportItem::field(
                "high input reaches saturation",
                yes_no(report.high_input_reaches_saturation),
            ),
            ReportItem::field(
                "switching states differ",
                yes_no(report.switches_cleanly),
            ),
            ReportItem::field(
                "on-state current is load-limited",
                yes_no(report.on_state_is_load_limited),
            ),
            ReportItem::field(
                "load voltage matches resistor drop",
                yes_no(report.load_voltage_matches_resistor_drop),
            ),
        ]))
}

pub fn run_and_print() -> io::Result<()> {
    let report = evaluate();

    println!("=== Answer ===");
    println!(
        "In this toy transistor-switch model, a low input leaves the transistor in cutoff (OFF) and a high input drives it into saturation (ON), so the load behaves like an on/off branch rather than a linear amplifier."
    );
    println!("case                             : transistor-switch");
    println!("low input state                  : {}", state_name(report.low_input));
    println!("high input state                 : {}", state_name(report.high_input));
    println!(
        "on-state load current            : {}",
        format_ma_from_ua(report.high_input.collector_current_ua)
    );
    println!();
    println!("=== Reason Why ===");
    println!(
        "We model an NPN low-side switch with exact millivolt and microamp arithmetic. The base current comes from (Vin - Vbe)/Rb when the base-emitter junction is forward biased, and the collector current is the smaller of beta * Ib and the load-limited current (Vcc - Vce,sat)/Rl."
    );
    println!("supply voltage                   : {}", format_v_from_mv(VCC_MV));
    println!("base resistor                    : {} ohms", BASE_RESISTANCE_OHMS);
    println!("load resistor                    : {} ohms", LOAD_RESISTANCE_OHMS);
    println!("transistor beta proxy            : {}", TRANSISTOR_BETA);
    println!("low input                        : {}", format_state_summary(report.low_input));
    println!("high input                       : {}", format_state_summary(report.high_input));
    println!(
        "high-input gain limit            : {}",
        format_ma_from_ua(report.high_input.collector_gain_limit_ua)
    );
    println!(
        "high-input load limit            : {}",
        format_ma_from_ua(report.high_input.collector_load_limit_ua)
    );
    println!();
    println!("=== Check ===");
    println!(
        "low input stays in cutoff        : {}",
        yes_no(report.low_input_stays_in_cutoff)
    );
    println!(
        "high input reaches saturation    : {}",
        yes_no(report.high_input_reaches_saturation)
    );
    println!(
        "switching states differ          : {}",
        yes_no(report.switches_cleanly)
    );
    println!(
        "on-state current is load-limited : {}",
        yes_no(report.on_state_is_load_limited)
    );
    println!(
        "load voltage matches resistor drop : {}",
        yes_no(report.load_voltage_matches_resistor_drop)
    );

    if !report.switches_cleanly
        || !report.low_input_stays_in_cutoff
        || !report.high_input_reaches_saturation
        || !report.on_state_is_load_limited
        || !report.load_voltage_matches_resistor_drop
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "transistor-switch check failed: cutoff/saturation or load-line checks did not hold",
        ));
    }

    Ok(())
}

fn evaluate() -> TransistorSwitchReport {
    let low_input = evaluate_state(VIN_LOW_MV);
    let high_input = evaluate_state(VIN_HIGH_MV);

    TransistorSwitchReport {
        low_input,
        high_input,
        switches_cleanly: low_input.cutoff && high_input.saturation,
        low_input_stays_in_cutoff: low_input.cutoff
            && low_input.base_current_ua == 0
            && low_input.collector_current_ua == 0
            && low_input.load_voltage_mv == 0
            && low_input.collector_emitter_voltage_mv == VCC_MV,
        high_input_reaches_saturation: high_input.saturation
            && high_input.collector_emitter_voltage_mv == VCE_SAT_MV,
        on_state_is_load_limited: high_input.collector_current_ua == high_input.collector_load_limit_ua
            && high_input.collector_gain_limit_ua > high_input.collector_load_limit_ua,
        load_voltage_matches_resistor_drop: high_input.load_voltage_mv
            == (high_input.collector_current_ua * LOAD_RESISTANCE_OHMS) / 1_000,
    }
}

fn evaluate_state(input_mv: u32) -> SwitchState {
    if input_mv <= VBE_ON_MV {
        return SwitchState {
            input_mv,
            base_current_ua: 0,
            collector_gain_limit_ua: 0,
            collector_load_limit_ua: 0,
            collector_current_ua: 0,
            load_voltage_mv: 0,
            collector_emitter_voltage_mv: VCC_MV,
            cutoff: true,
            saturation: false,
        };
    }

    let base_current_ua = ((input_mv - VBE_ON_MV) * 1_000) / BASE_RESISTANCE_OHMS;
    let collector_gain_limit_ua = base_current_ua * TRANSISTOR_BETA;
    let collector_load_limit_ua = ((VCC_MV - VCE_SAT_MV) * 1_000) / LOAD_RESISTANCE_OHMS;
    let collector_current_ua = collector_gain_limit_ua.min(collector_load_limit_ua);
    let saturation = collector_gain_limit_ua >= collector_load_limit_ua;
    let load_voltage_mv = (collector_current_ua * LOAD_RESISTANCE_OHMS) / 1_000;
    let collector_emitter_voltage_mv = if saturation {
        VCE_SAT_MV
    } else {
        VCC_MV.saturating_sub(load_voltage_mv)
    };

    SwitchState {
        input_mv,
        base_current_ua,
        collector_gain_limit_ua,
        collector_load_limit_ua,
        collector_current_ua,
        load_voltage_mv,
        collector_emitter_voltage_mv,
        cutoff: false,
        saturation,
    }
}

fn state_name(state: SwitchState) -> &'static str {
    if state.cutoff {
        "cutoff / OFF"
    } else if state.saturation {
        "saturation / ON"
    } else {
        "active / linear"
    }
}

fn format_state_summary(state: SwitchState) -> String {
    format!(
        "Vin={} -> Ib={}, Ic={}, Vce={}, state={}",
        format_v_from_mv(state.input_mv),
        format_ma_from_ua(state.base_current_ua),
        format_ma_from_ua(state.collector_current_ua),
        format_v_from_mv(state.collector_emitter_voltage_mv),
        state_name(state)
    )
}

fn format_v_from_mv(mv: u32) -> String {
    format!("{}.{:02} V", mv / 1_000, (mv % 1_000) / 10)
}

fn format_ma_from_ua(ua: u32) -> String {
    format!("{}.{:02} mA", ua / 1_000, (ua % 1_000) / 10)
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_input_keeps_the_switch_open() {
        let report = evaluate();

        assert_eq!(report.low_input.input_mv, 0);
        assert_eq!(report.low_input.base_current_ua, 0);
        assert_eq!(report.low_input.collector_current_ua, 0);
        assert_eq!(report.low_input.load_voltage_mv, 0);
        assert_eq!(report.low_input.collector_emitter_voltage_mv, 5_000);
        assert!(report.low_input.cutoff);
        assert!(!report.low_input.saturation);
    }

    #[test]
    fn high_input_drives_the_transistor_into_saturation() {
        let report = evaluate();

        assert_eq!(report.high_input.input_mv, 5_000);
        assert_eq!(report.high_input.base_current_ua, 430);
        assert_eq!(report.high_input.collector_gain_limit_ua, 43_000);
        assert_eq!(report.high_input.collector_load_limit_ua, 4_800);
        assert_eq!(report.high_input.collector_current_ua, 4_800);
        assert_eq!(report.high_input.load_voltage_mv, 4_800);
        assert_eq!(report.high_input.collector_emitter_voltage_mv, 200);
        assert!(report.high_input.saturation);
    }

    #[test]
    fn transistor_switch_report_is_consistent() {
        let report = evaluate();

        assert!(report.switches_cleanly);
        assert!(report.low_input_stays_in_cutoff);
        assert!(report.high_input_reaches_saturation);
        assert!(report.on_state_is_load_limited);
        assert!(report.load_voltage_matches_resistor_drop);
    }
}
