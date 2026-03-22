# Latest Benchmark Snapshot

Updated: 2026-03-22T03:12:28.644Z

This file is the small release-friendly benchmark snapshot. Raw JSON stays in `benchmark/results/`.

Combined report: [benchmark-2026-03-22T03-12-28-644Z.md](results/benchmark-2026-03-22T03-12-28-644Z.md)

## Compare

Relative to `antfu`, `hni` averaged `1.66x`.

Artifacts: [compare-2026-03-22T01-42-38-524Z.md](results/compare-2026-03-22T01-42-38-524Z.md) · [compare-2026-03-22T01-42-38-524Z.json](results/compare-2026-03-22T01-42-38-524Z.json)

| Case | antfu | hni | Relative |
| --- | ---: | ---: | ---: |
| ni --version | 334.60 ms | 129.05 ms | 2.59x |
| ni vite ? (npm) | 7.69 ms | 5.25 ms | 1.47x |
| nr build ? (pnpm) | 8.13 ms | 4.84 ms | 1.68x |
| nlx vitest ? (npm) | 7.76 ms | 6.60 ms | 1.18x |

## Native

Relative to `delegated`, `native` averaged `4.38x`.

Artifacts: [native-2026-03-22T02-34-12-022Z.md](results/native-2026-03-22T02-34-12-022Z.md) · [native-2026-03-22T02-34-12-022Z.json](results/native-2026-03-22T02-34-12-022Z.json)

| Case | delegated | native | Relative |
| --- | ---: | ---: | ---: |
| nr noop (npm) | 320.51 ms | 37.93 ms | 8.45x |
| nr hooks (npm) | 717.95 ms | 99.55 ms | 7.21x |
| node run noop (npm) | 369.93 ms | 42.44 ms | 8.72x |
| nr noop (pnpm) | 523.58 ms | 40.39 ms | 12.96x |
| nr hooks (pnpm) | 751.62 ms | 96.77 ms | 7.77x |
| node run noop (pnpm) | 471.04 ms | 30.59 ms | 15.40x |
| nr noop (yarn) | 287.91 ms | 38.47 ms | 7.48x |
| nr hooks (yarn) | 366.16 ms | 86.23 ms | 4.25x |
| node run noop (yarn) | 274.60 ms | 36.95 ms | 7.43x |
| nr noop (bun) | 48.67 ms | 36.06 ms | 1.35x |
| nr hooks (bun) | 108.44 ms | 86.44 ms | 1.25x |
| node run noop (bun) | 42.84 ms | 32.40 ms | 1.32x |
| nr noop (deno) | 34.29 ms | 34.68 ms | 0.99x |
| nr hooks (deno) | 35.18 ms | 34.86 ms | 1.01x |
| node run noop (deno) | 34.52 ms | 34.73 ms | 0.99x |
| nlx hello --flag (npm local bin) | 334.43 ms | 7.75 ms | 43.14x |

## Runtime

Relative to `hni`, `bun` averaged `0.75x`. Relative to `hni`, `deno` averaged `1.47x`.

Artifacts: [runtime-2026-03-22T02-37-34-419Z.md](results/runtime-2026-03-22T02-37-34-419Z.md) · [runtime-2026-03-22T02-37-34-419Z.json](results/runtime-2026-03-22T02-37-34-419Z.json)

| Case | hni | bun | deno |
| --- | ---: | ---: | ---: |
| task noop | 31.74 ms | 44.72 ms | 35.53 ms |
| task hooks | 88.14 ms | 110.60 ms | 36.25 ms |

## Direct

Relative to `direct`, `hni` averaged `4.90x`.

Artifacts: [direct-2026-03-22T03-12-28-644Z.md](results/direct-2026-03-22T03-12-28-644Z.md) · [direct-2026-03-22T03-12-28-644Z.json](results/direct-2026-03-22T03-12-28-644Z.json)

| Case | direct | hni | Relative |
| --- | ---: | ---: | ---: |
| task noop (npm) | 207.83 ms | 36.63 ms | 5.67x |
| task hooks (npm) | 404.74 ms | 85.62 ms | 4.73x |
| exec hello --flag (npm) | 249.48 ms | 10.25 ms | 24.35x |
| task noop (pnpm) | 418.28 ms | 32.26 ms | 12.97x |
| task hooks (pnpm) | 548.16 ms | 85.70 ms | 6.40x |
| exec hello --flag (pnpm) | 311.16 ms | 7.30 ms | 42.65x |
| task noop (yarn) | 279.87 ms | 32.17 ms | 8.70x |
| task hooks (yarn) | 359.87 ms | 85.77 ms | 4.20x |
| exec hello --flag (yarn) | 104.92 ms | 7.48 ms | 14.02x |
| task noop (bun) | 45.49 ms | 32.22 ms | 1.41x |
| task hooks (bun) | 109.17 ms | 86.38 ms | 1.26x |
| exec hello --flag (bun) | 15.07 ms | 7.46 ms | 2.02x |
| task noop (deno) | 36.75 ms | 35.42 ms | 1.04x |
| task hooks (deno) | 35.84 ms | 34.90 ms | 1.03x |

