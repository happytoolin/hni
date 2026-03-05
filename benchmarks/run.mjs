#!/usr/bin/env node

import { execFileSync, spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import process from 'node:process'

const DEFAULT_RUNS = 20
const DEFAULT_WARMUPS = 5

const PMS = [
  {
    id: 'npm',
    label: 'npm lock',
    placeholder: '<npmFixture>',
    requiredBins: ['npm', 'npx'],
  },
  {
    id: 'pnpm',
    label: 'pnpm lock',
    placeholder: '<pnpmFixture>',
    requiredBins: ['pnpm'],
  },
  {
    id: 'yarn',
    label: 'yarn lock',
    placeholder: '<yarnFixture>',
    requiredBins: ['yarn'],
  },
  {
    id: 'bun',
    label: 'bun lock',
    placeholder: '<bunFixture>',
    requiredBins: ['bun'],
  },
]

const NI_VARIANTS = [
  { id: 'base', label: 'ni ?', args: [] },
  { id: 'vite', label: 'ni vite ?', args: ['vite'] },
  { id: 'devdep', label: 'ni @types/node -D ?', args: ['@types/node', '-D'] },
  { id: 'prod', label: 'ni -P ?', args: ['-P'] },
  { id: 'global', label: 'ni -g eslint ?', args: ['-g', 'eslint'] },
  { id: 'frozen', label: 'ni --frozen ?', args: ['--frozen'] },
  { id: 'frozen_if_present', label: 'ni --frozen-if-present ?', args: ['--frozen-if-present'] },
]

const NR_VARIANTS = [
  { id: 'dev_port', label: 'nr dev --port=3000 ?', args: ['dev', '--port=3000'] },
  {
    id: 'if_present_missing',
    label: 'nr --if-present missing-script ?',
    args: ['--if-present', 'missing-script'],
  },
  { id: 'build', label: 'nr build ?', args: ['build'] },
]

const NLX_VARIANTS = [
  { id: 'vitest', label: 'nlx vitest ?', args: ['vitest'] },
  { id: 'vitest_help', label: 'nlx vitest --help ?', args: ['vitest', '--help'] },
]

const NUN_VARIANTS = [{ id: 'webpack', label: 'nun webpack ?', args: ['webpack'] }]

function makeCase({
  id,
  group,
  name,
  antfuBin,
  oursBin,
  antfuArgs,
  oursArgs,
  requiredBins = [],
}) {
  return {
    id,
    group,
    name,
    antfu: { bin: antfuBin, args: antfuArgs },
    ours: { bin: oursBin, args: oursArgs },
    requiredBins,
  }
}

function buildCases() {
  const cases = []

  cases.push(
    makeCase({
      id: 'startup_ni_version',
      group: 'startup',
      name: 'ni --version',
      antfuBin: 'ni',
      oursBin: 'ni',
      antfuArgs: ['--version'],
      oursArgs: ['--version'],
    }),
  )

  for (const pm of PMS) {
    for (const variant of NI_VARIANTS) {
      const args = ['-C', pm.placeholder, ...variant.args, '?']
      cases.push(
        makeCase({
          id: `${pm.id}_ni_${variant.id}`,
          group: 'ni',
          name: `${variant.label} (${pm.label})`,
          antfuBin: 'ni',
          oursBin: 'ni',
          antfuArgs: args,
          oursArgs: args,
          requiredBins: pm.requiredBins,
        }),
      )
    }

    for (const variant of NR_VARIANTS) {
      const args = ['-C', pm.placeholder, ...variant.args, '?']
      cases.push(
        makeCase({
          id: `${pm.id}_nr_${variant.id}`,
          group: 'nr',
          name: `${variant.label} (${pm.label})`,
          antfuBin: 'nr',
          oursBin: 'nr',
          antfuArgs: args,
          oursArgs: args,
          requiredBins: pm.requiredBins,
        }),
      )
    }

    for (const variant of NLX_VARIANTS) {
      const args = ['-C', pm.placeholder, ...variant.args, '?']
      cases.push(
        makeCase({
          id: `${pm.id}_nlx_${variant.id}`,
          group: 'nlx',
          name: `${variant.label} (${pm.label})`,
          antfuBin: 'nlx',
          oursBin: 'nlx',
          antfuArgs: args,
          oursArgs: args,
          requiredBins: pm.requiredBins,
        }),
      )
    }

    for (const variant of NUN_VARIANTS) {
      const args = ['-C', pm.placeholder, ...variant.args, '?']
      cases.push(
        makeCase({
          id: `${pm.id}_nun_${variant.id}`,
          group: 'nun',
          name: `${variant.label} (${pm.label})`,
          antfuBin: 'nun',
          oursBin: 'nun',
          antfuArgs: args,
          oursArgs: args,
          requiredBins: pm.requiredBins,
        }),
      )
    }

    {
      const args = ['-C', pm.placeholder, '?']
      cases.push(
        makeCase({
          id: `${pm.id}_nci_base`,
          group: 'nci',
          name: `nci ? (${pm.label})`,
          antfuBin: 'nci',
          oursBin: 'nci',
          antfuArgs: args,
          oursArgs: args,
          requiredBins: pm.requiredBins,
        }),
      )
    }

    {
      const args = ['-C', pm.placeholder, '?']
      cases.push(
        makeCase({
          id: `${pm.id}_nup_base`,
          group: 'nup',
          name: `nup ? (${pm.label})`,
          antfuBin: 'nup',
          oursBin: 'nu',
          antfuArgs: args,
          oursArgs: args,
          requiredBins: pm.requiredBins,
        }),
      )
    }

  }

  cases.push(
    makeCase({
      id: 'pnpm_nup_interactive',
      group: 'nup',
      name: 'nup -i ? (pnpm lock)',
      antfuBin: 'nup',
      oursBin: 'nu',
      antfuArgs: ['-C', '<pnpmFixture>', '-i', '?'],
      oursArgs: ['-C', '<pnpmFixture>', '-i', '?'],
      requiredBins: ['pnpm'],
    }),
  )

  cases.push(
    makeCase({
      id: 'pnpm_workspace_ni_base',
      group: 'workspace',
      name: 'ni ? (pnpm workspace subpkg)',
      antfuBin: 'ni',
      oursBin: 'ni',
      antfuArgs: ['-C', '<pnpmWorkspaceFixture>', '?'],
      oursArgs: ['-C', '<pnpmWorkspaceFixture>', '?'],
      requiredBins: ['pnpm'],
    }),
  )

  cases.push(
    makeCase({
      id: 'pnpm_workspace_nr_dev',
      group: 'workspace',
      name: 'nr dev ? (pnpm workspace subpkg)',
      antfuBin: 'nr',
      oursBin: 'nr',
      antfuArgs: ['-C', '<pnpmWorkspaceFixture>', 'dev', '?'],
      oursArgs: ['-C', '<pnpmWorkspaceFixture>', 'dev', '?'],
      requiredBins: ['pnpm'],
    }),
  )

  cases.push(
    makeCase({
      id: 'pnpm_workspace_nup_base',
      group: 'workspace',
      name: 'nup ? (pnpm workspace subpkg)',
      antfuBin: 'nup',
      oursBin: 'nu',
      antfuArgs: ['-C', '<pnpmWorkspaceFixture>', '?'],
      oursArgs: ['-C', '<pnpmWorkspaceFixture>', '?'],
      requiredBins: ['pnpm'],
    }),
  )

  cases.push(
    makeCase({
      id: 'pnpm_workspace_nup_interactive',
      group: 'workspace',
      name: 'nup -i ? (pnpm workspace subpkg)',
      antfuBin: 'nup',
      oursBin: 'nu',
      antfuArgs: ['-C', '<pnpmWorkspaceFixture>', '-i', '?'],
      oursArgs: ['-C', '<pnpmWorkspaceFixture>', '-i', '?'],
      requiredBins: ['pnpm'],
    }),
  )

  return cases
}

function parseArgs(argv) {
  const args = {
    runs: DEFAULT_RUNS,
    warmups: DEFAULT_WARMUPS,
    build: true,
    outputJson: true,
    outputMarkdown: true,
    minSpeedup: null,
  }

  for (const raw of argv) {
    if (raw === '--no-build') {
      args.build = false
      continue
    }

    if (raw === '--no-json') {
      args.outputJson = false
      continue
    }

    if (raw === '--no-markdown') {
      args.outputMarkdown = false
      continue
    }

    if (raw.startsWith('--runs=')) {
      args.runs = Number(raw.split('=')[1])
      continue
    }

    if (raw.startsWith('--warmups=')) {
      args.warmups = Number(raw.split('=')[1])
      continue
    }

    if (raw.startsWith('--min-speedup=')) {
      args.minSpeedup = Number(raw.split('=')[1])
      continue
    }
  }

  if (!Number.isInteger(args.runs) || args.runs <= 0) {
    throw new Error('--runs must be a positive integer')
  }

  if (!Number.isInteger(args.warmups) || args.warmups < 0) {
    throw new Error('--warmups must be a non-negative integer')
  }

  if (args.minSpeedup !== null && (!Number.isFinite(args.minSpeedup) || args.minSpeedup <= 0)) {
    throw new Error('--min-speedup must be a positive number')
  }

  return args
}

function run(cmd, cmdArgs, options = {}) {
  const { cwd, env, stdio = 'inherit' } = options
  execFileSync(cmd, cmdArgs, { cwd, env, stdio })
}

function lookupBinary(name) {
  const result = spawnSync('which', [name], { encoding: 'utf8' })
  if (result.status !== 0) return null
  const value = result.stdout.trim()
  return value.length > 0 ? value : null
}

function ensureBinary(name, installHint) {
  const value = lookupBinary(name)
  if (value) return value
  const hint = installHint ? ` (${installHint})` : ''
  throw new Error(`required binary not found: ${name}${hint}`)
}

function detectRealNodeDir() {
  const output = spawnSync('which', ['-a', 'node'], { encoding: 'utf8' })
  if (output.status !== 0) {
    return path.dirname(process.execPath)
  }

  const candidates = output.stdout
    .split('\n')
    .map((item) => item.trim())
    .filter(Boolean)

  for (const candidate of candidates) {
    if (candidate.includes('/.local/bin/node')) {
      continue
    }
    try {
      const resolved = fs.realpathSync(candidate)
      if (path.basename(resolved) === 'node') {
        return path.dirname(resolved)
      }
    } catch {
      // Ignore candidates that cannot be resolved.
    }
  }

  return path.dirname(process.execPath)
}

function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true })
}

