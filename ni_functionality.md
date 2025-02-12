Directory Structure:

└── ./
    ├── src
    │   ├── commands
    │   │   ├── index.ts
    │   │   ├── na.ts
    │   │   ├── nci.ts
    │   │   ├── ni.ts
    │   │   ├── nlx.ts
    │   │   ├── nr.ts
    │   │   ├── nu.ts
    │   │   └── nun.ts
    │   ├── completion.ts
    │   ├── config.ts
    │   ├── detect.ts
    │   ├── fetch.ts
    │   ├── fs.ts
    │   ├── index.ts
    │   ├── parse.ts
    │   ├── runner.ts
    │   ├── storage.ts
    │   └── utils.ts
    ├── build.config.ts
    └── README.md



---
File: /src/commands/index.ts
---

export * from '../index'



---
File: /src/commands/na.ts
---

import { parseNa } from '../parse'
import { runCli } from '../runner'

runCli(parseNa)



---
File: /src/commands/nci.ts
---

import { parseNi } from '../parse'
import { runCli } from '../runner'

runCli(
  (agent, args, hasLock) => parseNi(agent, [...args, '--frozen-if-present'], hasLock),
  { autoInstall: true },
)



---
File: /src/commands/ni.ts
---

import type { Choice } from '@posva/prompts'
import process from 'node:process'
import prompts from '@posva/prompts'
import c from 'ansis'
import { Fzf } from 'fzf'
import { fetchNpmPackages } from '../fetch'
import { parseNi } from '../parse'
import { runCli } from '../runner'
import { exclude } from '../utils'

runCli(async (agent, args, ctx) => {
  const isInteractive = args[0] === '-i'

  if (isInteractive) {
    let fetchPattern: string

    if (args[1] && !args[1].startsWith('-')) {
      fetchPattern = args[1]
    }
    else {
      const { pattern } = await prompts({
        type: 'text',
        name: 'pattern',
        message: 'search for package',
      })

      fetchPattern = pattern
    }

    if (!fetchPattern) {
      process.exitCode = 1
      return
    }

    const packages = await fetchNpmPackages(fetchPattern)

    if (!packages.length) {
      console.error('No results found')
      process.exitCode = 1
      return
    }

    const fzf = new Fzf(packages, {
      selector: (item: Choice) => item.title,
      casing: 'case-insensitive',
    })

    const { dependency } = await prompts({
      type: 'autocomplete',
      name: 'dependency',
      choices: packages,
      instructions: false,
      message: 'choose a package to install',
      limit: 15,
      async suggest(input: string, choices: Choice[]) {
        const results = fzf.find(input)
        return results.map(r => choices.find((c: any) => c.value === r.item.value))
      },
    })

    if (!dependency) {
      process.exitCode = 1
      return
    }

    args = exclude(args, '-d', '-p', '-i')

    /**
     * yarn and bun do not support
     * the installation of peers programmatically
     */
    const canInstallPeers = ['npm', 'pnpm'].includes(agent)

    const { mode } = await prompts({
      type: 'select',
      name: 'mode',
      message: `install ${c.yellow(dependency.name)} as`,
      choices: [
        {
          title: 'prod',
          value: '',
          selected: true,
        },
        {
          title: 'dev',
          value: '-D',
        },
        {
          title: `peer`,
          value: '--save-peer',
          disabled: !canInstallPeers,
        },
      ],
    })

    args.push(dependency.name, mode)
  }

  return parseNi(agent, args, ctx)
})



---
File: /src/commands/nlx.ts
---

import { parseNlx } from '../parse'
import { runCli } from '../runner'

runCli(parseNlx)



---
File: /src/commands/nr.ts
---

import type { Choice } from '@posva/prompts'
import type { RunnerContext } from '../runner'
import process from 'node:process'
import prompts from '@posva/prompts'
import { byLengthAsc, Fzf } from 'fzf'
import { rawCompletionScript } from '../completion'
import { getPackageJSON } from '../fs'
import { parseNr } from '../parse'
import { runCli } from '../runner'
import { dump, load } from '../storage'
import { limitText } from '../utils'

function readPackageScripts(ctx: RunnerContext | undefined) {
  // support https://www.npmjs.com/package/npm-scripts-info conventions
  const pkg = getPackageJSON(ctx)
  const scripts = pkg.scripts || {}
  const scriptsInfo = pkg['scripts-info'] || {}

  return Object.entries(scripts)
    .filter(i => !i[0].startsWith('?'))
    .map(([key, cmd]) => ({
      key,
      cmd,
      description: scriptsInfo[key] || scripts[`?${key}`] || cmd,
    }))
}

