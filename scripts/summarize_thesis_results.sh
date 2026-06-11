#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
DATA="$ROOT/thesis-results/data"

metric_summary() {
    local file=$1
    local column=$2
    local label=$3
    local values
    values=$(tail -n +2 "$file" | cut -d, -f"$column" | sort -n)
    awk -v label="$label" '
        { values[NR]=$1; sum+=$1; sumsq+=$1*$1 }
        END {
            if (NR == 0) exit;
            median = (NR % 2) ? values[(NR+1)/2] : (values[NR/2]+values[NR/2+1])/2;
            variance = (NR > 1) ? (sumsq-sum*sum/NR)/(NR-1) : 0;
            printf "%s count=%d min=%.6f max=%.6f mean=%.6f median=%.6f stddev=%.6f\n",
                label, NR, values[1], values[NR], sum/NR, median, sqrt(variance);
        }
    ' <<<"$values"
}

{
    metric_summary "$DATA/preempt.csv" 5 preempt_latency_ms
    metric_summary "$DATA/rr-fairness.csv" 5 rr_max_min_ratio
    metric_summary "$DATA/rr-fairness.csv" 6 rr_jain_index

    awk -F, '
        NR > 1 {
            key=$1;
            n[key]++; elapsed[key]+=$3; switches[key]+=$8;
            not_ready[key]+=$4; yields[key]+=$5; blocks[key]+=$6; wakeups[key]+=$7;
        }
        END {
            for (key in n) {
                printf "waitpid version=%s runs=%d mean_elapsed_ms=%.3f mean_switches=%.3f mean_not_ready=%.3f mean_yields=%.3f mean_blocks=%.3f mean_wakeups=%.3f\n",
                    key, n[key], elapsed[key]/n[key], switches[key]/n[key],
                    not_ready[key]/n[key], yields[key]/n[key],
                    blocks[key]/n[key], wakeups[key]/n[key];
            }
        }
    ' "$DATA/waitpid.csv" | sort
} >"$DATA/statistical-summary.txt"

printf '%s\n' \
    'version,time_slice,tasks,runs,mean_elapsed_ms,stddev_elapsed_ms,mean_context_switches,stddev_context_switches,mean_avg_scan' \
    >"$DATA/baseline-comparison-summary.csv"

awk -F, '
    NR > 1 {
        key=$1 FS $2 FS $3;
        n[key]++;
        elapsed[key]+=$5; elapsed2[key]+=$5*$5;
        switches[key]+=$6; switches2[key]+=$6*$6;
        scans[key]+=$9;
    }
    END {
        for (key in n) {
            split(key, parts, FS);
            elapsed_var=(n[key]>1) ? (elapsed2[key]-elapsed[key]*elapsed[key]/n[key])/(n[key]-1) : 0;
            switch_var=(n[key]>1) ? (switches2[key]-switches[key]*switches[key]/n[key])/(n[key]-1) : 0;
            printf "%s,%s,%s,%d,%.3f,%.3f,%.3f,%.3f,%.6f\n",
                parts[1], parts[2], parts[3], n[key],
                elapsed[key]/n[key], sqrt(elapsed_var),
                switches[key]/n[key], sqrt(switch_var), scans[key]/n[key];
        }
    }
' "$DATA/baseline-comparison.csv" |
    sort -t, -k1,1 -k2,2n -k3,3n \
    >>"$DATA/baseline-comparison-summary.csv"

printf 'Wrote statistical summaries into %s\n' "$DATA"