function writeFixture(dir, lockFile) {
  ensureDir(dir)
  fs.writeFileSync(
    path.join(dir, 'package.json'),
    JSON.stringify(
      {
        name: `bench-${path.basename(dir)}`,
        version: '1.0.0',
        scripts: { dev: 'vite', build: 'tsc -p .', test: 'vitest' },
      },
      null,
      2,
    ),
    'utf8',
  )
  fs.writeFileSync(path.join(dir, lockFile), 'lock\n', 'utf8')
}

function writePnpmWorkspaceFixture(rootDir) {
  const pkgDir = path.join(rootDir, 'packages', 'app')
  ensureDir(pkgDir)

  fs.writeFileSync(
    path.join(rootDir, 'package.json'),
    JSON.stringify(
      {
        name: 'bench-pnpm-workspace',
        private: true,
        packageManager: 'pnpm@9.0.0',
        workspaces: ['packages/*'],
      },
      null,
      2,
    ),
    'utf8',
  )

  fs.writeFileSync(path.join(rootDir, 'pnpm-lock.yaml'), 'lock\n', 'utf8')

  fs.writeFileSync(
    path.join(pkgDir, 'package.json'),
    JSON.stringify(
      {
        name: 'bench-app',
        version: '1.0.0',
        scripts: { dev: 'vite', build: 'tsc -p .', test: 'vitest' },
      },
      null,
      2,
    ),
    'utf8',
  )

  return pkgDir
}