runCli(async (agent, args, ctx) => {
  const storage = await load()

  // Use --completion to generate completion script and do completion logic
  // (No package manager would have an argument named --completion)
  if (args[0] === '--completion') {
    const compLine = process.env.COMP_LINE
    const rawCompCword = process.env.COMP_CWORD
    if (compLine !== undefined && rawCompCword !== undefined) {
      const compCword = Number.parseInt(rawCompCword, 10)
      const compWords = args.slice(1)
      // Only complete the second word (nr __here__ ...)
      if (compCword === 1) {
        const raw = readPackageScripts(ctx)
        const fzf = new Fzf(raw, {
          selector: item => item.key,
          casing: 'case-insensitive',
          tiebreakers: [byLengthAsc],
        })

        // compWords will be ['nr'] when the user does not type anything after `nr` so fallback to empty string
        const results = fzf.find(compWords[1] || '')

        // eslint-disable-next-line no-console
        console.log(results.map(r => r.item.key).join('\n'))
      }
    }
    else {
      // eslint-disable-next-line no-console
      console.log(rawCompletionScript)
    }
    return
  }

  if (args[0] === '-') {
    if (!storage.lastRunCommand) {
      if (!ctx?.programmatic) {
        console.error('No last command found')
        process.exit(1)
      }

      throw new Error('No last command found')
    }
    args[0] = storage.lastRunCommand
  }

  if (args.length === 0 && !ctx?.programmatic) {
    const raw = readPackageScripts(ctx)

    const terminalColumns = process.stdout?.columns || 80

    const choices: Choice[] = raw
      .map(({ key, description }) => ({
        title: key,
        value: key,
        description: limitText(description, terminalColumns - 15),
      }))

    const fzf = new Fzf(raw, {
      selector: item => `${item.key} ${item.description}`,
      casing: 'case-insensitive',
      tiebreakers: [byLengthAsc],
    })

    if (storage.lastRunCommand) {
      const last = choices.find(i => i.value === storage.lastRunCommand)
      if (last)
        choices.unshift(last)
    }

    try {
      const { fn } = await prompts({
        name: 'fn',
        message: 'script to run',
        type: 'autocomplete',
        choices,
        async suggest(input: string, choices: Choice[]) {
          if (!input)
            return choices
          const results = fzf.find(input)
          return results.map(r => choices.find(c => c.value === r.item.key))
        },
      })
      if (!fn)
        return
      args.push(fn)
    }
    catch {
      process.exit(1)
    }
  }

  if (storage.lastRunCommand !== args[0]) {
    storage.lastRunCommand = args[0]
    dump()
  }

  return parseNr(agent, args)
})



---
File: /src/commands/nu.ts
---

import { parseNu } from '../parse'
import { runCli } from '../runner'

runCli(parseNu)



---
File: /src/commands/nun.ts
---

import type { Choice, PromptType } from '@posva/prompts'
import process from 'node:process'
import prompts from '@posva/prompts'
import { Fzf } from 'fzf'
import { getPackageJSON } from '../fs'
import { parseNun } from '../parse'
import { runCli } from '../runner'
import { exclude } from '../utils'

runCli(async (agent, args, ctx) => {
  const isInteractive = !args.length && !ctx?.programmatic

  if (isInteractive || args[0] === '-m') {
    const pkg = getPackageJSON(ctx)

    const allDependencies = { ...pkg.dependencies, ...pkg.devDependencies }

    const raw = Object.entries(allDependencies) as [string, string][]

    if (!raw.length) {
      console.error('No dependencies found')
      return
    }

    const fzf = new Fzf(raw, {
      selector: ([dep, version]) => `${dep} ${version}`,
      casing: 'case-insensitive',
    })

    const choices: Choice[] = raw.map(([dependency, version]) => ({
      title: dependency,
      value: dependency,
      description: version,
    }))

    const isMultiple = args[0] === '-m'

    const type: PromptType = isMultiple
      ? 'autocompleteMultiselect'
      : 'autocomplete'

    if (isMultiple)
      args = exclude(args, '-m')

    try {
      const { depsToRemove } = await prompts({
        type,
        name: 'depsToRemove',
        choices,
        instructions: false,
        message: `remove ${isMultiple ? 'dependencies' : 'dependency'}`,
        async suggest(input: string, choices: Choice[]) {
          const results = fzf.find(input)
          return results.map(r => choices.find(c => c.value === r.item[0]))
        },
      })

      if (!depsToRemove) {
        process.exitCode = 1
        return
      }

      const isSingleDependency = typeof depsToRemove === 'string'

      if (isSingleDependency)
        args.push(depsToRemove)
      else args.push(...depsToRemove)
    }
    catch {
      process.exit(1)
    }
  }

  return parseNun(agent, args, ctx)
})



