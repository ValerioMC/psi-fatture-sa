/**
 * Which feature may import which, like `src-tauri/tests/architecture/domain_edges.rs` for the
 * backend. `model/` types are free to share, and a view may embed another feature's component or
 * read its store: views are where features meet. Every other import across features is an edge.
 */
import { describe, expect, it } from 'vitest'

/** A new edge is a design decision: change this table and the one in the README together. */
const ALLOWED: Readonly<Record<string, readonly string[]>> = {
  appointment: ['client', 'service'],
  client: ['settings'],
  dashboard: ['invoice'],
  email: ['invoice'],
  invoice: ['client', 'service', 'settings'],
  service: [],
  settings: ['terms'],
  sts: ['client', 'invoice'],
  terms: [],
  update: [],
}

interface FeatureImport {
  file: string
  from: string
  to: string
  folder: string
}

const SOURCES = import.meta.glob<string>('./features/**/*.{ts,vue}', { query: '?raw', import: 'default', eager: true })
const SPECIFIER = /(?:from\s+|import\s*\(\s*|import\s+)['"]([^'"]+)['"]/g

/** `./features/invoice/lib/tax.ts` → its segments after `features`. */
function segmentsOf(path: string): string[] {
  const parts: string[] = []
  for (const part of path.split('/')) {
    if (part === '..') parts.pop()
    else if (part !== '.' && part !== '') parts.push(part)
  }
  return parts.slice(parts.indexOf('features') + 1)
}

/** Where a specifier points inside `src/features`, if it does. */
function targetOf(file: string, specifier: string): string[] | null {
  if (specifier.startsWith('@/features/')) return segmentsOf(specifier.slice(2))
  if (!specifier.startsWith('.')) return null
  const directory = file.slice(0, file.lastIndexOf('/'))
  const target = segmentsOf(`${directory}/${specifier}`)
  return target.length > 0 ? target : null
}

function featureImports(): FeatureImport[] {
  const imports: FeatureImport[] = []
  for (const [file, source] of Object.entries(SOURCES)) {
    const from = segmentsOf(file)[0]
    for (const [, specifier] of source.matchAll(SPECIFIER)) {
      const target = targetOf(file, specifier)
      if (target !== null && target[0] !== from) imports.push({ file, from, to: target[0], folder: target[1] ?? '' })
    }
  }
  return imports
}

/** Shared types, and a view composing another feature's component or state, are not dependencies. */
function isEdge(file: string, folder: string): boolean {
  if (folder === 'model') return false
  const viewer = segmentsOf(file)[1]
  return !(viewer === 'views' && (folder === 'components' || folder === 'store'))
}

function edgesInCode(): Map<string, string> {
  const edges = new Map<string, string>()
  for (const { file, from, to, folder } of featureImports()) {
    const key = `${from} -> ${to}`
    if (isEdge(file, folder) && !edges.has(key)) edges.set(key, file)
  }
  return edges
}

/** A path of allowed edges from `from` back to `target`, if the table has one. */
function pathBackTo(from: string, target: string, visited: Set<string>): string[] | null {
  for (const next of ALLOWED[from] ?? []) {
    if (next === target) return [next]
    if (visited.has(next)) continue
    visited.add(next)
    const rest = pathBackTo(next, target, visited)
    if (rest !== null) return [next, ...rest]
  }
  return null
}

describe('feature boundaries', () => {
  it('lets every feature use only what the table allows', () => {
    const violations = [...edgesInCode()]
      .filter(([key]) => {
        const [from, to] = key.split(' -> ')
        return !(ALLOWED[from] ?? []).includes(to)
      })
      .map(([key, file]) => `${key} (${file})`)
    expect(violations).toEqual([])
  })

  it('allows only edges the code uses', () => {
    const used = new Set(edgesInCode().keys())
    const unused = Object.entries(ALLOWED)
      .flatMap(([from, uses]) => uses.map((to) => `${from} -> ${to}`))
      .filter((edge) => !used.has(edge))
    expect(unused).toEqual([])
  })

  it('lists exactly the features in the code', () => {
    const inCode = new Set(Object.keys(SOURCES).map((file) => segmentsOf(file)[0]))
    expect([...inCode].sort()).toEqual(Object.keys(ALLOWED).sort())
  })

  it('has no feature reaching itself through others', () => {
    const cycles = Object.keys(ALLOWED)
      .map((start) => {
        const path = pathBackTo(start, start, new Set())
        return path === null ? null : [start, ...path].join(' -> ')
      })
      .filter((cycle) => cycle !== null)
    expect(cycles).toEqual([])
  })
})