function createAlias(target, destination) {
  if (process.platform === 'win32') {
    fs.copyFileSync(target, destination)
    return
  }
  fs.symlinkSync(target, destination)
}

function aliasBinPath(rootDir, binName) {
  return process.platform === 'win32'
    ? path.join(rootDir, `${binName}.exe`)
    : path.join(rootDir, binName)
}

function shellQuote(value) {
  if (value.length === 0) {
    return "''"
  }
  return `'${value.replace(/'/g, `'\"'\"'`)}'`
}

function interpolateArgs(args, fixturePaths) {
  return args.map((arg) => {
    if (arg === '<npmFixture>') return fixturePaths.npm
    if (arg === '<pnpmFixture>') return fixturePaths.pnpm
    if (arg === '<yarnFixture>') return fixturePaths.yarn
    if (arg === '<bunFixture>') return fixturePaths.bun
    if (arg === '<pnpmWorkspaceFixture>') return fixturePaths.pnpmWorkspace
    return arg
  })
}

function buildCommand(envMap, binPath, cmdArgs) {
  const parts = ['env']
  for (const [key, value] of Object.entries(envMap)) {
    parts.push(`${key}=${value}`)
  }
  parts.push(binPath)
  parts.push(...cmdArgs)
  return parts.map((part) => shellQuote(part)).join(' ')
}