---
File: /src/completion.ts
---

// Print completion script
export const rawCompletionScript = `
###-begin-nr-completion-###

if type complete &>/dev/null; then
  _nr_completion() {
    local words
    local cur
    local cword
    _get_comp_words_by_ref -n =: cur words cword
    IFS=$'\\n'
    COMPREPLY=($(COMP_CWORD=$cword COMP_LINE=$cur nr --completion \${words[@]}))
  }
  complete -F _nr_completion nr
fi

###-end-nr-completion-###
`.trim()



---
File: /src/config.ts
---

import type { Agent } from 'package-manager-detector'
import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import ini from 'ini'
import { detect } from './detect'

const customRcPath = process.env.NI_CONFIG_FILE

const home = process.platform === 'win32'
  ? process.env.USERPROFILE
  : process.env.HOME

const defaultRcPath = path.join(home || '~/', '.nirc')

const rcPath = customRcPath || defaultRcPath

interface Config {
  defaultAgent: Agent | 'prompt'
  globalAgent: Agent
}

const defaultConfig: Config = {
  defaultAgent: 'prompt',
  globalAgent: 'npm',
}

let config: Config | undefined

export async function getConfig(): Promise<Config> {
  if (!config) {
    config = Object.assign(
      {},
      defaultConfig,
      fs.existsSync(rcPath)
        ? ini.parse(fs.readFileSync(rcPath, 'utf-8'))
        : null,
    )

    if (process.env.NI_DEFAULT_AGENT)
      config.defaultAgent = process.env.NI_DEFAULT_AGENT as Agent

    if (process.env.NI_GLOBAL_AGENT)
      config.globalAgent = process.env.NI_GLOBAL_AGENT as Agent

    const agent = await detect({ programmatic: true })
    if (agent)
      config.defaultAgent = agent
  }

  return config
}

export async function getDefaultAgent(programmatic?: boolean) {
  const { defaultAgent } = await getConfig()
  if (defaultAgent === 'prompt' && (programmatic || process.env.CI))
    return 'npm'
  return defaultAgent
}

export async function getGlobalAgent() {
  const { globalAgent } = await getConfig()
  return globalAgent
}



---
File: /src/detect.ts
---

import process from 'node:process'
import prompts from '@posva/prompts'
import { detect as detectPM } from 'package-manager-detector'
import { INSTALL_PAGE } from 'package-manager-detector/constants'
import terminalLink from 'terminal-link'
import { x } from 'tinyexec'
import { cmdExists } from './utils'

export interface DetectOptions {
  autoInstall?: boolean
  programmatic?: boolean
  cwd?: string
  /**
   * Should use Volta when present
   *
   * @see https://volta.sh/
   * @default true
   */
  detectVolta?: boolean
}

export async function detect({ autoInstall, programmatic, cwd }: DetectOptions = {}) {
  const {
    name,
    agent,
    version,
  } = await detectPM({
    cwd,
    onUnknown: (packageManager) => {
      if (!programmatic) {
        console.warn('[ni] Unknown packageManager:', packageManager)
      }
      return undefined
    },
  }) || {}

  // auto install
  if (name && !cmdExists(name) && !programmatic) {
    if (!autoInstall) {
      console.warn(`[ni] Detected ${name} but it doesn't seem to be installed.\n`)

      if (process.env.CI)
        process.exit(1)

      const link = terminalLink(name, INSTALL_PAGE[name])
      const { tryInstall } = await prompts({
        name: 'tryInstall',
        type: 'confirm',
        message: `Would you like to globally install ${link}?`,
      })
      if (!tryInstall)
        process.exit(1)
    }

    await x(
      'npm',
      ['i', '-g', `${name}${version ? `@${version}` : ''}`],
      {
        nodeOptions: {
          stdio: 'inherit',
          cwd,
        },
        throwOnError: true,
      },
    )
  }

  return agent
}



