import { useState, useRef, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Lock, Loader2 } from 'lucide-react';

interface LockScreenProps {
    onUnlock: (pin: string) => Promise<boolean>;
}

/**
 * Full-screen PIN gate shown when the app lock is enabled and the app is locked.
 * Independent of the Telegram session — purely a local "walk-up" guard.
 */
export function LockScreen({ onUnlock }: LockScreenProps) {
    const [pin, setPin] = useState('');
    const [error, setError] = useState<string | null>(null);
    const [checking, setChecking] = useState(false);
    const inputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
        const id = setTimeout(() => inputRef.current?.focus(), 50);
        return () => clearTimeout(id);
    }, []);

    const submit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!pin || checking) return;
        setChecking(true);
        setError(null);
        const ok = await onUnlock(pin);
        setChecking(false);
        if (!ok) {
            setError('Incorrect PIN');
            setPin('');
            inputRef.current?.focus();
        }
    };

    return (
        <main className="h-screen w-screen flex items-center justify-center bg-telegram-bg text-telegram-text">
            <motion.form
                onSubmit={submit}
                initial={{ opacity: 0, scale: 0.96 }}
                animate={{ opacity: 1, scale: 1 }}
                className="auth-glass rounded-3xl shadow-2xl w-full max-w-sm p-8 text-center"
            >
                <div className="flex justify-center mb-4">
                    <div className="w-14 h-14 rounded-2xl bg-telegram-primary/15 flex items-center justify-center">
                        <Lock className="w-7 h-7 text-telegram-primary" />
                    </div>
                </div>
                <h1 className="text-lg font-bold mb-1">App Locked</h1>
                <p className="text-sm text-telegram-subtext mb-6">Enter your PIN to continue</p>

                <input
                    ref={inputRef}
                    type="password"
                    inputMode="numeric"
                    autoComplete="off"
                    value={pin}
                    onChange={(e) => setPin(e.target.value)}
                    placeholder="••••••"
                    className="w-full text-center tracking-[0.4em] text-lg py-3 rounded-xl bg-telegram-hover/40 border border-telegram-border focus:border-telegram-primary outline-none"
                />

                {error && <p className="text-red-400 text-sm mt-3">{error}</p>}

                <button
                    type="submit"
                    disabled={!pin || checking}
                    className="mt-5 w-full py-3 rounded-xl font-semibold bg-telegram-primary text-black disabled:opacity-40 flex items-center justify-center gap-2"
                >
                    {checking ? <Loader2 className="w-4 h-4 animate-spin" /> : 'Unlock'}
                </button>
            </motion.form>
        </main>
    );
}
