export interface Account { username: string; token: string; lastLogin: number }
export interface AccountState { millida_accounts: Account[]; active_user: string | null }

export const MAX_ACCOUNTS = 5;
export const emptyState: AccountState = { millida_accounts: [], active_user: null };

/** Добавляет/обновляет аккаунт и делает его активным. Лимит — 5. */
//src/components/features/auth/Accountmanager.tsx
export function addAccount(s: AccountState, a: Account): AccountState {
    const exists = s.millida_accounts.some((x) => x.username === a.username);
    if (!exists && s.millida_accounts.length >= MAX_ACCOUNTS) throw new Error(`Maximum ${MAX_ACCOUNTS} accounts`);
    const list = exists
        ? s.millida_accounts.map((x) => (x.username === a.username ? a : x))
        : [...s.millida_accounts, a];
    return { millida_accounts: list, active_user: a.username };
}

/** Переключение без пароля — токен уже сохранён. */
export function switchAccount(s: AccountState, username: string): AccountState {
    return s.millida_accounts.some((x) => x.username === username) ? { ...s, active_user: username } : s;
}

/** Logout только активного; остальные остаются. */
export function logoutActive(s: AccountState): AccountState {
    const rest = s.millida_accounts.filter((x) => x.username !== s.active_user);
    return { millida_accounts: rest, active_user: rest[0]?.username ?? null };
}

export const getActive = (s: AccountState) => s.millida_accounts.find((x) => x.username === s.active_user) ?? null;