import { useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { Button } from "./ui/button";
import { Card, CardContent } from "./ui/card";
import {AccountState, getActive, MAX_ACCOUNTS} from "@/components/features/auth/Accountmanager.tsx";
import {invoke} from "@/lib/tauri.ts";




const SERVERS = [
    { name: "Classic", ip: "classic.millida.net:25565", online: 124 },
    { name: "PvP", ip: "pvp.millida.net:25565", online: 89 },
    { name: "Vanilla", ip: "vanilla.millida.net:25565", online: 45 },
];

const MAX_SKIN_SIZE = 1_048_576; // 1 MB

function makeSkin(seed: string): { url: string; bytes: number[] } {
    const c = document.createElement("canvas");
    c.width = c.height = 64;
    const g = c.getContext("2d")!;
    const hue = [...seed].reduce((a, ch) => a + ch.charCodeAt(0), 0) % 360;
    g.fillStyle = `hsl(${hue} 60% 45%)`;
    g.fillRect(0, 0, 64, 64);
    g.fillStyle = `hsl(${hue} 50% 75%)`;
    g.fillRect(8, 8, 8, 8);
    g.fillStyle = "#0a0a0f";
    g.fillRect(9, 11, 2, 2);
    g.fillRect(13, 11, 2, 2);
    const url = c.toDataURL("image/png");
    const bin = atob(url.split(",")[1]);
    return { url, bytes: Array.from(bin, (ch) => ch.charCodeAt(0)) };
}

interface Props {
    state: AccountState;
    onSwitch: (u: string) => void;
    onAdd: () => void;
    onLogout: () => void;
}

export default function Launcher({ state, onSwitch, onAdd, onLogout }: Props) {
    const active = getActive(state)!;
    const [server, setServer] = useState(SERVERS[0]);
    const [status, setStatus] = useState("");
    const [launching, setLaunching] = useState(false);
    const [skinMsg, setSkinMsg] = useState("");
    const [skinUrl, setSkinUrl] = useState("");
    const [skinBytes, setSkinBytes] = useState<number[]>([]);

    // Храним blob: URL загруженного файла, чтобы освобождать память
    const blobUrlRef = useRef<string | null>(null);

    const revokeBlobUrl = () => {
        if (blobUrlRef.current) {
            URL.revokeObjectURL(blobUrlRef.current);
            blobUrlRef.current = null;
        }
    };

    // При смене аккаунта — просто генерируем мок-скин. Никаких invoke.
    useEffect(() => {
        revokeBlobUrl();
        const mock = makeSkin(active.username);
        setSkinUrl(mock.url);
        setSkinBytes(mock.bytes);
        setSkinMsg("");
    }, [active.username]);

    // Очистка при размонтировании
    useEffect(() => revokeBlobUrl, []);

    // Загрузка PNG: проверка на фронте -> validate_skin_upload -> toast + skinMsg
    const onSkinFile = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        e.target.value = ""; // чтобы можно было выбрать тот же файл повторно
        if (!file) return;

        if (file.type !== "image/png" && !file.name.toLowerCase().endsWith(".png")) {
            toast.error("Only PNG files are allowed");
            return;
        }

        if (file.size >= MAX_SKIN_SIZE) {
            toast.error("Skin must be smaller than 1 MB");
            return;
        }

        const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));

        setSkinMsg("Checking skin…");
        try {
            const msg = await invoke<string>("validate_skin_upload", {
                fileName: file.name,
                bytes,
            });

            // Скин применяем только после успешной проверки
            revokeBlobUrl();
            const url = URL.createObjectURL(file);
            blobUrlRef.current = url;
            setSkinUrl(url);
            setSkinBytes(bytes);

            setSkinMsg(`✓ ${msg}`);
            toast.success(msg);
        } catch (err) {
            const msg = String(err);
            setSkinMsg(`✗ ${msg}`);
            toast.error(msg);
        }
    };

    // Скачать скин через браузер (без Tauri-плагинов)
    const downloadSkin = () => {
        if (!skinBytes.length) {
            toast.error("No skin to download");
            return;
        }

        try {
            const blob = new Blob([new Uint8Array(skinBytes)], { type: "image/png" });
            const url = URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = `${active.username}.png`;
            document.body.appendChild(a);
            a.click();
            a.remove();
            setTimeout(() => URL.revokeObjectURL(url), 1000);
            toast.success("Skin downloaded");
        } catch {
            toast.error("Failed to download skin");
        }
    };

    const play = async () => {
        if (launching) return;
        setLaunching(true);
        setStatus("Launching…");
        try {
            setStatus(
                await invoke<string>("secure_launch", {
                    serverIp: server.ip,
                    token: active.token,
                })
            );
        } catch (e) {
            setStatus(`Error: ${e}`);
        } finally {
            setLaunching(false);
        }
    };

    return (
        <div className="flex min-h-full flex-col gap-4 p-4 md:p-6">
            <header className="flex flex-wrap items-center justify-between gap-3">
                <h1 className="text-xl font-black tracking-[0.2em] text-accent md:text-2xl">
                    MILLIDA
                </h1>
                <div className="flex flex-wrap items-center gap-2">
                    <select
                        aria-label="Switch account"
                        value={active.username}
                        onChange={(e) => onSwitch(e.target.value)}
                        className="rounded-xl border border-border bg-[#14141c] px-3 py-2 text-sm text-white outline-none focus:border-accent"
                    >
                        {state.millida_accounts.map((a) => (
                            <option key={a.username} value={a.username}>
                                {a.username}
                            </option>
                        ))}
                    </select>
                    <Button
                        variant="outline"
                        onClick={onAdd}
                        disabled={state.millida_accounts.length >= MAX_ACCOUNTS}
                    >
                        + Add account
                    </Button>
                    <Button variant="ghost" onClick={onLogout}>
                        Logout
                    </Button>
                </div>
            </header>

            <div className="flex flex-1 flex-col gap-4 lg:flex-row">
                {/* Серверы */}
                <Card className="lg:w-1/2">
                    <CardContent>
                        <h2 className="mb-3 text-base font-bold md:text-lg">Servers</h2>
                        <ul className="space-y-2">
                            {SERVERS.map((s) => (
                                <li key={s.name}>
                                    <button
                                        onClick={() => setServer(s)}
                                        className={`flex w-full items-center justify-between rounded-xl border px-4 py-3 text-left text-sm transition md:text-base ${
                                            server.name === s.name
                                                ? "border-accent bg-accent/10"
                                                : "border-border hover:bg-white/5"
                                        }`}
                                    >
                                        <span className="font-semibold">{s.name}</span>
                                        <span className="text-muted">{s.online} online</span>
                                    </button>
                                </li>
                            ))}
                        </ul>
                    </CardContent>
                </Card>

                {/* Профиль + скин */}
                <Card className="lg:w-1/2">
                    <CardContent className="flex flex-col items-center gap-4 text-center">
                        <img
                            src={skinUrl}
                            width={64}
                            height={64}
                            alt={`${active.username} skin`}
                            style={{ imageRendering: "pixelated" }}
                            className="rounded-xl border border-border"
                        />

                        {/* Upload + Download */}
                        <div className="flex gap-4 text-xs">
                            <label className="cursor-pointer text-[#8a8a9a] underline transition hover:text-white">
                                Upload skin
                                <input
                                    type="file"
                                    accept="image/png"
                                    className="hidden"
                                    onChange={onSkinFile}
                                />
                            </label>

                            <button
                                type="button"
                                onClick={downloadSkin}
                                className="text-[#8a8a9a] underline transition hover:text-white"
                            >
                                Download skin
                            </button>
                        </div>

                        <div>
                            <p className="text-lg font-bold md:text-xl">{active.username}</p>
                            <p className="text-xs text-muted">
                                Last login: {new Date(active.lastLogin).toLocaleTimeString()}
                            </p>
                            <p className="mt-1 min-h-4 text-xs text-muted">{skinMsg}</p>
                        </div>

                        <p className="text-sm text-muted">
                            Saved accounts ({state.millida_accounts.length}/{MAX_ACCOUNTS}):{" "}
                            {state.millida_accounts.map((a) => a.username).join(", ")}
                        </p>

                        <Button
                            onClick={play}
                            disabled={launching}
                            className="w-full max-w-xs py-3"
                        >
                            {launching ? "Launching…" : `PLAY · ${server.name}`}
                        </Button>

                        {status && <p className="text-sm text-fg">{status}</p>}
                    </CardContent>
                </Card>
            </div>
        </div>
    );
}