---
File: /src/fetch.ts
---

import type { Choice } from '@posva/prompts'
import process from 'node:process'
import c from 'ansis'
import { formatPackageWithUrl } from './utils'

export interface NpmPackage {
  name: string
  description: string
  version: string
  keywords: string[]
  date: string
  links: {
    npm: string
    homepage: string
    repository: string
  }
}

interface NpmRegistryResponse {
  objects: { package: NpmPackage }[]
}

export async function fetchNpmPackages(pattern: string): Promise<Choice[]> {
  const registryLink = (pattern: string) =>
    `https://registry.npmjs.com/-/v1/search?text=${pattern}&size=35`

  const terminalColumns = process.stdout?.columns || 80

  try {
    const result = await fetch(registryLink(pattern))
      .then(res => res.json()) as NpmRegistryResponse

    return result.objects.map(({ package: pkg }) => ({
      title: formatPackageWithUrl(
        `${pkg.name.padEnd(30, ' ')} ${c.blue`v${pkg.version}`}`,
        pkg.links.repository ?? pkg.links.npm,
        terminalColumns,
      ),
      value: pkg,
    }))
  }
  catch {
    console.error('Error when fetching npm registry')
    process.exit(1)
  }
}



---
File: /src/fs.ts
---

import type { RunnerContext } from './runner'
import fs from 'node:fs'
import { resolve } from 'node:path'
import process from 'node:process'

export function getPackageJSON(ctx?: RunnerContext): any {
  const cwd = ctx?.cwd ?? process.cwd()
  const path = resolve(cwd, 'package.json')

  if (fs.existsSync(path)) {
    try {
      const raw = fs.readFileSync(path, 'utf-8')
      const data = JSON.parse(raw)
      return data
    }
    catch (e) {
      if (!ctx?.programmatic) {
        console.warn('Failed to parse package.json')
        process.exit(1)
      }

      throw e
    }
  }
}



---
File: /src/index.ts
---

export * from './config'
export * from './detect'

export * from './parse'
export * from './runner'
export * from './utils'
export * from 'package-manager-detector/commands'
export * from 'package-manager-detector/constants'



---
File: /src/parse.ts
---

import type { Agent, Command, ResolvedCommand } from 'package-manager-detector'
import type { Runner } from './runner'
import { COMMANDS, constructCommand } from '.'
import { exclude } from './utils'

export class UnsupportedCommand extends Error {
  constructor({ agent, command }: { agent: Agent, command: Command }) {
    super(`Command "${command}" is not support by agent "${agent}"`)
  }
}

export function getCommand(
  agent: Agent,
  command: Command,
  args: string[] = [],
): ResolvedCommand {
  if (!COMMANDS[agent])
    throw new Error(`Unsupported agent "${agent}"`)
  if (!COMMANDS[agent][command])
    throw new UnsupportedCommand({ agent, command })

  return constructCommand(COMMANDS[agent][command], args)!
}

export const parseNi = <Runner>((agent, args, ctx) => {
  // bun use `-d` instead of `-D`, #90
  if (agent === 'bun')
    args = args.map(i => i === '-D' ? '-d' : i)

  // npm use `--omit=dev` instead of `--production`
  if (agent === 'npm')
    args = args.map(i => i === '-P' ? '--omit=dev' : i)

  if (args.includes('-P'))
    args = args.map(i => i === '-P' ? '--production' : i)

  if (args.includes('-g'))
    return getCommand(agent, 'global', exclude(args, '-g'))

  if (args.includes('--frozen-if-present')) {
    args = exclude(args, '--frozen-if-present')
    return getCommand(agent, ctx?.hasLock ? 'frozen' : 'install', args)
  }

  if (args.includes('--frozen'))
    return getCommand(agent, 'frozen', exclude(args, '--frozen'))

  if (args.length === 0 || args.every(i => i.startsWith('-')))
    return getCommand(agent, 'install', args)

  return getCommand(agent, 'add', args)
})

export const parseNr = <Runner>((agent, args) => {
  if (args.length === 0)
    args.push('start')

  let hasIfPresent = false
  if (args.includes('--if-present')) {
    args = exclude(args, '--if-present')
    hasIfPresent = true
  }

  const cmd = getCommand(agent, 'run', args)
  if (!cmd)
    return cmd

  if (hasIfPresent)
    cmd.args.splice(1, 0, '--if-present')

  return cmd
})

