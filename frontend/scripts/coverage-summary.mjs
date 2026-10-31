// Turns Vitest's coverage/coverage-summary.json into Markdown for the GitHub job summary:
// the totals, then the least-covered files. CI appends the output to $GITHUB_STEP_SUMMARY.
import { readFileSync } from 'node:fs';
import { relative } from 'node:path';

const SUMMARY_PATH = 'coverage/coverage-summary.json';
const METRICS = ['lines', 'statements', 'functions', 'branches'];
const LEAST_COVERED_COUNT = 15;
const GOOD_PCT = 90;
const FAIR_PCT = 75;

const summary = JSON.parse(readFileSync(SUMMARY_PATH, 'utf8'));
const badge = (/** @type {number} */ pct) => (pct >= GOOD_PCT ? '🟢' : pct >= FAIR_PCT ? '🟡' : '🔴');
const cell = (/** @type {{pct: number, covered: number, total: number}} */ m) =>
	`${badge(m.pct)} ${m.pct.toFixed(2)}% (${m.covered}/${m.total})`;

const out = [
	'## Frontend: test coverage',
	'',
	'Unit (Node) and component (jsdom) tests, merged.',
	'',
	'| Metric | Coverage |',
	'| --- | --- |',
	...METRICS.map((k) => `| ${k[0].toUpperCase()}${k.slice(1)} | ${cell(summary.total[k])} |`),
	'',
];

const files = Object.entries(summary)
	.filter(([file]) => file !== 'total')
	.map(([file, m]) => ({ file: relative(process.cwd(), file), ...m }))
	.filter((f) => f.lines.total > 0)
	.sort((a, b) => a.lines.pct - b.lines.pct || b.lines.total - a.lines.total)
	.slice(0, LEAST_COVERED_COUNT);

if (files.length > 0) {
	out.push(
		`<details><summary>${files.length} least-covered files</summary>`,
		'',
		'| File | Lines | Branches | Functions |',
		'| --- | --- | --- | --- |',
		...files.map((f) => `| \`${f.file}\` | ${cell(f.lines)} | ${cell(f.branches)} | ${cell(f.functions)} |`),
		'',
		'</details>',
		'',
	);
}

console.log(out.join('\n'));
