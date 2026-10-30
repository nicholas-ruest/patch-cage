#!/usr/bin/env node
// Necessary evidence checks; independent source/product review remains mandatory.
import { createHash, randomUUID } from 'node:crypto';
import { execFileSync, spawnSync } from 'node:child_process';
import { closeSync, existsSync, mkdirSync, openSync, readFileSync, readdirSync,
  realpathSync, statSync, writeFileSync } from 'node:fs';
import { isAbsolute, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

export const suites = ['rust', 'frontend', 'api', 'middleware', 'integrations',
  'benchmark', 'ablations', 'containers', 'browser'];
const kinds = ['desktop_screenshot', 'mobile_screenshot', 'browser_report',
  'browser_trace', 'correlation_trace', 'semantic_review', 'composition_report'];
const digest = (bytes) => createHash('sha256').update(bytes).digest('hex');
const git = (root, ...args) => execFileSync('git', ['-C', root, ...args],
  { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
const inside = (root, path) => {
  const rel = relative(root, path);
  return rel === '' || (!isAbsolute(rel) && rel !== '..' && !rel.startsWith(`..${sep}`));
};
function source(root) {
  const sha = git(root, 'rev-parse', 'HEAD');
  if (!/^[a-f0-9]{40}$/.test(sha)) throw new Error('Invalid source commit');
  if (git(root, 'status', '--porcelain', '--untracked-files=all')) {
    throw new Error('Source checkout must be clean; keep evidence outside it');
  }
  return sha;
}
function safeFile(root, path) {
  if (typeof path !== 'string' || !path || isAbsolute(path)) throw new Error('Invalid artifact path');
  const target = realpathSync(resolve(root, path));
  if (!inside(realpathSync(root), target) || !statSync(target).isFile()) {
    throw new Error(`Artifact escapes evidence directory: ${path}`);
  }
  return target;
}
function evidenceDirectory(root, evidence) {
  mkdirSync(evidence, { recursive: true });
  if (inside(realpathSync(root), realpathSync(evidence))) {
    throw new Error('Use an external evidence directory to avoid circular commit hashes');
  }
}

export function record(root, evidence, suite, argv) {
  root = realpathSync(root);
  evidence = resolve(evidence);
  if (!suites.includes(suite) || !argv.length) throw new Error('Unknown suite or missing command');
  evidenceDirectory(root, evidence);
  const sourceSha = source(root);
  const id = `${suite}-${randomUUID()}`;
  const log = `${id}.log`;
  const fd = openSync(resolve(evidence, log), 'wx');
  const startedAt = new Date().toISOString();
  let result;
  try {
    result = spawnSync(argv[0], argv.slice(1), {
      cwd: root, shell: false, stdio: ['ignore', fd, fd],
      env: { ...process.env, DREAM_MACHINE_EVIDENCE_DIR: evidence,
        DREAM_MACHINE_SOURCE_SHA: sourceSha },
    });
  } finally { closeSync(fd); }
  let integrityError = null;
  try { if (source(root) !== sourceSha) integrityError = 'Source changed during execution'; }
  catch (error) { integrityError = error.message; }
  const receipt = {
    schema_version: 2, suite, source_sha: sourceSha, argv,
    started_at: startedAt, ended_at: new Date().toISOString(),
    exit_code: result.status, signal: result.signal, error: result.error?.message ?? null,
    integrity_error: integrityError, log, log_sha256: digest(readFileSync(resolve(evidence, log))),
  };
  writeFileSync(resolve(evidence, `${id}.receipt.json`), `${JSON.stringify(receipt, null, 2)}\n`);
  return receipt;
}

function browserJourneys(report) {
  const observed = new Set();
  let count = 0;
  function walk(suite) {
    for (const spec of suite.specs ?? []) {
      for (const test of spec.tests ?? []) {
        count += 1;
        if (test.expectedStatus !== 'passed' || !test.results?.length ||
            test.results.some((result) => result.status !== 'passed')) {
          throw new Error(`Browser test failed, skipped, retried or incomplete: ${spec.title}`);
        }
        for (const journey of ['primary', 'denial-recovery', 'durable-retrieval']) {
          if (spec.title?.includes(`[journey:${journey}]`)) observed.add(journey);
        }
      }
    }
    for (const child of suite.suites ?? []) walk(child);
  }
  walk(report);
  if (report.errors?.length || count < 3 || observed.size !== 3) {
    throw new Error('Executed Playwright report must cover all three named journeys without errors');
  }
}

function composition(report, root, evidence, manifest, sha) {
  const fail = (message) => { throw new Error(`NONCOMPLIANT_COMPOSITION: ${message}`); };
  const required = (value, label) => {
    if (typeof value !== 'string' || !value.trim()) fail(`Missing ${label}`);
    return value;
  };
  const list = (value, label) => {
    if (!Array.isArray(value) || !value.length) fail(`Missing ${label}`);
    return value;
  };
  const sourcePath = (path) => safeFile(root, required(path, 'source path'));
  const trace = (path) => {
    required(path, 'runtime evidence path');
    const artifact = manifest.artifacts.find((item) => item.path === path &&
      ['integration_execution', 'correlation_trace'].includes(item.kind));
    if (!artifact) fail(`Runtime evidence must be a hashed trace artifact: ${path}`);
    const bytes = readFileSync(safeFile(evidence, path));
    if (!bytes.length || digest(bytes) !== artifact.sha256) fail(`Invalid runtime trace: ${path}`);
  };
  const upstreams = (items) => {
    const repos = new Map();
    for (const item of list(items, 'dependencies')) {
      const repo = required(item.repo, 'canonical owner/repository').toLowerCase();
      if (!/^[a-z0-9_.-]+\/[a-z0-9_.-]+$/.test(repo) || repo.endsWith('.git')) fail('Use canonical owner/repository');
      if (!['ruvnet', 'enterprise'].includes(item.category) ||
          (item.category === 'ruvnet') !== repo.startsWith('ruvnet/')) fail(`Invalid dependency category: ${repo}`);
      if (repos.has(repo)) fail(`Duplicate upstream project: ${repo}`);
      repos.set(repo, item.category);
    }
    return repos;
  };
  if (report.schema_version !== 1 || report.source_sha !== sha) fail('Stale or invalid composition report');
  const repos = upstreams(report.dependencies);
  for (const [category, floor] of [['ruvnet', 5], ['enterprise', 2]]) {
    if ([...repos.values()].filter((value) => value === category).length < floor) {
      fail(`Need at least ${floor} distinct executed ${category} upstream projects`);
    }
  }
  for (const item of report.dependencies) {
    if (!/^[a-f0-9]{40}$/.test(item.revision ?? '')) fail(`Unpinned revision: ${item.repo}`);
    required(item.license, 'OSS license'); required(item.responsibility, 'product responsibility');
    for (const key of ['manifest_path', 'call_site_path', 'test_path']) sourcePath(item[key]);
    trace(item.runtime_evidence_path);
  }
  if (report.history_status !== 'VERIFIED') fail('SELECTION_HISTORY_UNVERIFIED');
  const history = list(report.history, 'verified previous build history');
  if (history.length > 10) fail('Use the last ten attributable completed builds');
  const priorShas = new Set();
  const priorRepos = history.map((build) => {
    required(build.date, 'prior build date');
    required(build.evidence_url, 'prior build evidence URL');
    if (!/^https:\/\/github\.com\//.test(build.evidence_url) ||
        !/^[a-f0-9]{40}$/.test(build.source_sha ?? '') || priorShas.has(build.source_sha)) fail('Invalid history provenance');
    priorShas.add(build.source_sha);
    return upstreams(build.dependencies);
  });
  for (const [category, floor] of [['ruvnet', 2], ['enterprise', 1]]) {
    const fresh = [...repos].filter(([repo, type]) => type === category && !priorRepos[0].has(repo));
    if (fresh.length < floor) fail(`Rotate at least ${floor} ${category} projects from the previous build`);
  }
  const signature = (map) => [...map.keys()].sort().join(',');
  if (priorRepos.some((prior) => signature(prior) === signature(repos))) fail('Repeated full ingredient set');
  const entrypoint = sourcePath(report.backend?.entrypoint);
  if (!entrypoint.endsWith('.rs')) fail('Backend entrypoint must be Rust source');
  const crates = new Set();
  for (const crate of list(report.backend?.crates, 'backend crate call-path map')) {
    const path = sourcePath(crate.manifest_path);
    if (!path.endsWith(`${sep}Cargo.toml`) || crates.has(path)) fail('Duplicate or invalid crate manifest');
    crates.add(path);
    for (const key of ['implementation_path', 'caller_path', 'test_path']) sourcePath(crate[key]);
  }
  if (crates.size < 20) fail('Backend must compose at least 20 meaningful Rust crates');
  for (const path of list(report.middleware?.source_paths, 'middleware source paths')) sourcePath(path);
  sourcePath(report.middleware?.denial_test_path); trace(report.middleware?.runtime_evidence_path);
  const observed = new Set(); const coveredCrates = new Set(); const primaryRepos = new Set();
  for (const journey of list(report.journeys, 'full-stack journeys')) {
    if (!['primary', 'denial-recovery', 'durable-retrieval'].includes(journey.name)) fail('Invalid journey');
    observed.add(journey.name);
    for (const key of ['frontend_path', 'api_path', 'middleware_path', 'backend_path']) sourcePath(journey[key]);
    required(journey.correlation_id, 'journey correlation ID'); trace(journey.runtime_evidence_path);
    for (const path of list(journey.crate_manifest_paths, 'journey crate call paths')) {
      const target = sourcePath(path);
      if (!crates.has(target)) fail('Journey references an uncounted crate');
      coveredCrates.add(target);
    }
    if (!Array.isArray(journey.dependency_repos)) fail('Missing journey dependency mapping');
    for (const repo of journey.dependency_repos) {
      if (!repos.has(repo.toLowerCase())) fail('Journey references an uncounted upstream');
      if (journey.name === 'primary') primaryRepos.add(repo.toLowerCase());
    }
  }
  if (observed.size !== 3 || coveredCrates.size !== crates.size || primaryRepos.size !== repos.size) {
    fail('All counted crates and dependencies must participate in complete product workflows');
  }
}

export function verify(root, evidence, expectedSha) {
  root = realpathSync(root);
  evidence = realpathSync(evidence);
  evidenceDirectory(root, evidence);
  const sha = source(root);
  if (expectedSha && sha !== expectedSha) throw new Error('Requested candidate differs from checkout HEAD');
  const missing = [];
  for (const path of ['Cargo.toml', 'Cargo.lock', 'frontend/package.json',
    'frontend/package-lock.json', 'docs/prototype-acceptance.md', 'scripts/verify-prototype.sh']) {
    if (!existsSync(resolve(root, path))) missing.push(path);
  }
  if (missing.length) throw new Error(`NONCOMPLIANT_BUILD: missing product layers: ${missing.join(', ')}`);
  const manifest = JSON.parse(readFileSync(resolve(evidence, 'manifest.json'), 'utf8'));
  if (manifest.schema_version !== 2 || manifest.source_sha !== sha) {
    throw new Error('Unbound/stale evidence: schema_version 2 and exact tested source_sha required');
  }
  const receipts = readdirSync(evidence).filter((name) => name.endsWith('.receipt.json'))
    .map((name) => JSON.parse(readFileSync(safeFile(evidence, name), 'utf8')));
  for (const receipt of receipts) {
    if (receipt.schema_version !== 2 || receipt.source_sha !== sha || receipt.exit_code !== 0 ||
        receipt.signal || receipt.error || receipt.integrity_error ||
        !Array.isArray(receipt.argv) || !receipt.argv.length ||
        !Number.isFinite(Date.parse(receipt.started_at)) ||
        !Number.isFinite(Date.parse(receipt.ended_at)) ||
        Date.parse(receipt.ended_at) < Date.parse(receipt.started_at)) {
      throw new Error('Failed, stale or invalid execution receipt');
    }
    if (digest(readFileSync(safeFile(evidence, receipt.log))) !== receipt.log_sha256) {
      throw new Error('Execution log hash mismatch');
    }
  }
  for (const suite of suites) {
    if (!receipts.some((receipt) => receipt.suite === suite)) throw new Error(`Missing executed suite: ${suite}`);
  }
  const artifacts = manifest.artifacts;
  if (!Array.isArray(artifacts)) throw new Error('Manifest must list hashed evidence artifacts');
  for (const kind of kinds) {
    const artifact = artifacts.find((item) => item.kind === kind);
    if (!artifact) throw new Error(`Missing evidence artifact: ${kind}`);
    const bytes = readFileSync(safeFile(evidence, artifact.path));
    if (!/^[a-f0-9]{64}$/.test(artifact.sha256 ?? '') || digest(bytes) !== artifact.sha256 || !bytes.length) {
      throw new Error(`Missing, empty or hash-mismatched artifact: ${kind}`);
    }
    if (kind.endsWith('_screenshot')) {
      if (bytes.length < 24 || !bytes.subarray(0, 8).equals(Buffer.from('89504e470d0a1a0a', 'hex'))) {
        throw new Error('Screenshot must be actual PNG bytes');
      }
      const width = bytes.readUInt32BE(16);
      const height = bytes.readUInt32BE(20);
      if (height < 480 || (kind === 'desktop_screenshot' ? width < 1024 : width < 320 || width > 600)) {
        throw new Error('Screenshot dimensions do not match desktop/mobile evidence');
      }
    }
    if (kind === 'browser_trace' && !bytes.subarray(0, 2).equals(Buffer.from('PK'))) {
      throw new Error('Browser trace must be a Playwright ZIP');
    }
    if (kind === 'browser_report') browserJourneys(JSON.parse(bytes.toString('utf8')));
    if (kind === 'semantic_review' && !bytes.toString('utf8').includes(sha)) {
      throw new Error('Semantic review is not bound to the tested source commit');
    }
    if (kind === 'composition_report') composition(JSON.parse(bytes.toString('utf8')), root, evidence, manifest, sha);
  }
  return { status: 'EVIDENCE_CHECKS_PASS', source_sha: sha, suites: suites.length,
    limitation: 'Necessary checks only. Independently review crate substance, real call paths, upstream execution, UI quality and CI provenance before BUILT.' };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const [mode, root, evidence, ...rest] = process.argv.slice(2);
    if (!root || !evidence || !['record', 'verify'].includes(mode)) {
      throw new Error('Usage: gate record ROOT EVIDENCE SUITE -- COMMAND [ARGS] | gate verify ROOT EVIDENCE [SHA]');
    }
    if (mode === 'record') {
      if (rest[1] !== '--') throw new Error('Separate suite and command with --');
      const receipt = record(root, evidence, rest[0], rest.slice(2));
      console.log(JSON.stringify(receipt));
      process.exitCode = receipt.integrity_error || receipt.error || receipt.signal ? 1 : receipt.exit_code ?? 1;
    } else console.log(JSON.stringify(verify(root, evidence, rest[0]), null, 2));
  } catch (error) {
    console.error(JSON.stringify({ status: 'NONCOMPLIANT_BUILD', error: error.message }));
    process.exitCode = 1;
  }
}

