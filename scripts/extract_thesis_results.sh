#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
LOG_DIR="$ROOT/thesis-results/logs"
DATA_DIR="$ROOT/thesis-results/data"
mkdir -p "$DATA_DIR"

grep -hE '^(TEST|DATA|EVENT|RESULT|PASS|FAIL|SCHED_STATS)' \
    "$LOG_DIR"/*.log >"$DATA_DIR/all-events.txt" || true

printf '%s\n' 'run,period,expected_priority,actual_priority,pass' \
    >"$DATA_DIR/priority-map.csv"
for log in "$LOG_DIR"/E1-priority-map-run*.log; do
    [[ -e "$log" ]] || continue
    run=$(basename "$log" | sed -E 's/.*run([0-9]+).*/\1/')
    while read -r line; do
        period=$(sed -nE 's/.*period=([0-9]+).*/\1/p' <<<"$line")
        actual=$(sed -nE 's/.*priority=([0-9]+).*/\1/p' <<<"$line")
        expected=$(sed -nE 's/.*expected=([0-9]+).*/\1/p' <<<"$line")
        pass=false
        [[ "$actual" == "$expected" ]] && pass=true
        printf '%s,%s,%s,%s,%s\n' "$run" "$period" "$expected" "$actual" "$pass" \
            >>"$DATA_DIR/priority-map.csv"
    done < <(grep '^DATA period=.* expected=' "$log")
done

printf '%s\n' 'run,observed_period_order,pass' >"$DATA_DIR/ready-order.csv"
for log in "$LOG_DIR"/E2-ready-order-run*.log; do
    [[ -e "$log" ]] || continue
    run=$(basename "$log" | sed -E 's/.*run([0-9]+).*/\1/')
    order=$(grep '^EVENT type=first_run ' "$log" |
        sed -nE 's/.*period=([0-9]+).*/\1/p' |
        paste -sd, -)
    pass=false
    [[ "$order" == '100,400,900' ]] && pass=true
    printf '%s,"%s",%s\n' "$run" "$order" "$pass" >>"$DATA_DIR/ready-order.csv"
done

printf '%s\n' \
    'run,create_time_ms,high_start_ms,low_finish_ms,latency_ms,priority_preemptions,pass' \
    >"$DATA_DIR/preempt.csv"

for log in "$LOG_DIR"/E3-preempt-run*.log; do
    [[ -e "$log" ]] || continue
    run=$(basename "$log" | sed -E 's/.*run([0-9]+).*/\1/')
    result=$(grep '^RESULT create_time_ms=' "$log" | tail -n 1 || true)
    stats=$(grep '^SCHED_STATS ' "$log" | tail -n 1 || true)
    pass=false
    grep -q '^PASS name=rms_preempt_strict' "$log" && pass=true
    create=$(sed -nE 's/.*create_time_ms=([0-9-]+).*/\1/p' <<<"$result")
    high=$(sed -nE 's/.*high_start_ms=([0-9-]+).*/\1/p' <<<"$result")
    low=$(sed -nE 's/.*low_finish_ms=([0-9-]+).*/\1/p' <<<"$result")
    latency=$(sed -nE 's/.*latency_ms=([0-9-]+).*/\1/p' <<<"$result")
    preempt=$(sed -nE 's/.*prio_preempt=([0-9]+).*/\1/p' <<<"$stats")
    printf '%s,%s,%s,%s,%s,%s,%s\n' \
        "$run" "$create" "$high" "$low" "$latency" "$preempt" "$pass" \
        >>"$DATA_DIR/preempt.csv"
done

printf '%s\n' \
    'run,task1_units,task2_units,task3_units,max_min_ratio,jain_index,timeslice_preemptions' \
    >"$DATA_DIR/rr-fairness.csv"

