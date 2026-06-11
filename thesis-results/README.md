# Thesis Experiment Results

This directory keeps raw logs, extracted CSV data, environment snapshots, and
the exact experiment sources used for the thesis scheduler evaluation.

## Workflow

```bash
./scripts/collect_thesis_env.sh
./scripts/run_qemu_test.sh rms_priority_map E1-priority-map-run01 3
./scripts/run_qemu_test.sh rms_ready_order E2-ready-order-run01 3
./scripts/run_qemu_test.sh rms_preempt_strict E3-preempt-run01 3
./scripts/run_qemu_test.sh rms_rr_fairness E4-rr-run01 6
./scripts/run_qemu_test.sh rms_slice_trace E5-slice-run01 4
./scripts/run_qemu_test.sh rms_waitpid_block E6-waitpid-run01 4
./scripts/extract_thesis_results.sh
./scripts/summarize_thesis_results.sh
```

Use the same host and QEMU version for all baseline and improved runs. Keep
every raw log, including outliers and failed runs.

The E7 comparison uses:

- `baseline`: detached `origin/ch5` worktree at `/home/siyun/rCore-baseline`
- `improved,time_slice=1`: `make -C os build CARGO_FEATURES=timeslice-1`
- `improved,time_slice=2`: default `make -C os build`

Key generated tables are `data/preempt.csv`, `data/rr-fairness.csv`,
`data/waitpid.csv`, `data/baseline-comparison.csv`, and their statistical
summaries.