function percentile(sortedValues, p) {
  if (sortedValues.length === 0) return null
  const index = Math.max(0, Math.ceil(sortedValues.length * p) - 1)
  return sortedValues[index]
}

function fromHyperfineResult(rawResult) {
  const times = Array.isArray(rawResult.times)
    ? [...rawResult.times].sort((a, b) => a - b)
    : []

  return {
    mean: rawResult.mean * 1000,
    median: rawResult.median * 1000,
    p95: (percentile(times, 0.95) ?? rawResult.max) * 1000,
    min: rawResult.min * 1000,
    max: rawResult.max * 1000,
    stddev: rawResult.stddev * 1000,
    samples: times.length,
  }
}

function safeRatio(numerator, denominator) {
  if (!(numerator > 0) || !(denominator > 0)) {
    return null
  }
  return numerator / denominator
}

function runHyperfineCase({
  repoRoot,
  caseDef,
  runs,
  warmups,
  rawOutputPath,
  antfuCommand,
  ourCommand,
}) {
  const cmdArgs = [
    '--runs',
    String(runs),
    '--warmup',
    String(warmups),
    '--style',
    'none',
    '--command-name',
    'antfu',
    '--command-name',
    'ours',
    '--export-json',
    rawOutputPath,
    antfuCommand,
    ourCommand,
  ]

  const result = spawnSync('hyperfine', cmdArgs, {
    cwd: repoRoot,
    encoding: 'utf8',
  })

  if (result.status !== 0) {
    throw new Error(
      `hyperfine failed for case ${caseDef.id}\nstdout:\n${result.stdout || ''}\nstderr:\n${
        result.stderr || ''
      }`,
    )
  }

  const raw = JSON.parse(fs.readFileSync(rawOutputPath, 'utf8'))
  if (!Array.isArray(raw.results) || raw.results.length !== 2) {
    throw new Error(`unexpected hyperfine result format for case ${caseDef.id}`)
  }

  const antfuStats = fromHyperfineResult(raw.results[0])
  const ourStats = fromHyperfineResult(raw.results[1])

  return {
    id: caseDef.id,
    group: caseDef.group,
    case: caseDef.name,
    raw_json: rawOutputPath,
    antfu: antfuStats,
    ours: ourStats,
    speedup_mean: safeRatio(antfuStats.mean, ourStats.mean),
    speedup_median: safeRatio(antfuStats.median, ourStats.median),
  }
}

function geometricMean(values) {
  if (values.length === 0 || values.some((value) => value <= 0)) {
    return null
  }
  const sum = values.reduce((acc, value) => acc + Math.log(value), 0)
  return Math.exp(sum / values.length)
}

function groupBy(rows, key) {
  const out = {}
  for (const row of rows) {
    const group = row[key]
    if (!out[group]) out[group] = []
    out[group].push(row)
  }
  return out
}

