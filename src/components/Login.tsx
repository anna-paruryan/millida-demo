import { useState } from "react";
import { toast } from "sonner";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Card, CardContent, CardHeader } from "./ui/card";
import { Account } from "@/components/features/auth/Accountmanager.tsx";
import { invoke } from "@/lib/tauri";

const USER_RE = /^[a-zA-Z0-9_]+$/;

interface Props {
    onSuccess: (a: Account) => void;
    onCancel?: () => void;
}

interface AuthResult {
    username: string;
    token: string;
}

export default function Login({ onSuccess, onCancel }: Props) {
    const [mode, setMode] = useState<"login" | "register">("login");
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");
    const [email, setEmail] = useState("");
    const [showPassword, setShowPassword] = useState(false);
    const [errors, setErrors] = useState<Record<string, string>>({});
    const [isLoading, setIsLoading] = useState(false);

    const clearFieldError = (field: string) =>
        setErrors((e) => {
            const n = { ...e };
            delete n[field];
            delete n.form;
            return n;
        });

    const validate = () => {
        const e: Record<string, string> = {};
        if (username.length < 3 || username.length > 16)
            e.username = "Username: 3–16 characters";
        else if (!USER_RE.test(username))
            e.username = "Only letters, digits and _";
        if (password.length < 6) e.password = "Password: at least 6 characters";
        if (mode === "register") {
            if (email.length < 5 || !email.includes("@"))
                e.email = "Valid email required";
        }
        setErrors(e);
        return Object.keys(e).length === 0;
    };

    const submit = async (ev: React.FormEvent) => {
        ev.preventDefault();
        if (isLoading || !validate()) return;
        setIsLoading(true);
        try {
            const cmd = mode === "login" ? "secure_login" : "secure_register";
            const payload =
                mode === "login"
                    ? { username, password }
                    : { username, password, email };

            const r = await invoke<AuthResult>(cmd, payload);

            toast.success(
                mode === "login" ? `Welcome, ${r.username}!` : "Account created!"
            );

            onSuccess({
                username: r.username,
                token: r.token,
                lastLogin: Date.now(),
            });
        } catch (err) {
            const msg = String(err);
            setErrors({ form: msg });
            toast.error(msg);
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="flex min-h-full items-center justify-center p-4 md:p-6">
            <Card className="w-full max-w-sm border-border/50 bg-[#12121a]/95 shadow-xl">
                <CardHeader className="pb-2">
                    <h1 className="text-center text-3xl font-black tracking-[0.2em] text-[#ff2d55]">
                        MILLIDA
                    </h1>
                    <p className="mt-2 text-center text-sm text-[#8a8a9a]">
                        {mode === "login"
                            ? "Sign in to your account"
                            : "Create a new account"}
                    </p>
                </CardHeader>

                <CardContent>
                    <form onSubmit={submit} className="space-y-5" noValidate>
                        {/* Username */}
                        <div className="space-y-1.5">
                            <label
                                htmlFor="u"
                                className="block text-xs font-medium uppercase tracking-wider text-[#8a8a9a]"
                            >
                                Username
                            </label>
                            <Input
                                id="u"
                                value={username}
                                onChange={(e) => {
                                    setUsername(e.target.value);
                                    clearFieldError("username");
                                }}
                                aria-invalid={!!errors.username}
                                placeholder="Anna123"
                                autoComplete="username"
                                className="h-11"
                            />
                            {errors.username && (
                                <p className="text-xs text-[#ff2d55]">{errors.username}</p>
                            )}
                        </div>

                        {/* Email (только register) */}
                        {mode === "register" && (
                            <div className="space-y-1.5">
                                <label
                                    htmlFor="e"
                                    className="block text-xs font-medium uppercase tracking-wider text-[#8a8a9a]"
                                >
                                    Email
                                </label>
                                <Input
                                    id="e"
                                    type="email"
                                    value={email}
                                    onChange={(e) => {
                                        setEmail(e.target.value);
                                        clearFieldError("email");
                                    }}
                                    aria-invalid={!!errors.email}
                                    placeholder="you@example.com"
                                    autoComplete="email"
                                    className="h-11"
                                />
                                {errors.email && (
                                    <p className="text-xs text-[#ff2d55]">{errors.email}</p>
                                )}
                            </div>
                        )}

                        {/* Password */}
                        <div className="space-y-1.5">
                            <label
                                htmlFor="p"
                                className="block text-xs font-medium uppercase tracking-wider text-[#8a8a9a]"
                            >
                                Password
                            </label>
                            <div className="relative">
                                <Input
                                    id="p"
                                    type={showPassword ? "text" : "password"}
                                    value={password}
                                    onChange={(e) => {
                                        setPassword(e.target.value);
                                        clearFieldError("password");
                                    }}
                                    aria-invalid={!!errors.password}
                                    placeholder="••••••••"
                                    autoComplete={
                                        mode === "login" ? "current-password" : "new-password"
                                    }
                                    className="h-11 pr-10"
                                />
                                <button
                                    type="button"
                                    onClick={() => setShowPassword((v) => !v)}
                                    className="absolute right-3 top-1/2 -translate-y-1/2 text-[#8a8a9a] hover:text-white transition"
                                    aria-label={showPassword ? "Hide password" : "Show password"}
                                >
                                    {showPassword ? "🙈" : "👁"}
                                </button>
                            </div>
                            {errors.password && (
                                <p className="text-xs text-[#ff2d55]">{errors.password}</p>
                            )}
                        </div>

                        {errors.form && (
                            <p className="rounded-lg bg-[#ff2d55]/10 px-3 py-2 text-sm text-[#ff2d55]">
                                {errors.form}
                            </p>
                        )}

                        <Button
                            type="submit"
                            disabled={isLoading}
                            className="h-11 w-full text-sm font-semibold tracking-wide"
                        >
                            {isLoading
                                ? mode === "login"
                                    ? "Signing in…"
                                    : "Creating…"
                                : mode === "login"
                                    ? "LOGIN"
                                    : "CREATE ACCOUNT"}
                        </Button>

                        <button
                            type="button"
                            className="w-full text-center text-sm text-[#8a8a9a] hover:text-white transition"
                            onClick={() => {
                                setMode(mode === "login" ? "register" : "login");
                                setErrors({});
                            }}
                            disabled={isLoading}
                        >
                            {mode === "login"
                                ? "No account? Create one"
                                : "Already have an account? Sign in"}
                        </button>

                        {onCancel && (
                            <Button
                                type="button"
                                variant="ghost"
                                className="w-full"
                                onClick={onCancel}
                                disabled={isLoading}
                            >
                                Cancel
                            </Button>
                        )}

                        {mode === "login" && (
                            <p className="pt-1 text-center text-[11px] text-[#6a6a7a]">
                                Demo: Anna123 / anna12345
                            </p>
                        )}
                    </form>
                </CardContent>
            </Card>
        </div>
    );
}