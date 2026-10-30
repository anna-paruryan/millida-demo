//C:\Users\annap\Desktop\millida-demo\src-tauri\src\tauri.ts
import { invoke as tauriInvoke } from "@tauri-apps/api/core";

/** true только внутри окна Tauri; в обычном браузере (localhost:1420) будет false */
export const isTauri = () =>
    typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

const DEMO_USERS = ["Anna123", "Petya2005"];

/** Моки Rust-команд — используются ТОЛЬКО в браузере, для разработки интерфейса */
const mocks: Record<string, (args: any) => unknown> = {
    secure_login: ({ username, password }) => {
        const user = DEMO_USERS.find(
            (u) => u.toLowerCase() === String(username).toLowerCase()
        );
        if (!user) throw "User not found (demo: Anna123 or Petya2005)";
        if (String(password).length < 6) throw "Wrong password";
        return { username: user, token: `mock-token-${user}` };
    },

    secure_register: ({ username, password, email }) => {
        if (!email || String(password).length < 6) throw "Invalid data";
        return { username, token: `mock-token-${username}` };
    },

    validate_skin_upload: ({ bytes }) => {
        const png = [0x89, 0x50, 0x4e, 0x47];
        if (!png.every((b, i) => bytes[i] === b)) throw "Not a valid PNG file";
        return "Skin is valid (browser mock)";
    },

    secure_launch: ({ serverIp }) => `Launch is available only in the app (${serverIp})`,
};

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    if (isTauri()) return tauriInvoke<T>(cmd, args);

    const mock = mocks[cmd];
    if (!mock) throw `Command "${cmd}" is available only in the Tauri app`;
    await new Promise((r) => setTimeout(r, 300)); // имитация задержки
    return mock(args ?? {}) as T;
}