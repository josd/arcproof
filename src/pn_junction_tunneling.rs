use std::io;

use crate::report::{CaseReport, ReportItem};

const ORDINARY_DEPLETION_WIDTH_NM: u32 = 8;
const TUNNEL_DEPLETION_WIDTH_NM: u32 = 1;
const N_FILLED_STATES: [i32; 4] = [1, 2, 3, 4];
const P_EMPTY_ZERO_BIAS: [i32; 4] = [3, 4, 5, 6];
const BIAS_POINTS: [i32; 7] = [0, 1, 2, 3, 4, 5, 6];

struct PnJunctionReport {
    ordinary_depletion_width_nm: u32,
    tunnel_depletion_width_nm: u32,
    n_filled_states: [i32; 4],
    p_empty_zero_bias: [i32; 4],
    current_proxy: [u32; 7],
    peak_bias: i32,
    peak_current: u32,
    valley_bias: i32,
    valley_current: u32,
    barrier_is_narrower: bool,
    peak_is_before_valley: bool,
    negative_differential_region: bool,
    overlap_closes_at_high_bias: bool,
    peak_matches_full_overlap: bool,
}

pub fn report() -> io::Result<CaseReport> {
    let report = evaluate();

    if !report.barrier_is_narrower
        || !report.peak_is_before_valley
        || !report.negative_differential_region
        || !report.overlap_closes_at_high_bias
        || !report.peak_matches_full_overlap
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "pn-junction-tunneling check failed: depletion-width or overlap-window checks did not hold",
        ));
    }

    Ok(CaseReport::new("pn-junction-tunneling")
        .with_answer(vec![
            ReportItem::text(
                "In this toy PN-junction tunneling model, heavy doping narrows the depletion region enough for a tunneling window that rises to a peak and then falls, producing a negative-differential region.",
            ),
            ReportItem::field("case", "pn-junction-tunneling"),
            ReportItem::field("peak bias", report.peak_bias.to_string()),
            ReportItem::field("peak current proxy", report.peak_current.to_string()),
            ReportItem::field(
                "negative differential region",
                yes_no(report.negative_differential_region),
            ),
        ])
        .with_reason_why(vec![
            ReportItem::text(
                "We model tunneling current as an exact overlap count between filled N-side states and empty P-side states while forward bias shifts the bands. Heavy doping is represented by a much narrower depletion region.",
            ),
            ReportItem::field(
                "ordinary depletion width (nm)",
                report.ordinary_depletion_width_nm.to_string(),
            ),
            ReportItem::field(
                "tunnel depletion width (nm)",
                report.tunnel_depletion_width_nm.to_string(),
            ),
            ReportItem::field(
                "filled N-side states",
                format_levels(report.n_filled_states),
            ),
            ReportItem::field(
                "empty P-side states at 0 bias",
                format_levels(report.p_empty_zero_bias),
            ),
            ReportItem::field(
                "bias -> overlap current proxy",
                format_curve(&BIAS_POINTS, &report.current_proxy),
            ),
            ReportItem::field("peak point", format!("{} -> {}", report.peak_bias, report.peak_current)),
            ReportItem::field(
                "high-bias point",
                format!("{} -> {}", report.valley_bias, report.valley_current),
            ),
        ])
        .with_check(vec![
            ReportItem::field(
                "heavily doped barrier is narrower",
                yes_no(report.barrier_is_narrower),
            ),
            ReportItem::field(
                "peak occurs before overlap closes",
                yes_no(report.peak_is_before_valley),
            ),
            ReportItem::field(
                "negative differential region present",
                yes_no(report.negative_differential_region),
            ),
            ReportItem::field(
                "high-bias overlap closes",
                yes_no(report.overlap_closes_at_high_bias),
            ),
            ReportItem::field(
                "peak equals full four-state overlap",
                yes_no(report.peak_matches_full_overlap),
            ),
        ]))
}

