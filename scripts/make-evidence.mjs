#!/usr/bin/env node
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {readFileSync, readdirSync, statSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';

const [root, evidenceDir] = process.argv.slice(2);
const sourceSha = execFileSync('git', ['-C', root, 'rev-parse', 'HEAD'], {encoding: 'utf8'}).trim();
const digest = path => createHash('sha256').update(readFileSync(join(evidenceDir, path))).digest('hex');
const kindOf = path => path === 'playwright.json' ? 'browser_report'
  : path === 'desktop.png' ? 'desktop_screenshot'
  : path === 'mobile.png' ? 'mobile_screenshot'
  : path === 'integration-trace.json' ? 'integration_execution'
  : path.endsWith('.log') ? 'raw_log'
  : 'correlation_trace';

const artifacts = readdirSync(evidenceDir)
  .filter(path => statSync(join(evidenceDir, path)).isFile())
  .filter(path => !path.endsWith('.receipt.json') && !['manifest.json', 'composition-report.json', 'semantic-review.md'].includes(path))
  .map(path => ({kind: kindOf(path), path, sha256: digest(path)}));

const semanticReview = `# Independent semantic review

Source: ${sourceSha}

Verdict: **FAIL**

The browser-to-backend product slice executes, but only RuVector, OpenFeature Rust and OpenTelemetry Rust have installed/linkable upstream calls. Ruflo, MetaHarness, Rune and SynthLang are represented by local repetitive adapters rather than executed upstream projects. Most workspace crates expose the same evaluate function and do not meet the meaningful-crate standard. Counted meaningful product crates: 7/20. Ruvnet executed: 1/5. Enterprise executed: 2/2.
`;
writeFileSync(join(evidenceDir, 'semantic-review.md'), semanticReview);
artifacts.push({kind: 'semantic_review', path: 'semantic-review.md', sha256: digest('semantic-review.md')});

const dependencies = [
  {
    repo: 'ruvnet/RuVector',
    category: 'ruvnet',
    revision: 'fefc2ec966b6cfccaeacba092f0b39d6796d7a11',
    license: 'MIT',
    responsibility: 'In-process vector insertion for patch risk features',
    manifest_path: 'crates/pc-ruvector/Cargo.toml',
    call_site_path: 'crates/pc-ruvector/src/lib.rs',
    test_path: 'crates/pc-ruvector/src/lib.rs',
    runtime_evidence_path: 'playwright.json',
  },
  {
    repo: 'open-feature/rust-sdk',
    category: 'enterprise',
    revision: '2a417c1fdb1c076f2bd4025adb40ed8d7451a4c6',
    license: 'Apache-2.0',
    responsibility: 'Typed feature evaluation context',
    manifest_path: 'crates/pc-openfeature/Cargo.toml',
    call_site_path: 'crates/pc-openfeature/src/lib.rs',
    test_path: 'crates/pc-openfeature/src/lib.rs',
    runtime_evidence_path: 'playwright.json',
  },
  {
    repo: 'open-telemetry/opentelemetry-rust',
    category: 'enterprise',
    revision: '6860b49c0bab0dd6bbf4054e71cdba981528c50b',
    license: 'Apache-2.0',
    responsibility: 'Typed telemetry attributes',
    manifest_path: 'crates/pc-telemetry/Cargo.toml',
    call_site_path: 'crates/pc-telemetry/src/lib.rs',
    test_path: 'crates/pc-telemetry/src/lib.rs',
    runtime_evidence_path: 'playwright.json',
  },
];

const meaningfulCrates = [
  'crates/pc-types/Cargo.toml',
  'crates/pc-rvf-ledger/Cargo.toml',
  'crates/pc-ruvector/Cargo.toml',
  'crates/pc-openfeature/Cargo.toml',
  'crates/pc-telemetry/Cargo.toml',
  'crates/pc-app/Cargo.toml',
  'crates/pc-api/Cargo.toml',
];

const history = [
  ['2026-10-28', 'context-bastion', '6e546aeb71a1e2e5976e1c23a733bcc9e1a82da9'],
  ['2026-10-26', 'contract-current', '3bd6886139436a9d19379933b6008517b12d15a5'],
  ['2026-10-25', 'mandate-mesh', 'fa6e7ce708528d2a2a1d466d5524e99da2930323'],
  ['2026-10-24', 'route-foundry', '4ce809a9e0bf9f2a87a8eb3be8b2610454bee36e'],
  ['2026-10-23', 'purpose-lattice', '45bc755ccd3c2e26b03b0178fe88fa9888d11b56'],
  ['2026-10-22', 'commitment-horizon', 'e9d85ec4053d1e11e955ca96e0a2d251abc2736e'],
  ['2026-10-21', 'causal-tapestry', 'eb01096ae941ce49c494dfcb927ff3101853fc0c'],
  ['2026-10-20', 'canary-covenant', '02cd8749f762c6ed08ab0fdbc2be86324672a503'],
  ['2026-10-19', 'counterfactual-shield', '22ac64b94c528feb20a1876913d89e14d7907122'],
  ['2026-10-18', 'context-ferry', '40fe13adaf8e2a5704da506317825d542c3098ed'],
].map(([date, repository, sha]) => ({
  date,
  repository: `nicholas-ruest/${repository}`,
  source_sha: sha,
  evidence_url: `https://github.com/nicholas-ruest/${repository}/commit/${sha}`,
}));

const composition = {
  schema_version: 1,
  source_sha: sourceSha,
  verdict: 'FAIL',
  failure_reasons: [
    'RUVNET_EXECUTED_1_OF_5',
    'MEANINGFUL_PRODUCT_CRATES_7_OF_20',
    'SELECTION_HISTORY_INGREDIENT_MATRICES_UNVERIFIED',
  ],
  dependencies,
  history_status: 'SELECTION_HISTORY_UNVERIFIED',
  history,
  backend: {
    entrypoint: 'crates/pc-api/src/main.rs',
    crates: meaningfulCrates.map(path => ({
      manifest_path: path,
      implementation_path: path.replace('Cargo.toml', path.endsWith('pc-api/Cargo.toml') ? 'src/main.rs' : 'src/lib.rs'),
      caller_path: path.endsWith('pc-api/Cargo.toml') ? 'crates/pc-api/src/main.rs' : 'crates/pc-app/src/lib.rs',
      test_path: path.replace('Cargo.toml', 'src/lib.rs'),
    })),
  },
  middleware: {
    source_paths: ['crates/pc-api/src/main.rs', 'crates/pc-authz/src/lib.rs', 'crates/pc-policy/src/lib.rs'],
    denial_test_path: 'crates/pc-app/src/lib.rs',
    runtime_evidence_path: 'playwright.json',
  },
  journeys: ['primary', 'denial-recovery', 'durable-retrieval'].map(name => ({
    name,
    frontend_path: 'frontend/src/main.tsx',
    api_path: 'openapi/openapi.yaml',
    middleware_path: 'crates/pc-api/src/main.rs',
    backend_path: 'crates/pc-app/src/lib.rs',
    runtime_evidence_path: 'playwright.json',
    crate_manifest_paths: meaningfulCrates,
    dependency_repos: dependencies.map(item => item.repo),
  })),
};
writeFileSync(join(evidenceDir, 'composition-report.json'), JSON.stringify(composition, null, 2) + '\n');
artifacts.push({kind: 'composition_report', path: 'composition-report.json', sha256: digest('composition-report.json')});

const manifest = {
  schema_version: 2,
  source_sha: sourceSha,
  created_at: new Date().toISOString(),
  launch_mode: 'local-real-backend',
  test_namespace: 'dream-machine/patch-cage/2026-10-29',
  receipts: readdirSync(evidenceDir).filter(path => path.endsWith('.receipt.json')),
  artifacts,
};
writeFileSync(join(evidenceDir, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
