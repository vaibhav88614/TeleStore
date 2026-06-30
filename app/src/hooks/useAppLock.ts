import { useCallback, useEffect, useState } from 'react';
import { load } from '@tauri-apps/plugin-store';

// App-level PIN lock (Track C).
//
// This is a LOCAL gate in front of the dashboard, independent of the Telegram
// session. The grammers session on disk still grants account access to anyone
// with filesystem access (see docs/E2E_ENCRYPTION.md for the at-rest plan), but
// this PIN prevents casual "walk-up" access to an unlocked, logged-in app.
//
// The PIN is never stored. We persist only a random salt and the PBKDF2-SHA256
// derivation of the PIN, and compare derivations on unlock.

const STORE_FILE = 'config.json';
const ENABLED_KEY = 'app_lock_enabled';
const SALT_KEY = 'app_lock_salt';
const HASH_KEY = 'app_lock_hash';
const PBKDF2_ITERATIONS = 210_000;

function toHex(buf: ArrayBuffer): string {
    return Array.from(new Uint8Array(buf))
        .map((b) => b.toString(16).padStart(2, '0'))
        .join('');
}

function randomSaltHex(): string {
    const salt = new Uint8Array(16);
    crypto.getRandomValues(salt);
    return Array.from(salt).map((b) => b.toString(16).padStart(2, '0')).join('');
}

async function derive(pin: string, saltHex: string): Promise<string> {
    const enc = new TextEncoder();
    const salt = new Uint8Array((saltHex.match(/.{2}/g) || []).map((h) => parseInt(h, 16)));
    const keyMaterial = await crypto.subtle.importKey(
        'raw',
        enc.encode(pin),
        { name: 'PBKDF2' },
        false,
        ['deriveBits'],
    );
    const bits = await crypto.subtle.deriveBits(
        { name: 'PBKDF2', salt, iterations: PBKDF2_ITERATIONS, hash: 'SHA-256' },
        keyMaterial,
        256,
    );
    return toHex(bits);
}

interface AppLockState {
    enabled: boolean;
    isLoaded: boolean;
    unlocked: boolean;
}

export function useAppLock() {
    const [state, setState] = useState<AppLockState>({
        enabled: false,
        isLoaded: false,
        unlocked: false,
    });

    useEffect(() => {
        let cancelled = false;
        (async () => {
            try {
                const store = await load(STORE_FILE);
                const enabled = (await store.get<boolean>(ENABLED_KEY)) ?? false;
                if (!cancelled) {
                    // If lock is enabled the app starts locked; otherwise it's open.
                    setState({ enabled, isLoaded: true, unlocked: !enabled });
                }
            } catch {
                if (!cancelled) setState({ enabled: false, isLoaded: true, unlocked: true });
            }
        })();
        return () => { cancelled = true; };
    }, []);

    const unlock = useCallback(async (pin: string): Promise<boolean> => {
        try {
            const store = await load(STORE_FILE);
            const salt = await store.get<string>(SALT_KEY);
            const hash = await store.get<string>(HASH_KEY);
            if (!salt || !hash) return false;
            const candidate = await derive(pin, salt);
            const ok = candidate === hash;
            if (ok) setState((s) => ({ ...s, unlocked: true }));
            return ok;
        } catch {
            return false;
        }
    }, []);

    const setPin = useCallback(async (pin: string): Promise<void> => {
        const store = await load(STORE_FILE);
        const salt = randomSaltHex();
        const hash = await derive(pin, salt);
        await store.set(SALT_KEY, salt);
        await store.set(HASH_KEY, hash);
        await store.set(ENABLED_KEY, true);
        await store.save();
        setState((s) => ({ ...s, enabled: true, unlocked: true }));
    }, []);

    const disablePin = useCallback(async (pin: string): Promise<boolean> => {
        try {
            const store = await load(STORE_FILE);
            const salt = await store.get<string>(SALT_KEY);
            const hash = await store.get<string>(HASH_KEY);
            if (salt && hash) {
                const candidate = await derive(pin, salt);
                if (candidate !== hash) return false;
            }
            await store.set(ENABLED_KEY, false);
            await store.delete(SALT_KEY);
            await store.delete(HASH_KEY);
            await store.save();
            setState((s) => ({ ...s, enabled: false, unlocked: true }));
            return true;
        } catch {
            return false;
        }
    }, []);

    return { ...state, unlock, setPin, disablePin };
}
