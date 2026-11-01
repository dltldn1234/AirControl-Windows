export type KeyboardKey = { id: string; label: string; korean?: string; shifted?: string; kind?: "modifier" | "action"; width?: number };

export const KEYBOARD_ROWS: KeyboardKey[][] = [
  [..."1234567890"].map((id, index) => ({ id, label: id, shifted: "!@#$%^&*()"[index] })),
  [..."qwertyuiop"].map((id, index) => ({ id, label: id.toUpperCase(), korean: "ㅂㅈㄷㄱㅅㅛㅕㅑㅐㅔ"[index] })),
  [..."asdfghjkl"].map((id, index) => ({ id, label: id.toUpperCase(), korean: "ㅁㄴㅇㄹㅎㅗㅓㅏㅣ"[index] })),
  [
    { id: "caps", label: "Caps", kind: "modifier" as const, width: 1.35 },
    ...[..."zxcvbnm"].map((id, index) => ({ id, label: id.toUpperCase(), korean: "ㅋㅌㅊㅍㅠㅜㅡ"[index] })),
    { id: "backspace", label: "⌫", kind: "action" as const, width: 1.5 },
  ],
  [
    { id: "shift", label: "Shift", kind: "modifier" as const, width: 1.5 },
    { id: "layout", label: "한/영", kind: "action" as const, width: 1.25 },
    { id: "-", label: "-" }, { id: ",", label: "," }, { id: "space", label: "Space", kind: "action" as const, width: 4.5 },
    { id: ".", label: "." }, { id: "/", label: "/" }, { id: "?", label: "?" },
    { id: "enter", label: "Enter", kind: "action" as const, width: 1.5 },
  ],
];

export function commandKey(key: KeyboardKey): string {
  return key.id;
}
