// Blues, slates and earth tones only: red means error, aqua means selection, green means online.
// White initials stay above 4.5:1 on every fill.
const fills = [
  "#34548c",
  "#2f6690",
  "#56657f",
  "#4b5363",
  "#94681a",
  "#8a5a2b",
  "#6f6a2e",
  "#7a6a52",
];

// FNV-1a: spreads short, similar names across the palette.
export function avatarFill(name: string): string {
  let hash = 0x811c9dc5;
  for (const ch of name) {
    hash ^= ch.codePointAt(0)!;
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return fills[hash % fills.length];
}

export function initial(name: string): string {
  return ([...name.trim()][0] ?? "?").toUpperCase();
}