export const parseNu = <Runner>((agent, args) => {
  if (args.includes('-i'))
    return getCommand(agent, 'upgrade-interactive', exclude(args, '-i'))

  return getCommand(agent, 'upgrade', args)
})

export const parseNun = <Runner>((agent, args) => {
  if (args.includes('-g'))
    return getCommand(agent, 'global_uninstall', exclude(args, '-g'))
  return getCommand(agent, 'uninstall', args)
})

export const parseNlx = <Runner>((agent, args) => {
  return getCommand(agent, 'execute', args)
})

export const parseNa = <Runner>((agent, args) => {
  return getCommand(agent, 'agent', args)
})

export function serializeCommand(command?: ResolvedCommand) {
  if (!command)
    return undefined
  if (command.args.length === 0)
    return command.command
  return `${command.command} ${command.args.map(i => i.includes(' ') ? `"${i}"` : i).join(' ')}`
}



---
File: /src/runner.ts
---

import type { Agent, ResolvedCommand } from 'package-manager-detector'
import type { Options as TinyExecOptions } from 'tinyexec'
import type { DetectOptions } from './detect'
/* eslint-disable no-console */
import { resolve } from 'node:path'
import process from 'node:process'
import prompts from '@posva/prompts'
import c from 'ansis'
import { AGENTS } from 'package-manager-detector'
import { x } from 'tinyexec'
import { version } from '../package.json'
import { getDefaultAgent, getGlobalAgent } from './config'
import { detect } from './detect'
import { getCommand, UnsupportedCommand } from './parse'
import { cmdExists, remove } from './utils'

const DEBUG_SIGN = '?'

export interface RunnerContext {
  programmatic?: boolean
  hasLock?: boolean
  cwd?: string
}

export type Runner = (agent: Agent, args: string[], ctx?: RunnerContext) => Promise<ResolvedCommand | undefined> | ResolvedCommand | undefined

export async function runCli(fn: Runner, options: DetectOptions & { args?: string[] } = {}) {
  const {
    args = process.argv.slice(2).filter(Boolean),
  } = options
  try {
    await run(fn, args, options)
  }
  catch (error) {
    if (error instanceof UnsupportedCommand && !options.programmatic)
      console.log(c.red(`\u2717 ${error.message}`))

    if (!options.programmatic)
      process.exit(1)

    throw error
  }
}

export async function getCliCommand(
  fn: Runner,
  args: string[],
  options: DetectOptions = {},
  cwd: string = options.cwd ?? process.cwd(),
) {
  const isGlobal = args.includes('-g')
  if (isGlobal)
    return await fn(await getGlobalAgent(), args)

  let agent = (await detect({ ...options, cwd })) || (await getDefaultAgent(options.programmatic))
  if (agent === 'prompt') {
    agent = (
      await prompts({
        name: 'agent',
        type: 'select',
        message: 'Choose the agent',
        choices: AGENTS.filter(i => !i.includes('@')).map(value => ({ title: value, value })),
      })
    ).agent
    if (!agent)
      return
  }

  return await fn(agent as Agent, args, {
    programmatic: options.programmatic,
    hasLock: Boolean(agent),
    cwd,
  })
}