for log in "$LOG_DIR"/E4-rr-run*.log; do
    [[ -e "$log" ]] || continue
    run=$(basename "$log" | sed -E 's/.*run([0-9]+).*/\1/')
    mapfile -t units < <(
        grep '^RESULT type=worker ' "$log" |
            sed -nE 's/.*work_units=([0-9]+).*/\1/p' |
            head -n 3
    )
    [[ ${#units[@]} -eq 3 ]] || continue
    stats=$(grep '^SCHED_STATS ' "$log" | tail -n 1 || true)
    ts=$(sed -nE 's/.*ts_preempt=([0-9]+).*/\1/p' <<<"$stats")
    metrics=$(awk -v a="${units[0]}" -v b="${units[1]}" -v c="${units[2]}" '
        BEGIN {
            min=a; if (b<min) min=b; if (c<min) min=c;
            max=a; if (b>max) max=b; if (c>max) max=c;
            sum=a+b+c; squares=a*a+b*b+c*c;
            printf "%.6f,%.6f", max/min, (sum*sum)/(3*squares);
        }')
    printf '%s,%s,%s,%s,%s,%s\n' \
        "$run" "${units[0]}" "${units[1]}" "${units[2]}" "$metrics" "$ts" \
        >>"$DATA_DIR/rr-fairness.csv"
done

printf '%s\n' \
    'version,time_slice,tasks,run,elapsed_ms,context_switches,fetch_calls,queue_levels_scanned,avg_scan' \
    >"$DATA_DIR/baseline-comparison.csv"

for log in "$LOG_DIR"/E7-*-run*.log; do
    [[ -e "$log" ]] || continue
    name=$(basename "$log")
    case "$name" in
        E7-baseline-*) version=baseline; time_slice=1 ;;
        E7-improved-ts1-*) version=improved; time_slice=1 ;;
        E7-improved-ts2-*) version=improved; time_slice=2 ;;
        *) continue ;;
    esac
    run=$(sed -nE 's/.*run([0-9]+).*/\1/p' <<<"$name")
    mapfile -t cases < <(grep '^RESULT type=case ' "$log")
    mapfile -t stat_lines < <(grep '^SCHED_STATS ' "$log")
    count=${#cases[@]}
    [[ ${#stat_lines[@]} -lt $count ]] && count=${#stat_lines[@]}
    for ((i = 0; i < count; i++)); do
        tasks=$(sed -nE 's/.*tasks=([0-9]+).*/\1/p' <<<"${cases[$i]}")
        elapsed=$(sed -nE 's/.*elapsed_ms=([0-9]+).*/\1/p' <<<"${cases[$i]}")
        switches=$(sed -nE 's/.*switch=([0-9]+).*/\1/p' <<<"${stat_lines[$i]}")
        fetch=$(sed -nE 's/.*fetch=([0-9]+).*/\1/p' <<<"${stat_lines[$i]}")
        scan=$(sed -nE 's/.*scan=([0-9]+).*/\1/p' <<<"${stat_lines[$i]}")
        avg=$(awk -v scan="$scan" -v fetch="$fetch" \
            'BEGIN { if (fetch == 0) print "0"; else printf "%.6f", scan/fetch }')
        printf '%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
            "$version" "$time_slice" "$tasks" "$run" "$elapsed" \
            "$switches" "$fetch" "$scan" "$avg" \
            >>"$DATA_DIR/baseline-comparison.csv"
    done
done

printf '%s\n' \
    'version,run,elapsed_ms,waitpid_not_ready,wait_yields,waitpid_blocks,wakeups,context_switches' \
    >"$DATA_DIR/waitpid.csv"

for log in "$LOG_DIR"/E6-waitpid-common-*-run*.log; do
    [[ -e "$log" ]] || continue
    name=$(basename "$log")
    case "$name" in
        *-baseline-*) version=baseline ;;
        *-improved-*) version=improved ;;
        *) continue ;;
    esac
    run=$(sed -nE 's/.*run([0-9]+).*/\1/p' <<<"$name")
    result=$(grep '^RESULT waited_pid=' "$log" | tail -n 1 || true)
    stats=$(grep '^SCHED_STATS ' "$log" | tail -n 1 || true)
    elapsed=$(sed -nE 's/.*elapsed_ms=([0-9]+).*/\1/p' <<<"$result")
    not_ready=$(sed -nE 's/.*wait_not_ready=([0-9]+).*/\1/p' <<<"$stats")
    wait_yield=$(sed -nE 's/.*wait_yield=([0-9]+).*/\1/p' <<<"$stats")
    blocks=$(sed -nE 's/.*wait_block=([0-9]+).*/\1/p' <<<"$stats")
    wakeups=$(sed -nE 's/.*wakeup=([0-9]+).*/\1/p' <<<"$stats")
    switches=$(sed -nE 's/.*switch=([0-9]+).*/\1/p' <<<"$stats")
    printf '%s,%s,%s,%s,%s,%s,%s,%s\n' \
        "$version" "$run" "$elapsed" "${not_ready:-0}" "${wait_yield:-0}" \
        "${blocks:-0}" "${wakeups:-0}" "$switches" \
        >>"$DATA_DIR/waitpid.csv"
done

printf 'Extracted data into %s\n' "$DATA_DIR"
