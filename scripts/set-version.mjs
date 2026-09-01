#!/usr/bin/env node
// Allinea la versione dichiarata nel repository a quella del tag di release.
// Uso: node scripts/set-version.mjs 0.9.0

import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const SEMVER = /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/;

const version = process.argv[2];
if (!version || !SEMVER.test(version)) {
  console.error(`set-version: versione non valida: ${JSON.stringify(process.argv[2])}. Attesa forma X.Y.Z.`);
  process.exit(1);
}

const root = join(dirname(fileURLToPath(import.meta.url)), '..');

/** Riscrive il campo "version" di un file JSON mantenendo l'indentazione a 2 spazi. */
function setJsonVersion(relativePath) {
  const path = join(root, relativePath);
  const parsed = JSON.parse(readFileSync(path, 'utf8'));
  if (typeof parsed.version !== 'string') {
    throw new Error(`${relativePath}: campo "version" assente o non testuale`);
  }
  parsed.version = version;
  writeFileSync(path, `${JSON.stringify(parsed, null, 2)}\n`);
}

/** Riscrive la versione della sezione [package] di un Cargo.toml. */
function setCargoVersion(relativePath) {
  const path = join(root, relativePath);
  const source = readFileSync(path, 'utf8');
  const packageSection = source.indexOf('[package]');
  if (packageSection === -1) {
    throw new Error(`${relativePath}: sezione [package] assente`);
  }
  const versionLine = /^version\s*=\s*".*"$/m;
  const head = source.slice(0, packageSection);
  const tail = source.slice(packageSection);
  if (!versionLine.test(tail)) {
    throw new Error(`${relativePath}: nessuna riga "version" in [package]`);
  }
  writeFileSync(path, head + tail.replace(versionLine, `version = "${version}"`));
}

try {
  setJsonVersion('package.json');
  setJsonVersion('src-tauri/tauri.conf.json');
  setCargoVersion('src-tauri/Cargo.toml');
} catch (error) {
  console.error(`set-version: ${error.message}`);
  process.exit(1);
}

console.log(`set-version: versione allineata a ${version}`);