pub fn run_and_print() -> io::Result<()> {
    let report = evaluate();

    println!("=== Answer ===");
    println!(
        "In this toy PN-junction tunneling model, heavy doping narrows the depletion region enough for a tunneling window that rises to a peak and then falls, producing a negative-differential region."
    );
    println!("case                          : pn-junction-tunneling");
    println!("peak bias                     : {}", report.peak_bias);
    println!("peak current proxy            : {}", report.peak_current);
    println!(
        "negative differential region  : {}",
        yes_no(report.negative_differential_region)
    );
    println!();
    println!("=== Reason Why ===");
    println!(
        "We model tunneling current as an exact overlap count between filled N-side states and empty P-side states while forward bias shifts the bands. Heavy doping is represented by a much narrower depletion region."
    );
    println!(
        "ordinary depletion width (nm) : {}",
        report.ordinary_depletion_width_nm
    );
    println!(
        "tunnel depletion width (nm)   : {}",
        report.tunnel_depletion_width_nm
    );
    println!(
        "filled N-side states          : {}",
        format_levels(report.n_filled_states)
    );
    println!(
        "empty P-side states at 0 bias : {}",
        format_levels(report.p_empty_zero_bias)
    );
    println!(
        "bias -> overlap current proxy : {}",
        format_curve(&BIAS_POINTS, &report.current_proxy)
    );
    println!(
        "peak point                    : {} -> {}",
        report.peak_bias, report.peak_current
    );
    println!(
        "high-bias point               : {} -> {}",
        report.valley_bias, report.valley_current
    );
    println!();
    println!("=== Check ===");
    println!(
        "heavily doped barrier is narrower : {}",
        yes_no(report.barrier_is_narrower)
    );
    println!(
        "peak occurs before overlap closes : {}",
        yes_no(report.peak_is_before_valley)
    );
    println!(
        "negative differential region present : {}",
        yes_no(report.negative_differential_region)
    );
    println!(
        "high-bias overlap closes           : {}",
        yes_no(report.overlap_closes_at_high_bias)
    );
    println!(
        "peak equals full four-state overlap: {}",
        yes_no(report.peak_matches_full_overlap)
    );

    if !report.barrier_is_narrower
        || !report.peak_is_before_valley
        || !report.negative_differential_region
        || !report.overlap_closes_at_high_bias
        || !report.peak_matches_full_overlap
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "pn-junction-tunneling check failed: depletion-width or overlap-window checks did not hold",
        ));
    }

    Ok(())
}

fn evaluate() -> PnJunctionReport {
    let current_proxy = BIAS_POINTS.map(|bias| overlap_count(N_FILLED_STATES, shift_levels(P_EMPTY_ZERO_BIAS, bias)));

    let (peak_index, peak_current) = current_proxy
        .iter()
        .copied()
        .enumerate()
        .max_by_key(|&(_, current)| current)
        .expect("current proxy should not be empty");

    let valley_index = current_proxy.len() - 1;
    let valley_current = current_proxy[valley_index];

    PnJunctionReport {
        ordinary_depletion_width_nm: ORDINARY_DEPLETION_WIDTH_NM,
        tunnel_depletion_width_nm: TUNNEL_DEPLETION_WIDTH_NM,
        n_filled_states: N_FILLED_STATES,
        p_empty_zero_bias: P_EMPTY_ZERO_BIAS,
        current_proxy,
        peak_bias: BIAS_POINTS[peak_index],
        peak_current,
        valley_bias: BIAS_POINTS[valley_index],
        valley_current,
        barrier_is_narrower: TUNNEL_DEPLETION_WIDTH_NM < ORDINARY_DEPLETION_WIDTH_NM,
        peak_is_before_valley: peak_index < valley_index,
        negative_differential_region: current_proxy
            .as_slice()
            .windows(2)
            .skip(peak_index)
            .any(|pair| pair[1] < pair[0]),
        overlap_closes_at_high_bias: valley_current == 0,
        peak_matches_full_overlap: peak_current as usize == N_FILLED_STATES.len(),
    }
}

fn shift_levels(levels: [i32; 4], bias: i32) -> [i32; 4] {
    [
        levels[0] - bias,
        levels[1] - bias,
        levels[2] - bias,
        levels[3] - bias,
    ]
}

fn overlap_count(lhs: [i32; 4], rhs: [i32; 4]) -> u32 {
    lhs.into_iter().filter(|level| rhs.contains(level)).count() as u32
}

fn format_levels(levels: [i32; 4]) -> String {
    format!("[{}, {}, {}, {}]", levels[0], levels[1], levels[2], levels[3])
}

fn format_curve(biases: &[i32; 7], current_proxy: &[u32; 7]) -> String {
    biases
        .iter()
        .zip(current_proxy.iter())
        .map(|(bias, current)| format!("{bias}->{current}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlap_curve_has_the_expected_peak_and_shutdown() {
        let report = evaluate();

        assert_eq!(report.current_proxy, [2, 3, 4, 3, 2, 1, 0]);
        assert_eq!(report.peak_bias, 2);
        assert_eq!(report.peak_current, 4);
        assert_eq!(report.valley_bias, 6);
        assert_eq!(report.valley_current, 0);
        assert!(report.negative_differential_region);
        assert!(report.overlap_closes_at_high_bias);
    }

    #[test]
    fn pn_junction_tunneling_report_is_consistent() {
        let report = evaluate();

        assert!(report.barrier_is_narrower);
        assert!(report.peak_is_before_valley);
        assert!(report.negative_differential_region);
        assert!(report.overlap_closes_at_high_bias);
        assert!(report.peak_matches_full_overlap);
        assert_eq!(report.n_filled_states, [1, 2, 3, 4]);
        assert_eq!(report.p_empty_zero_bias, [3, 4, 5, 6]);
    }
}