function printSummary(payload) {
  const lines = []
  lines.push('')
  lines.push(
    'case'.padEnd(44) +
      'antfu mean (ms)'.padStart(16) +
      'ours mean (ms)'.padStart(16) +
      'speedup'.padStart(12),
  )
  lines.push('-'.repeat(92))

  for (const row of payload.results) {
    const speedupText =
      row.speedup_mean === null || !Number.isFinite(row.speedup_mean)
        ? 'n/a'
        : `${row.speedup_mean.toFixed(2)}x`
    lines.push(
      row.case.slice(0, 44).padEnd(44) +
        row.antfu.mean.toFixed(2).padStart(16) +
        row.ours.mean.toFixed(2).padStart(16) +
        speedupText.padStart(12),
    )
  }

  lines.push('-'.repeat(92))
  if (payload.summary.command_geometric_mean_speedup !== null) {
    lines.push(
      `command geometric mean speedup (excludes startup): ${payload.summary.command_geometric_mean_speedup.toFixed(
        2,
      )}x`,
    )
  }
  if (payload.summary.all_geometric_mean_speedup !== null) {
    lines.push(`all-case geometric mean speedup: ${payload.summary.all_geometric_mean_speedup.toFixed(2)}x`)
  }
  lines.push(`executed cases: ${payload.summary.executed_cases}, skipped cases: ${payload.summary.skipped_cases}`)
  lines.push('')
  process.stdout.write(lines.join('\n'))
}

function toMarkdown(payload) {
  const lines = []
  lines.push('# Benchmark Summary')
  lines.push('')
  lines.push(`- Timestamp: ${payload.timestamp}`)
  lines.push(`- Tool: ${payload.benchmark_tool}`)
  lines.push(`- Runs: ${payload.runs}`)
  lines.push(`- Warmups: ${payload.warmups}`)
  lines.push(`- Platform: ${payload.platform}`)
  lines.push(`- Arch: ${payload.arch}`)
  lines.push(`- Total cases: ${payload.summary.total_cases}`)
  lines.push(`- Executed cases: ${payload.summary.executed_cases}`)
  lines.push(`- Skipped cases: ${payload.summary.skipped_cases}`)
  lines.push('')
  lines.push('| Case | Group | Antfu mean (ms) | Ours mean (ms) | Speedup |')
  lines.push('|---|---|---:|---:|---:|')
  for (const row of payload.results) {
    const speedupText =
      row.speedup_mean === null || !Number.isFinite(row.speedup_mean)
        ? 'n/a'
        : `${row.speedup_mean.toFixed(2)}x`
    lines.push(
      `| ${row.case} | ${row.group} | ${row.antfu.mean.toFixed(2)} | ${row.ours.mean.toFixed(2)} | ${speedupText} |`,
    )
  }
  lines.push('')
  if (payload.summary.command_geometric_mean_speedup !== null) {
    lines.push(
      `- Command geometric mean speedup (excludes startup): ${payload.summary.command_geometric_mean_speedup.toFixed(
        2,
      )}x`,
    )
  }
  if (payload.summary.all_geometric_mean_speedup !== null) {
    lines.push(`- All-case geometric mean speedup: ${payload.summary.all_geometric_mean_speedup.toFixed(2)}x`)
  }
  lines.push('')
  lines.push('## Per-Group Geometric Mean Speedup')
  lines.push('')
  lines.push('| Group | Geometric mean speedup |')
  lines.push('|---|---:|')
  for (const [group, value] of Object.entries(payload.summary.per_group_geometric_mean_speedup)) {
    lines.push(`| ${group} | ${value.toFixed(2)}x |`)
  }
  if (payload.skipped.length > 0) {
    lines.push('')
    lines.push('## Skipped Cases')
    lines.push('')
    lines.push('| Case | Reason |')
    lines.push('|---|---|')
    for (const skipped of payload.skipped) {
      lines.push(`| ${skipped.case} | ${skipped.reason} |`)
    }
  }
  lines.push('')

  return `${lines.join('\n')}\n`
}

