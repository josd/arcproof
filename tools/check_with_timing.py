#!/usr/bin/env python3
from __future__ import annotations

import statistics
import subprocess
import sys
import time
from pathlib import Path

EXAMPLES = [
    'collatz_1000',
    'control_system',
    'deep_taxonomy_100000',
    'delfour',
    'euler_identity',
    'fibonacci',
    'goldbach_1000',
    'gps',
    'kaprekar_6174',
    'matrix_mechanics',
    'odrl_dpv_ehds_risk_ranked',
    'path_discovery',
    'pn_junction_tunneling',
    'polynomial',
    'transistor_switch',
    'sudoku',
]

WARMUP_RUNS = 2
MIN_TIMED_RUNS = 9
MIN_TOTAL_TIMED_SECONDS = 0.250
MAX_TIMED_RUNS = 51


def run_once(binary: Path, verbose: bool) -> tuple[bool, float, str | None]:
    stdout = None if verbose else subprocess.DEVNULL
    stderr = None if verbose else subprocess.DEVNULL
    start = time.perf_counter_ns()
    try:
        completed = subprocess.run([str(binary)], stdout=stdout, stderr=stderr, check=False)
    except OSError as exc:
        return False, 0.0, str(exc)
    elapsed_s = (time.perf_counter_ns() - start) / 1_000_000_000.0
    if completed.returncode != 0:
        return False, elapsed_s, f'exit {completed.returncode}'
    return True, elapsed_s, None


def stable_time_ms(binary: Path, verbose: bool) -> tuple[str, float | None, str | None]:
    for _ in range(WARMUP_RUNS):
        ok, _, err = run_once(binary, verbose)
        if not ok:
            return 'FAIL', None, err

    samples: list[float] = []
    total = 0.0
    while len(samples) < MAX_TIMED_RUNS:
        ok, elapsed_s, err = run_once(binary, verbose)
        if not ok:
            return 'FAIL', None, err
        samples.append(elapsed_s)
        total += elapsed_s
        if len(samples) >= MIN_TIMED_RUNS and total >= MIN_TOTAL_TIMED_SECONDS:
            break

    return 'OK', statistics.median(samples) * 1000.0, None


def main(argv: list[str]) -> int:
    verbose = '--verbose' in argv
    root = Path(__file__).resolve().parent.parent
    bin_dir = root / 'bin'

    print('Stable timing report for standalone examples')
    print('status = OK if every warmup and timed run exited with code 0; FAIL otherwise')
    print(f'time (ms) = median wall-clock time over repeated quiet runs (warmups hidden)')
    print()
    print(f"{'example':<25}  {'status':<7}  {'time (ms)':>10}")
    print(f"{'-' * 25}  {'-' * 7}  {'-' * 10}")

    all_ok = True
    for name in EXAMPLES:
        binary = bin_dir / name
        if not binary.exists():
            print(f'{name:<25}  {"MISSING":<7}  {"-":>10}')
            all_ok = False
            continue
        status, ms, err = stable_time_ms(binary, verbose)
        if status == 'OK' and ms is not None:
            print(f'{name:<25}  {status:<7}  {ms:10.3f}')
        else:
            print(f'{name:<25}  {status:<7}  {"-":>10}')
            if err and verbose:
                print(f'  {err}')
            all_ok = False

    return 0 if all_ok else 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