export async function run(fn: Runner, args: string[], options: DetectOptions = {}) {
  const {
    detectVolta = true,
  } = options

  const debug = args.includes(DEBUG_SIGN)
  if (debug)
    remove(args, DEBUG_SIGN)

  let cwd = options.cwd ?? process.cwd()
  if (args[0] === '-C') {
    cwd = resolve(cwd, args[1])
    args.splice(0, 2)
  }

  if (args.length === 1 && (args[0]?.toLowerCase() === '-v' || args[0] === '--version')) {
    const getCmd = (a: Agent) => AGENTS.includes(a)
      ? getCommand(a, 'agent', ['-v'])
      : { command: a, args: ['-v'] }
    const xVersionOptions = {
      nodeOptions: {
        cwd,
      },
      throwOnError: true,
    } satisfies Partial<TinyExecOptions>
    const getV = (a: string) => {
      const { command, args } = getCmd(a as Agent)
      return x(command, args, xVersionOptions)
        .then(e => e.stdout)
        .then(e => e.startsWith('v') ? e : `v${e}`)
    }
    const globalAgentPromise = getGlobalAgent()
    const globalAgentVersionPromise = globalAgentPromise.then(getV)
    const agentPromise = detect({ ...options, cwd }).then(a => a || '')
    const agentVersionPromise = agentPromise.then(a => a && getV(a))
    const nodeVersionPromise = getV('node')

    console.log(`@antfu/ni  ${c.cyan`v${version}`}`)
    console.log(`node       ${c.green(await nodeVersionPromise)}`)
    const [agent, agentVersion] = await Promise.all([agentPromise, agentVersionPromise])
    if (agent)
      console.log(`${agent.padEnd(10)} ${c.blue(agentVersion)}`)
    else
      console.log('agent      no lock file')
    const [globalAgent, globalAgentVersion] = await Promise.all([globalAgentPromise, globalAgentVersionPromise])
    console.log(`${(`${globalAgent} -g`).padEnd(10)} ${c.blue(globalAgentVersion)}`)
    return
  }

  if (args.length === 1 && ['-h', '--help'].includes(args[0])) {
    const dash = c.dim('-')
    console.log(c.green.bold('@antfu/ni') + c.dim` use the right package manager v${version}\n`)
    console.log(`ni    ${dash}  install`)
    console.log(`nr    ${dash}  run`)
    console.log(`nlx   ${dash}  execute`)
    console.log(`nu    ${dash}  upgrade`)
    console.log(`nun   ${dash}  uninstall`)
    console.log(`nci   ${dash}  clean install`)
    console.log(`na    ${dash}  agent alias`)
    console.log(`ni -v ${dash}  show used agent`)
    console.log(`ni -i ${dash}  interactive package management`)
    console.log(c.yellow('\ncheck https://github.com/antfu/ni for more documentation.'))
    return
  }

  const command = await getCliCommand(fn, args, options, cwd)

  if (!command)
    return

  if (detectVolta && cmdExists('volta')) {
    command.args = ['run', command.command, ...command.args]
    command.command = 'volta'
  }

  if (debug) {
    console.log(command)
    return
  }

  await x(
    command.command,
    command.args,
    {
      nodeOptions: {
        stdio: 'inherit',
        cwd,
      },
      throwOnError: true,
    },
  )
}



---
File: /src/storage.ts
---

import { existsSync, promises as fs } from 'node:fs'
import { resolve } from 'node:path'
import { CLI_TEMP_DIR, writeFileSafe } from './utils'

export interface Storage {
  lastRunCommand?: string
}

let storage: Storage | undefined

const storagePath = resolve(CLI_TEMP_DIR, '_storage.json')

export async function load(fn?: (storage: Storage) => Promise<boolean> | boolean) {
  if (!storage) {
    storage = existsSync(storagePath)
      ? (JSON.parse(await fs.readFile(storagePath, 'utf-8') || '{}') || {})
      : {}
  }

  if (fn) {
    if (await fn(storage!))
      await dump()
  }

  return storage!
}

export async function dump() {
  if (storage)
    await writeFileSafe(storagePath, JSON.stringify(storage))
}



---
File: /src/utils.ts
---

import type { Buffer } from 'node:buffer'
import { existsSync, promises as fs } from 'node:fs'
import os from 'node:os'
import { dirname, join } from 'node:path'
import process from 'node:process'
import c from 'ansis'
import terminalLink from 'terminal-link'
import which from 'which'

export const CLI_TEMP_DIR = join(os.tmpdir(), 'antfu-ni')

export function remove<T>(arr: T[], v: T) {
  const index = arr.indexOf(v)
  if (index >= 0)
    arr.splice(index, 1)

  return arr
}

export function exclude<T>(arr: T[], ...v: T[]) {
  return arr.slice().filter(item => !v.includes(item))
}

export function cmdExists(cmd: string) {
  return which.sync(cmd, { nothrow: true }) !== null
}

interface TempFile {
  path: string
  fd: fs.FileHandle
  cleanup: () => void
}

let counter = 0

async function openTemp(): Promise<TempFile | undefined> {
  if (!existsSync(CLI_TEMP_DIR))
    await fs.mkdir(CLI_TEMP_DIR, { recursive: true })

  const competitivePath = join(CLI_TEMP_DIR, `.${process.pid}.${counter}`)
  counter += 1

  return fs.open(competitivePath, 'wx')
    .then(fd => ({
      fd,
      path: competitivePath,
      cleanup() {
        fd.close().then(() => {
          if (existsSync(competitivePath))
            fs.unlink(competitivePath)
        })
      },
    }))
    .catch((error: any) => {
      if (error && error.code === 'EEXIST')
        return openTemp()

      else
        return undefined
    })
}