function main() {
  const args = parseArgs(process.argv.slice(2))
  const scriptDir = path.dirname(fileURLToPath(import.meta.url))
  const repoRoot = path.resolve(scriptDir, '..')
  const ourBin = path.join(repoRoot, 'target', 'release', 'hni')
  const benchCacheDir = path.join(repoRoot, 'benchmarks', '.cache')
  const antfuPrefix = path.join(benchCacheDir, 'antfu-ni')
  const antfuBinDir = path.join(antfuPrefix, 'bin')
  const resultsDir = path.join(repoRoot, 'benchmarks', 'results')
  const rawResultsDir = path.join(resultsDir, 'raw')

  ensureDir(benchCacheDir)
  ensureDir(resultsDir)
  ensureDir(rawResultsDir)

  ensureBinary('hyperfine', 'install via `brew install hyperfine` or your package manager')
  ensureBinary('npm')
  if (args.build) {
    ensureBinary('cargo')
  }

  if (args.build) {
    process.stdout.write('Building release binary...\n')
    run('cargo', ['build', '--release'], { cwd: repoRoot })
  }

  if (!fs.existsSync(ourBin)) {
    throw new Error(`missing binary: ${ourBin}`)
  }

  process.stdout.write('Installing/updating @antfu/ni in benchmarks cache...\n')
  run('npm', ['i', '-g', '@antfu/ni', '--prefix', antfuPrefix], { cwd: repoRoot })

  const nodeDir = detectRealNodeDir()
  const bunBin = lookupBinary('bun')
  const bunDir = bunBin ? path.dirname(fs.realpathSync(bunBin)) : null

  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'nirs-bench-'))
  try {
    const allCases = buildCases()
    const ourAliasDir = path.join(tempRoot, 'our-bin')
    ensureDir(ourAliasDir)

    const ourBins = [...new Set(allCases.map((caseDef) => caseDef.ours.bin))]
    for (const binName of ourBins) {
      createAlias(ourBin, aliasBinPath(ourAliasDir, binName))
    }

    const fixturesRoot = path.join(tempRoot, 'fixtures')
    const fixturePaths = {
      npm: path.join(fixturesRoot, 'npm'),
      pnpm: path.join(fixturesRoot, 'pnpm'),
      yarn: path.join(fixturesRoot, 'yarn'),
      bun: path.join(fixturesRoot, 'bun'),
      pnpmWorkspace: '',
    }

    writeFixture(fixturePaths.npm, 'package-lock.json')
    writeFixture(fixturePaths.pnpm, 'pnpm-lock.yaml')
    writeFixture(fixturePaths.yarn, 'yarn.lock')
    writeFixture(fixturePaths.bun, 'bun.lockb')
    fixturePaths.pnpmWorkspace = writePnpmWorkspaceFixture(path.join(fixturesRoot, 'pnpm-workspace'))

    const availableBins = {
      npm: Boolean(lookupBinary('npm')),
      npx: Boolean(lookupBinary('npx')),
      pnpm: Boolean(lookupBinary('pnpm')),
      yarn: Boolean(lookupBinary('yarn')),
      bun: Boolean(lookupBinary('bun')),
    }

    const benchPath = [nodeDir, bunDir, antfuBinDir, ourAliasDir, process.env.PATH]
      .filter(Boolean)
      .join(path.delimiter)

    const baseEnv = { PATH: benchPath }
    const ourExtraEnv = {
      HNI_SKIP_PM_CHECK: '1',
      HNI_AUTO_INSTALL: 'false',
    }

    const skipped = []
    const runnableCases = []
    for (const caseDef of allCases) {
      const missingBins = caseDef.requiredBins.filter((bin) => !availableBins[bin])
      if (missingBins.length > 0) {
        skipped.push({
          id: caseDef.id,
          case: caseDef.name,
          reason: `missing required binaries: ${missingBins.join(', ')}`,
        })
        continue
      }

      const antfuBinPath = path.join(antfuBinDir, caseDef.antfu.bin)
      if (!fs.existsSync(antfuBinPath)) {
        skipped.push({
          id: caseDef.id,
          case: caseDef.name,
          reason: `missing antfu binary: ${caseDef.antfu.bin}`,
        })
        continue
      }

      runnableCases.push(caseDef)
    }

    if (runnableCases.length === 0) {
      throw new Error('no runnable benchmark cases after filtering')
    }

    const stamp = new Date().toISOString().replace(/[:.]/g, '-')
    const results = []

    process.stdout.write(
      `Running hyperfine benchmark matrix (${args.warmups} warmups + ${args.runs} measured runs per case)...\n`,
    )
    process.stdout.write(`Total cases: ${allCases.length}, runnable: ${runnableCases.length}\n`)

    for (const [index, caseDef] of runnableCases.entries()) {
      process.stdout.write(`[${index + 1}/${runnableCases.length}] ${caseDef.name}\n`)

      const antfuBinPath = path.join(antfuBinDir, caseDef.antfu.bin)
      const ourBinPath = aliasBinPath(ourAliasDir, caseDef.ours.bin)
      const antfuArgs = interpolateArgs(caseDef.antfu.args, fixturePaths)
      const ourArgs = interpolateArgs(caseDef.ours.args, fixturePaths)

      const antfuCommand = buildCommand(baseEnv, antfuBinPath, antfuArgs)
      const ourCommand = buildCommand({ ...baseEnv, ...ourExtraEnv }, ourBinPath, ourArgs)
      const rawOutputPath = path.join(rawResultsDir, `${stamp}-${caseDef.id}.json`)

      try {
        results.push(
          runHyperfineCase({
            repoRoot,
            caseDef,
            runs: args.runs,
            warmups: args.warmups,
            rawOutputPath,
            antfuCommand,
            ourCommand,
          }),
        )
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error)
        const summaryLine = message.split('\n')[0]
        skipped.push({
          id: caseDef.id,
          case: caseDef.name,
          reason: `runtime failure: ${summaryLine}`,
        })
        process.stdout.write(`  skipped: ${summaryLine}\n`)
      }
    }

    if (results.length === 0) {
      throw new Error('all benchmark cases were skipped')
    }

    const allSpeedups = results
      .map((row) => row.speedup_mean)
      .filter((value) => value !== null && Number.isFinite(value) && value > 0)
    const commandSpeedups = results
      .filter((row) => row.group !== 'startup')
      .map((row) => row.speedup_mean)
      .filter((value) => value !== null && Number.isFinite(value) && value > 0)
    const grouped = groupBy(results, 'group')
    const perGroup = {}
    for (const [group, rows] of Object.entries(grouped)) {
      const gm = geometricMean(
        rows
          .map((row) => row.speedup_mean)
          .filter((value) => value !== null && Number.isFinite(value) && value > 0),
      )
      if (gm !== null) {
        perGroup[group] = gm
      }
    }

    const payload = {
      timestamp: new Date().toISOString(),
      benchmark_tool: 'hyperfine',
      host: os.hostname(),
      platform: process.platform,
      arch: process.arch,
      runs: args.runs,
      warmups: args.warmups,
      binaries: {
        ours: ourBin,
        antfu_prefix: antfuPrefix,
        hyperfine: ensureBinary('hyperfine'),
        node_dir: nodeDir,
        bun_bin: bunBin,
      },
      fixtures: fixturePaths,
      summary: {
        total_cases: allCases.length,
        executed_cases: results.length,
        skipped_cases: skipped.length,
        command_geometric_mean_speedup: geometricMean(commandSpeedups),
        all_geometric_mean_speedup: geometricMean(allSpeedups),
        per_group_geometric_mean_speedup: perGroup,
      },
      skipped,
      results,
    }

    printSummary(payload)

    const resultBase = path.join(resultsDir, `benchmark-${stamp}`)
    let jsonPath = null
    let markdownPath = null

    if (args.outputJson) {
      jsonPath = `${resultBase}.json`
      fs.writeFileSync(jsonPath, `${JSON.stringify(payload, null, 2)}\n`, 'utf8')
      process.stdout.write(`JSON written to ${jsonPath}\n`)
    }

    if (args.outputMarkdown) {
      markdownPath = `${resultBase}.md`
      fs.writeFileSync(markdownPath, toMarkdown(payload), 'utf8')
      process.stdout.write(`Markdown written to ${markdownPath}\n`)
    }

    if (
      args.minSpeedup !== null &&
      payload.summary.command_geometric_mean_speedup !== null &&
      payload.summary.command_geometric_mean_speedup < args.minSpeedup
    ) {
      process.stderr.write(
        `command geometric mean speedup ${payload.summary.command_geometric_mean_speedup.toFixed(
          2,
        )}x is below threshold ${args.minSpeedup.toFixed(2)}x\n`,
      )
      process.exitCode = 2
    }
  } finally {
    fs.rmSync(tempRoot, { recursive: true, force: true })
  }
}

try {
  main()
} catch (error) {
  const message = error instanceof Error ? error.message : String(error)
  process.stderr.write(`${message}\n`)
  process.exitCode = 1
}
