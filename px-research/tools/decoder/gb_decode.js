// Reference impl of the px-3 `gB(t)` base-91-ish decoder.
// Mirrors the JS exactly (see px-research/deobf/eT15wiaE/2026-05-19-init.webcrack.js).
// Two alphabets observed in the 2026-05-19 capture:
//   ALPHABET_OUTER — used by the top-level gB at the module entry
//   ALPHABET_VM    — used by the VM-style inner decoder

'use strict';

const ALPHABET_OUTER =
  'RgbH8zv3:#<yhG17YJs}TM_"x~49+mI;`jAedPp]C)F=ouQ@E?,KX!nq6$iZDlcSW0aBwkU.V(2^*fLrtN&[|O%5{/>';
const ALPHABET_VM =
  '50}.!w*?x@~62Nv_VKP|G%=4jot9#m:q;8i$,)kW1LE<F`rDM&QH[lIahZ]S>JCzYuBcARXe+g/7^Un{pbd3Tyf(Os';

function decodeWith(alphabet, t) {
  const n = '' + (t || '');
  const e = n.length;
  const r = [];
  let g = 0;
  let o = 0;
  let i = -1;
  for (let a = 0; a < e; a++) {
    const c = alphabet.indexOf(n[a]);
    if (c === -1) continue;
    if (i < 0) {
      i = c;
    } else {
      i += c * 91;
      g |= i << o;
      o += (i & 8191) > 88 ? 13 : 14;
      do {
        r.push(g & 255);
        g >>= 8;
        o -= 8;
      } while (o > 7);
      i = -1;
    }
  }
  if (i > -1) {
    r.push((g | (i << o)) & 255);
  }
  // gN() in the captured source is identity-on-bytes-to-string for the
  // top-level usage. Lower the byte array to a JS string (latin1).
  return Buffer.from(r).toString('latin1');
}

module.exports = {
  ALPHABET_OUTER,
  ALPHABET_VM,
  decode: (t) => decodeWith(ALPHABET_OUTER, t),
  decodeOuter: (t) => decodeWith(ALPHABET_OUTER, t),
  decodeVM: (t) => decodeWith(ALPHABET_VM, t),
};
