// Standalone JS reference implementation of the eT15wiaE cipher primitives.
// Each function is a direct port of the un-flattened body from
// px-research/deobf/eT15wiaE/main-pass2/deobfuscated.js, lines:
//   jw  — 1882   |   hP/hQ — 596/645   |   vL — 5982   |   vJ — 5980
//   vM — 5985    |   vN    — 5988      |   vQ — 6065
//
// Used to emit fixtures for the Rust port (`px-native/src/cipher/`).

'use strict';

const HI = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=';
const VJ = 10;       // secret-feed XOR key
const IS = 50;       // payload-cipher XOR key

function jw(t, n) {
  let e = '';
  for (let r = 0; r < t.length; r++) {
    e += String.fromCharCode(n ^ t.charCodeAt(r));
  }
  return e;
}

// hQ: base64 encode (string in → base64 string out).
function hP(t) {
  if (!t) return t;
  const utf8 = unescape(encodeURIComponent(t));
  const u = [];
  let a = 0;
  let c = 0;
  while (a < utf8.length) {
    const o = (utf8.charCodeAt(a++) << 16) | (utf8.charCodeAt(a++) << 8) | utf8.charCodeAt(a++);
    u[c++] =
      HI.charAt((o >> 18) & 63) +
      HI.charAt((o >> 12) & 63) +
      HI.charAt((o >> 6) & 63) +
      HI.charAt(o & 63);
  }
  const f = u.join('');
  const l = utf8.length % 3;
  return (l ? f.slice(0, l - 3) : f) + '==='.slice(l || 3);
}

function vM(t, n, e, r, g) {
  return Math.floor(((t - n) / (e - n)) * (g - r) + r);
}

// vL produces the secret feed. The captured implementation pulls
// `pf()` (page fingerprint) at call time; for fixture purposes the
// caller passes the same input the runtime would.
function vL(pf) {
  return jw(hP(pf || ''), VJ);
}

// vN — un-flattened from VM. See r3-sensor-grammar notes.
function vN(target, lenBound, secret) {
  const aF = jw(hP(secret), VJ);
  let aH = -1;
  for (let aI = 0; aI < target.length; aI++) {
    const aJ = Math.floor(aI / aF.length + 1);
    const aK = aI >= aF.length ? aI % aF.length : aI;
    const aL = aF.charCodeAt(aK) * aF.charCodeAt(aJ);
    if (aL > aH) aH = aL;
  }
  const aG = [];
  for (let aM = 0; aM < target.length; aM++) {
    const aN = Math.floor(aM / aF.length) + 1;
    const aO = aM % aF.length;
    let aP = aF.charCodeAt(aO) * aF.charCodeAt(aN);
    if (aP >= lenBound) aP = vM(aP, 0, aH, 0, lenBound - 1);
    while (aG.indexOf(aP) !== -1) aP += 1;
    aG.push(aP);
  }
  return aG.sort((t, n) => t - n);
}

// vQ — splice salt characters into payload at the offsets vN produced.
function vQ(salt, payload, offsets) {
  const bf = salt.split('');
  let bd = '';
  let be = 0;
  for (let bg = 0; bg < salt.length; bg++) {
    bd += payload.substring(be, offsets[bg] - bg - 1) + bf[bg];
    be = offsets[bg] - bg - 1;
  }
  bd += payload.substring(be);
  return bd;
}

module.exports = { HI, VJ, IS, jw, hP, vM, vL, vN, vQ };
