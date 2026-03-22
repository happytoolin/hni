# Latest Benchmark Snapshot

Updated: 2026-03-22T00:32:35.442Z

This file is the small release-friendly benchmark snapshot. Raw JSON stays in `benchmark/results/`.

Combined report: [benchmark-2026-03-22T00-32-35-442Z.md](results/benchmark-2026-03-22T00-32-35-442Z.md)

## Compare

Relative to `antfu`, `hni` averaged `1.46x`.

Artifacts: [compare-2026-03-22T00-28-58-739Z.md](results/compare-2026-03-22T00-28-58-739Z.md) · [compare-2026-03-22T00-28-58-739Z.json](results/compare-2026-03-22T00-28-58-739Z.json)

| Case | antfu | hni | Relative |
| --- | ---: | ---: | ---: |
| ni --version | 451.62 ms | 336.74 ms | 1.34x |
| ni vite ? (npm) | 6.91 ms | 11.60 ms | 0.60x |
| nr build ? (pnpm) | 9.26 ms | 2.46 ms | 3.77x |
| nlx vitest ? (npm) | 6.74 ms | 4.48 ms | 1.51x |

## Native

Relative to `delegated`, `native` averaged `3.65x`.

Artifacts: [native-2026-03-22T00-32-22-022Z.md](results/native-2026-03-22T00-32-22-022Z.md) · [native-2026-03-22T00-32-22-022Z.json](results/native-2026-03-22T00-32-22-022Z.json)

| Case | delegated | native | Relative |
| --- | ---: | ---: | ---: |
| nr noop (npm) | 399.19 ms | 55.25 ms | 7.23x |
| nr hooks (npm) | 921.61 ms | 171.40 ms | 5.38x |
| node run noop (npm) | 600.81 ms | 171.54 ms | 3.50x |
| nr noop (pnpm) | 922.51 ms | 77.18 ms | 11.95x |
| nr hooks (pnpm) | 967.81 ms | 129.91 ms | 7.45x |
| node run noop (pnpm) | 741.88 ms | 70.09 ms | 10.58x |
| nr noop (yarn) | 366.27 ms | 49.95 ms | 7.33x |
| nr hooks (yarn) | 395.44 ms | 103.77 ms | 3.81x |
| node run noop (yarn) | 307.60 ms | 55.06 ms | 5.59x |
| nr noop (bun) | 48.41 ms | 39.08 ms | 1.24x |
| nr hooks (bun) | 113.05 ms | 126.80 ms | 0.89x |
| node run noop (bun) | 55.13 ms | 40.16 ms | 1.37x |
| nr noop (deno) | 46.77 ms | 43.22 ms | 1.08x |
| nr hooks (deno) | 41.81 ms | 64.34 ms | 0.65x |
| node run noop (deno) | 40.05 ms | 40.75 ms | 0.98x |
| nlx hello --flag (npm local bin) | 363.95 ms | 7.55 ms | 48.21x |

## Runtime

Relative to `hni`, `bun` averaged `0.67x`. Relative to `hni`, `deno` averaged `1.30x`.

Artifacts: [runtime-2026-03-22T00-32-35-442Z.md](results/runtime-2026-03-22T00-32-35-442Z.md) · [runtime-2026-03-22T00-32-35-442Z.json](results/runtime-2026-03-22T00-32-35-442Z.json)

| Case | hni | bun | deno |
| --- | ---: | ---: | ---: |
| task noop | 43.51 ms | 50.80 ms | 39.68 ms |
| task hooks | 107.08 ms | 203.35 ms | 69.51 ms |