/**
 * Write file safely avoiding conflicts
 */
export async function writeFileSafe(
  path: string,
  data: string | Buffer = '',
): Promise<boolean> {
  const temp = await openTemp()

  if (temp) {
    fs.writeFile(temp.path, data)
      .then(() => {
        const directory = dirname(path)
        if (!existsSync(directory))
          fs.mkdir(directory, { recursive: true })

        return fs.rename(temp.path, path)
          .then(() => true)
          .catch(() => false)
      })
      .catch(() => false)
      .finally(temp.cleanup)
  }

  return false
}

export function limitText(text: string, maxWidth: number) {
  if (text.length <= maxWidth)
    return text
  return `${text.slice(0, maxWidth)}${c.dim('…')}`
}

export function formatPackageWithUrl(pkg: string, url?: string, limits = 80) {
  return url
    ? terminalLink(
        pkg,
        url,
        {
          fallback: (_, url) => (pkg.length + url.length > limits)
            ? pkg
            : pkg + c.dim` - ${url}`,
        },
      )
    : pkg
}



---
File: /build.config.ts
---

import { basename } from 'node:path'
import { globSync } from 'tinyglobby'
import { defineBuildConfig } from 'unbuild'

export default defineBuildConfig({
  entries: globSync(
    ['src/commands/*.ts'],
    { expandDirectories: false },
  ).map(i => ({
    input: i.slice(0, -3),
    name: basename(i).slice(0, -3),
  })),
  clean: true,
  declaration: true,
  rollup: {
    emitCJS: true,
    inlineDependencies: true,
    commonjs: {
      exclude: ['**/*.d.ts'],
    },
  },
})



---
File: /README.md
---

# ni

~~*`npm i` in a yarn project, again? F\*\*k!*~~

**ni** - use the right package manager

<br>

```
npm i -g @antfu/ni
```

<a href='https://docs.npmjs.com/cli/v6/commands/npm'>npm</a> · <a href='https://yarnpkg.com'>yarn</a> · <a href='https://pnpm.io/'>pnpm</a> · <a href='https://bun.sh/'>bun</a>

<br>

### `ni` - install

```bash
ni

# npm install
# yarn install
# pnpm install
# bun install
```

```bash
ni vite

# npm i vite
# yarn add vite
# pnpm add vite
# bun add vite
```

```bash
ni @types/node -D

# npm i @types/node -D
# yarn add @types/node -D
# pnpm add -D @types/node
# bun add -d @types/node
```

```bash
ni -P

# npm i --omit=dev
# yarn install --production
# pnpm i --production
# bun install --production
```

```bash
ni --frozen

# npm ci
# yarn install --frozen-lockfile (Yarn 1)
# yarn install --immutable (Yarn Berry)
# pnpm install --frozen-lockfile
# bun install --frozen-lockfile
```

```bash
ni -g eslint

# npm i -g eslint
# yarn global add eslint (Yarn 1)
# pnpm add -g eslint
# bun add -g eslint

# this uses default agent, regardless your current working directory
```

```bash
ni -i

# interactively select the dependency to install
# search for packages by name
```

<br>

### `nr` - run

```bash
nr dev --port=3000

# npm run dev -- --port=3000
# yarn run dev --port=3000
# pnpm run dev --port=3000
# bun run dev --port=3000
```

```bash
nr

# interactively select the script to run
# supports https://www.npmjs.com/package/npm-scripts-info convention
```

```bash
nr -

# rerun the last command
```

```bash
nr --completion >> ~/.bashrc

# add completion script to your shell (only bash supported for now)
```

<br>

### `nlx` - download & execute

```bash
nlx vitest

# npx vitest
# yarn dlx vitest
# pnpm dlx vitest
# bunx vitest
```

<br>

### `nu` - upgrade

```bash
nu

# npm upgrade
# yarn upgrade (Yarn 1)
# yarn up (Yarn Berry)
# pnpm update
# bun update
```

```bash
nu -i

# (not available for npm & bun)
# yarn upgrade-interactive (Yarn 1)
# yarn up -i (Yarn Berry)
# pnpm update -i
```

<br>

### `nun` - uninstall

