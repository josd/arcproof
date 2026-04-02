use std::fmt;
use std::io;

use crate::report::{CaseReport, ReportItem};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Matrix2 {
    a11: i32,
    a12: i32,
    a21: i32,
    a22: i32,
}

impl Matrix2 {
    const fn new(a11: i32, a12: i32, a21: i32, a22: i32) -> Self {
        Self {
            a11,
            a12,
            a21,
            a22,
        }
    }

    const fn identity() -> Self {
        Self::new(1, 0, 0, 1)
    }

    fn mul(self, other: Self) -> Self {
        Self::new(
            self.a11 * other.a11 + self.a12 * other.a21,
            self.a11 * other.a12 + self.a12 * other.a22,
            self.a21 * other.a11 + self.a22 * other.a21,
            self.a21 * other.a12 + self.a22 * other.a22,
        )
    }

    fn sub(self, other: Self) -> Self {
        Self::new(
            self.a11 - other.a11,
            self.a12 - other.a12,
            self.a21 - other.a21,
            self.a22 - other.a22,
        )
    }

    fn trace(self) -> i32 {
        self.a11 + self.a22
    }

    fn determinant(self) -> i32 {
        self.a11 * self.a22 - self.a12 * self.a21
    }

    fn apply(self, vector: [i32; 2]) -> [i32; 2] {
        [
            self.a11 * vector[0] + self.a12 * vector[1],
            self.a21 * vector[0] + self.a22 * vector[1],
        ]
    }

    fn is_zero(self) -> bool {
        self == Self::new(0, 0, 0, 0)
    }
}

impl fmt::Display for Matrix2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[[{}, {}], [{}, {}]]",
            self.a11, self.a12, self.a21, self.a22
        )
    }
}

struct MatrixMechanicsReport {
    hamiltonian: Matrix2,
    observable_x: Matrix2,
    h_times_x: Matrix2,
    x_times_h: Matrix2,
    commutator: Matrix2,
    energy_levels: [i32; 2],
    h_on_e1: [i32; 2],
    h_on_e2: [i32; 2],
    x_on_e1: [i32; 2],
    x_on_e2: [i32; 2],
    commutator_is_nonzero: bool,
    trace_matches_levels: bool,
    determinant_matches_levels: bool,
    e1_is_eigenvector: bool,
    e2_is_eigenvector: bool,
    x_is_involution: bool,
}

pub fn report() -> io::Result<CaseReport> {
    let report = evaluate();

    if !report.commutator_is_nonzero
        || !report.trace_matches_levels
        || !report.determinant_matches_levels
        || !report.e1_is_eigenvector
        || !report.e2_is_eigenvector
        || !report.x_is_involution
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "matrix-mechanics check failed: commutator, spectrum, or basis checks did not hold",
        ));
    }

    Ok(CaseReport::new("matrix-mechanics")
        .with_answer(vec![
            ReportItem::text(
                "In this toy matrix-mechanics model, the Hamiltonian has two discrete energy levels and does not commute with a second observable.",
            ),
            ReportItem::field("case", "matrix-mechanics"),
            ReportItem::field("energy levels", format_levels(report.energy_levels)),
            ReportItem::field(
                "commutator nonzero",
                yes_no(report.commutator_is_nonzero),
            ),
        ])
        .with_reason_why(vec![
            ReportItem::text(
                "We represent observables as exact 2x2 matrices, multiply them in both orders, and compare the products. The Hamiltonian is diagonal in the chosen basis, while X swaps the two basis states.",
            ),
            ReportItem::field("H", report.hamiltonian.to_string()),
            ReportItem::field("X", report.observable_x.to_string()),
            ReportItem::field("HX", report.h_times_x.to_string()),
            ReportItem::field("XH", report.x_times_h.to_string()),
            ReportItem::field("[H, X]", report.commutator.to_string()),
            ReportItem::field("H e1", format_vector(report.h_on_e1)),
            ReportItem::field("H e2", format_vector(report.h_on_e2)),
            ReportItem::field("X e1", format_vector(report.x_on_e1)),
            ReportItem::field("X e2", format_vector(report.x_on_e2)),
        ])
        .with_check(vec![
            ReportItem::field("trace matches levels", yes_no(report.trace_matches_levels)),
            ReportItem::field(
                "determinant matches levels",
                yes_no(report.determinant_matches_levels),
            ),
            ReportItem::field("e1 eigenvector check", yes_no(report.e1_is_eigenvector)),
            ReportItem::field("e2 eigenvector check", yes_no(report.e2_is_eigenvector)),
            ReportItem::field("X^2 = I", yes_no(report.x_is_involution)),
            ReportItem::field("[H, X] != 0", yes_no(report.commutator_is_nonzero)),
        ]))
}

