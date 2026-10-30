import { useState } from "react";
import { Toaster } from "sonner";
import Login from "./components/Login";
import Launcher from "./components/Launcher";
import {
    Account,
    AccountState,
    addAccount,
    emptyState,
    getActive,
    logoutActive,
    switchAccount,
} from "./components/features/auth/Accountmanager.tsx";

export default function App() {
    const [state, setState] = useState<AccountState>(emptyState);
    const [adding, setAdding] = useState(false);

    const onLogin = (a: Account) => {
        try {
            setState((s) => addAccount(s, a));
            setAdding(false);
        } catch (e) {
            alert(String(e));
        }
    };

    return (
        <>
            {!getActive(state) || adding ? (
                <Login
                    onSuccess={onLogin}
                    onCancel={adding ? () => setAdding(false) : undefined}
                />
            ) : (
                <Launcher
                    state={state}
                    onSwitch={(u) => setState((s) => switchAccount(s, u))}
                    onAdd={() => setAdding(true)}
                    onLogout={() => setState(logoutActive)}
                />
            )}

            <Toaster
                theme="dark"
                position="top-center"
                richColors
                closeButton
            />
        </>
    );
}