```bash
nun webpack

# npm uninstall webpack
# yarn remove webpack
# pnpm remove webpack
# bun remove webpack
```

```bash
nun

# interactively select
# the dependency to remove
```

```bash
nun -m

# interactive select,
# but with multiple dependencies
```

```bash
nun -g silent

# npm uninstall -g silent
# yarn global remove silent
# pnpm remove -g silent
# bun remove -g silent
```

<br>

### `nci` - clean install

```bash
nci

# npm ci
# yarn install --frozen-lockfile
# pnpm install --frozen-lockfile
# bun install --frozen-lockfile
```

if the corresponding node manager is not present, this command will install it globally along the way.

<br>

### `na` - agent alias

```bash
na

# npm
# yarn
# pnpm
# bun
```

```bash
na run foo

# npm run foo
# yarn run foo
# pnpm run foo
# bun run foo
```

<br>

### Global Flags

```bash
# ?               | Print the command execution depends on the agent
ni vite ?

# -C              | Change directory before running the command
ni -C packages/foo vite
nr -C playground dev

# -v, --version   | Show version number
ni -v

# -h, --help      | Show help
ni -h
```

<br>

### Config

```ini
; ~/.nirc

; fallback when no lock found
defaultAgent=npm # default "prompt"

; for global installs
globalAgent=npm
```

```bash
# ~/.bashrc

# custom configuration file path
export NI_CONFIG_FILE="$HOME/.config/ni/nirc"

# environment variables have higher priority than config file if presented
export NI_DEFAULT_AGENT="npm" # default "prompt"
export NI_GLOBAL_AGENT="npm"
```

```ps
# for Windows

# custom configuration file path in PowerShell accessible within the `$profile` path
$Env:NI_CONFIG_FILE = 'C:\to\your\config\location'
```

<br>

### Integrations

#### asdf

You can also install ni via the [3rd-party asdf-plugin](https://github.com/CanRau/asdf-ni.git) maintained by [CanRau](https://github.com/CanRau)

```bash
# first add the plugin
asdf plugin add ni https://github.com/CanRau/asdf-ni.git

# then install the latest version
asdf install ni latest

# and make it globally available
asdf global ni latest
```

### How?

**ni** assumes that you work with lock-files (and you should).

Before `ni` runs the command, it detects your `yarn.lock` / `pnpm-lock.yaml` / `package-lock.json` / `bun.lock` / `bun.lockb` to know the current package manager (or `packageManager` field in your packages.json if specified) using the [package-manager-detector](https://github.com/antfu-collective/package-manager-detector) package and then runs the corresponding [package-manager-detector command](https://github.com/antfu-collective/package-manager-detector/blob/main/src/commands.ts).

### Trouble shooting

#### Conflicts with PowerShell

PowerShell comes with a built-in alias `ni` for the `New-Item` cmdlet. To remove the alias in your current PowerShell session in favor of this package, use the following command:

```PowerShell
'Remove-Item Alias:ni -Force -ErrorAction Ignore'
```

If you want to persist the changes, you can add them to your PowerShell profile. The profile path is accessible within the `$profile` variable. The ps1 profile file can normally be found at

- PowerShell 5 (Windows PowerShell): `C:\Users\USERNAME\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1`
- PowerShell 7: `C:\Users\USERNAME\Documents\PowerShell\Microsoft.PowerShell_profile.ps1`
- VSCode: `C:\Users\USERNAME\Documents\PowerShell\Microsoft.VSCode_profile.ps1`

You can use the following script to remove the alias at shell start by adding the above command to your profile:

```PowerShell
if (-not (Test-Path $profile)) {
  New-Item -ItemType File -Path (Split-Path $profile) -Force -Name (Split-Path $profile -Leaf)
}

$profileEntry = 'Remove-Item Alias:ni -Force -ErrorAction Ignore'
$profileContent = Get-Content $profile
if ($profileContent -notcontains $profileEntry) {
  ("`n" + $profileEntry) | Out-File $profile -Append -Force -Encoding UTF8
}
```

#### `nx` and `nix` is no longer available

We renamed `nx`/`nix` to `nlx` to avoid conflicts with the other existing tools - [nx](https://nx.dev/) and [nix](https://nixos.org/). You can always alias them back on your shell configuration file (`.zshrc`, `.bashrc`, etc).

```bash
alias nx="nlx"
# or
alias nix="nlx"
```

