#!/usr/bin/env node
// Emit JSON test vectors for each cipher primitive so the Rust port
// in `px-native/src/cipher/` can round-trip against them.
//
// Output: <repo>/px-native/tests/fixtures/cipher/{jw,hP,vL,vM,vN,vQ}.json

'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { jw, hP, vL, vM, vN, vQ, vP, IS } = require('./cipher_ref');

const OUT_DIR = path.resolve(__dirname, '../../../px-native/tests/fixtures/cipher');
fs.mkdirSync(OUT_DIR, { recursive: true });

// Some primitives produce JS strings whose code units may exceed 127.
// Serialising those directly to JSON yields a UTF-8 re-encoding that
// no longer equals the raw bytes. Convert to a u8 array first.
function asBytes(s) {
  const out = new Array(s.length);
  for (let i = 0; i < s.length; i++) out[i] = s.charCodeAt(i) & 0xff;
  return out;
}

function dump(name, vectors) {
  const file = path.join(OUT_DIR, `${name}.json`);
  fs.writeFileSync(file, JSON.stringify(vectors, null, 2) + '\n');
  process.stdout.write(`  wrote ${file} (${vectors.length} vectors)\n`);
}

// jw — single-byte XOR cipher. Outputs may include bytes > 127 so the
// `expected` field is a byte array rather than a JS string.
dump(
  'jw',
  [
    { input: asBytes(''), key: 50, expected: asBytes(jw('', 50)) },
    { input: asBytes('A'), key: 50, expected: asBytes(jw('A', 50)) },
    { input: asBytes('hello world'), key: IS, expected: asBytes(jw('hello world', IS)) },
    { input: asBytes('hello world'), key: 10, expected: asBytes(jw('hello world', 10)) },
    {
      input: asBytes('{"t":"PX12457","d":{}}'),
      key: IS,
      expected: asBytes(jw('{"t":"PX12457","d":{}}', IS)),
    },
    {
      input: asBytes('\x00\x01\x02\x03\x04'),
      key: 0xff,
      expected: asBytes(jw('\x00\x01\x02\x03\x04', 0xff)),
    },
  ],
);

// hP — base64 encode (standard alphabet, UTF-8 input)
dump(
  'hP',
  [
    { input: '', expected: '' },
    { input: 'A', expected: hP('A') },
    { input: 'Man', expected: hP('Man') },
    { input: 'Ma', expected: hP('Ma') },
    { input: 'M', expected: hP('M') },
    { input: 'hello world', expected: hP('hello world') },
    { input: 'pedidosya.com.ar', expected: hP('pedidosya.com.ar') },
    { input: '{"vid":"abc-123","tag":"v3.1"}', expected: hP('{"vid":"abc-123","tag":"v3.1"}') },
  ],
);

// vM — linear remap
dump(
  'vM',
  [
    { t: 50, n: 0, e: 100, r: 0, g: 10, expected: vM(50, 0, 100, 0, 10) },
    { t: 12345, n: 0, e: 65536, r: 0, g: 255, expected: vM(12345, 0, 65536, 0, 255) },
    { t: 0, n: 0, e: 1, r: 0, g: 1, expected: vM(0, 0, 1, 0, 1) },
    { t: 9999, n: 100, e: 10000, r: 0, g: 1023, expected: vM(9999, 100, 10000, 0, 1023) },
  ],
);

// vL — secret feed builder; `expected` is a byte array because the
// XOR step pushes some chars above 127.
dump(
  'vL',
  [
    { pf: '', expected: asBytes(vL('')) },
    { pf: 'pedidosya.com.ar', expected: asBytes(vL('pedidosya.com.ar')) },
    {
      pf: 'https://www.pedidosya.com.ar/restaurantes',
      expected: asBytes(vL('https://www.pedidosya.com.ar/restaurantes')),
    },
  ],
);

// vN — offsets PRNG
{
  const tA = 'hello world this is a sensor payload string';
  const sA = 'pedidosya.com.ar';
  const tB = jw('{"events":[{"t":"PX12457"}]}', IS);
  const sB = 'mock-secret-feed';
  const tC = 'A'.repeat(200);
  const sC = 'tenant-eT15wiaE';
  dump('vN', [
    { target: tA, lenBound: tA.length, secret: sA, expected: vN(tA, tA.length, sA) },
    { target: tB, lenBound: tB.length, secret: sB, expected: vN(tB, tB.length, sB) },
    { target: tC, lenBound: tC.length, secret: sC, expected: vN(tC, tC.length, sC) },
  ]);
}

// vQ — splice salt into payload at offsets
{
  const salt = 'pedidosya.com.ar';
  const payload = 'A'.repeat(500);
  const offsets = vN(salt, payload.length, 'mock-secret-feed');
  dump('vQ', [
    { salt, payload, offsets, expected: vQ(salt, payload, offsets) },
    {
      salt: 'tag',
      payload: 'helloworld',
      offsets: vN('tag', 'helloworld'.length, 'seed'),
      expected: vQ('tag', 'helloworld', vN('tag', 'helloworld'.length, 'seed')),
    },
  ]);
}

// vP — end-to-end sensor encryptor. Outputs may contain non-ASCII so
// expected is a byte array (asBytes).
{
  const cases = [
    {
      events: '[{"t":"PX12457","d":{"AzNweUZUfEs=":1716192345678}}]',
      pf: 'pedidosya.com.ar',
      cu: 'b9b7f7a0-deadbeef',
    },
    {
      events: '[{"t":"PX561","d":{}}]',
      pf: 'https://www.pedidosya.com.ar',
      cu: 'short',
    },
    {
      events: '[]',
      pf: '',
      cu: 'cu-empty-pf',
    },
  ];
  dump(
    'vP',
    cases.map((c) => ({
      events_json: c.events,
      pf: c.pf,
      cu: c.cu,
      expected: asBytes(vP(c.events, c.pf, c.cu)),
    })),
  );
}

process.stdout.write('done.\n');
