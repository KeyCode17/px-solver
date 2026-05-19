#!/usr/bin/env node
// Static-decode every `gB("…")` and `gC(N)` call in a webcrack output.
//
//   gB("literal")  -> "<decoded>"
//   gC(N)          -> "<decoded gE[N]>"
//
// Also leaves a `// gB: <text>` trail next to multi-line replacements.

'use strict';

const fs = require('node:fs');
const path = require('node:path');
const parser = require('@babel/parser');
const traverse = require('@babel/traverse').default;
const generate = require('@babel/generator').default;
const t = require('@babel/types');

const { decodeOuter } = require('./gb_decode');

function usage() {
  process.stderr.write('usage: transform.js <input.js> <output.js>\n');
  process.exit(2);
}
if (process.argv.length < 4) usage();

const inputPath = path.resolve(process.argv[2]);
const outputPath = path.resolve(process.argv[3]);

const src = fs.readFileSync(inputPath, 'utf8');
const ast = parser.parse(src, { sourceType: 'script', errorRecovery: true });

// Extract `gE = ["...","..."];`
let gE = null;
traverse(ast, {
  VariableDeclarator(p) {
    if (
      p.node.id.type === 'Identifier' &&
      p.node.id.name === 'gE' &&
      p.node.init &&
      p.node.init.type === 'ArrayExpression'
    ) {
      gE = p.node.init.elements.map((e) => {
        if (!e || e.type !== 'StringLiteral') return null;
        return e.value;
      });
    }
  },
});
if (!gE) {
  process.stderr.write('FATAL: could not locate `gE` array of encoded strings\n');
  process.exit(1);
}

let gbInline = 0;
let gcInline = 0;
let gcDecodeFailures = 0;

function safeDecode(raw) {
  try {
    return decodeOuter(raw);
  } catch (_) {
    return null;
  }
}

traverse(ast, {
  CallExpression(p) {
    const callee = p.node.callee;
    if (callee.type !== 'Identifier') return;
    if (callee.name === 'gB') {
      const arg = p.node.arguments[0];
      if (arg && arg.type === 'StringLiteral') {
        const out = safeDecode(arg.value);
        if (out !== null) {
          p.replaceWith(t.stringLiteral(out));
          gbInline += 1;
        }
      }
      return;
    }
    if (callee.name === 'gC' || callee.name === 'gF') {
      const args = p.node.arguments;
      if (args.length !== 1) return; // the global-getter `gF()` takes none
      const arg = args[0];
      if (arg && arg.type === 'NumericLiteral') {
        const raw = gE[arg.value];
        if (raw == null) {
          gcDecodeFailures += 1;
          return;
        }
        const out = safeDecode(raw);
        if (out !== null) {
          p.replaceWith(t.stringLiteral(out));
          gcInline += 1;
        } else {
          gcDecodeFailures += 1;
        }
      }
    }
  },
});

const { code } = generate(
  ast,
  {
    retainLines: false,
    comments: true,
    compact: false,
    jsescOption: { minimal: true },
  },
  src,
);

fs.writeFileSync(outputPath, code);

process.stderr.write(
  `gB inlined: ${gbInline}, gC inlined: ${gcInline}, gC failures: ${gcDecodeFailures}\n`,
);
