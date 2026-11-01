import { describe, expect, it } from "vitest";
import { commandKey, KEYBOARD_ROWS } from "./keyboardLayout";

describe("Air Keyboard layout", () => {
  it("shows English and Dubeolsik legends on the same QWERTY key positions", () => {
    const qwerty = KEYBOARD_ROWS[1];
    expect(qwerty[0]).toMatchObject({ id: "q", label: "Q", korean: "ㅂ" });
    expect(qwerty[3]).toMatchObject({ id: "r", label: "R", korean: "ㄱ" });
    expect(KEYBOARD_ROWS[2][0]).toMatchObject({ id: "a", korean: "ㅁ" });
    expect(KEYBOARD_ROWS[3][1]).toMatchObject({ id: "z", korean: "ㅋ" });
  });

  it("includes editing controls, number row, and unambiguous command IDs", () => {
    expect(KEYBOARD_ROWS[0].map((key) => key.id)).toEqual(["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"]);
    const controls = KEYBOARD_ROWS.flat().map((key) => key.id);
    expect(controls).toEqual(expect.arrayContaining(["caps", "shift", "layout", "space", "enter", "backspace"]));
    expect(commandKey({ id: ",", label: "," })).toBe(",");
    expect(commandKey({ id: "?", label: "?" })).toBe("?");
    expect(KEYBOARD_ROWS[0][1].shifted).toBe("@");
  });
});