pub fn run_and_print() -> io::Result<()> {
    let report = evaluate();

    println!("=== Answer ===");
    println!(
        "In this toy matrix-mechanics model, the Hamiltonian has two discrete energy levels and does not commute with a second observable."
    );
    println!("case                : matrix-mechanics");
    println!("energy levels       : {}", format_levels(report.energy_levels));
    println!(
        "commutator nonzero  : {}",
        yes_no(report.commutator_is_nonzero)
    );
    println!();
    println!("=== Reason Why ===");
    println!(
        "We represent observables as exact 2x2 matrices, multiply them in both orders, and compare the products. The Hamiltonian is diagonal in the chosen basis, while X swaps the two basis states."
    );
    println!("H                   : {}", report.hamiltonian);
    println!("X                   : {}", report.observable_x);
    println!("HX                  : {}", report.h_times_x);
    println!("XH                  : {}", report.x_times_h);
    println!("[H, X]              : {}", report.commutator);
    println!("H e1                : {}", format_vector(report.h_on_e1));
    println!("H e2                : {}", format_vector(report.h_on_e2));
    println!("X e1                : {}", format_vector(report.x_on_e1));
    println!("X e2                : {}", format_vector(report.x_on_e2));
    println!();
    println!("=== Check ===");
    println!(
        "trace matches levels        : {}",
        yes_no(report.trace_matches_levels)
    );
    println!(
        "determinant matches levels  : {}",
        yes_no(report.determinant_matches_levels)
    );
    println!(
        "e1 eigenvector check        : {}",
        yes_no(report.e1_is_eigenvector)
    );
    println!(
        "e2 eigenvector check        : {}",
        yes_no(report.e2_is_eigenvector)
    );
    println!("X^2 = I                     : {}", yes_no(report.x_is_involution));
    println!("[H, X] != 0                : {}", yes_no(report.commutator_is_nonzero));

    if !report.commutator_is_nonzero
        || !report.trace_matches_levels
        || !report.determinant_matches_levels
        || !report.e1_is_eigenvector
        || !report.e2_is_eigenvector
        || !report.x_is_involution
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "matrix-mechanics check failed: commutator, spectrum, or basis checks did not hold",
        ));
    }

    Ok(())
}

fn evaluate() -> MatrixMechanicsReport {
    let hamiltonian = Matrix2::new(1, 0, 0, 2);
    let observable_x = Matrix2::new(0, 1, 1, 0);

    let h_times_x = hamiltonian.mul(observable_x);
    let x_times_h = observable_x.mul(hamiltonian);
    let commutator = h_times_x.sub(x_times_h);

    let energy_levels = [1, 2];
    let e1 = [1, 0];
    let e2 = [0, 1];
    let h_on_e1 = hamiltonian.apply(e1);
    let h_on_e2 = hamiltonian.apply(e2);
    let x_on_e1 = observable_x.apply(e1);
    let x_on_e2 = observable_x.apply(e2);

    MatrixMechanicsReport {
        hamiltonian,
        observable_x,
        h_times_x,
        x_times_h,
        commutator,
        energy_levels,
        h_on_e1,
        h_on_e2,
        x_on_e1,
        x_on_e2,
        commutator_is_nonzero: !commutator.is_zero(),
        trace_matches_levels: hamiltonian.trace() == energy_levels.iter().copied().sum::<i32>(),
        determinant_matches_levels: hamiltonian.determinant() == energy_levels.iter().copied().product::<i32>(),
        e1_is_eigenvector: h_on_e1 == [1, 0],
        e2_is_eigenvector: h_on_e2 == [0, 2],
        x_is_involution: observable_x.mul(observable_x) == Matrix2::identity(),
    }
}

fn format_vector(vector: [i32; 2]) -> String {
    format!("[{}, {}]", vector[0], vector[1])
}

fn format_levels(levels: [i32; 2]) -> String {
    format!("{}, {}", levels[0], levels[1])
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_mechanics_report_is_consistent() {
        let report = evaluate();

        assert_eq!(report.hamiltonian, Matrix2::new(1, 0, 0, 2));
        assert_eq!(report.observable_x, Matrix2::new(0, 1, 1, 0));
        assert_eq!(report.h_times_x, Matrix2::new(0, 1, 2, 0));
        assert_eq!(report.x_times_h, Matrix2::new(0, 2, 1, 0));
        assert_eq!(report.commutator, Matrix2::new(0, -1, 1, 0));
        assert_eq!(report.energy_levels, [1, 2]);
        assert_eq!(report.h_on_e1, [1, 0]);
        assert_eq!(report.h_on_e2, [0, 2]);
        assert_eq!(report.x_on_e1, [0, 1]);
        assert_eq!(report.x_on_e2, [1, 0]);
        assert!(report.commutator_is_nonzero);
        assert!(report.trace_matches_levels);
        assert!(report.determinant_matches_levels);
        assert!(report.e1_is_eigenvector);
        assert!(report.e2_is_eigenvector);
        assert!(report.x_is_involution);
    }
}
