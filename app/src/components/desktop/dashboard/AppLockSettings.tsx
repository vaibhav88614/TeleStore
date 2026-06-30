import { useState } from 'react';
import { Lock, ShieldCheck, ShieldOff } from 'lucide-react';
import { useAppLock } from '../../../hooks/useAppLock';

/**
 * Self-contained "App Lock" settings section (Track C).
 * Lets the user enable a local PIN gate or remove it. Encapsulated as its own
 * component so it can be dropped into the large SettingsModal with a single line.
 */
export function AppLockSettings() {
    const { enabled, setPin, disablePin } = useAppLock();
    const [pin, setPinInput] = useState('');
    const [confirmPin, setConfirmPin] = useState('');
    const [currentPin, setCurrentPin] = useState('');
    const [msg, setMsg] = useState<{ kind: 'ok' | 'err'; text: string } | null>(null);
    const [busy, setBusy] = useState(false);

    const enable = async () => {
        setMsg(null);
        if (pin.length < 4) {
            setMsg({ kind: 'err', text: 'PIN must be at least 4 characters.' });
            return;
        }
        if (pin !== confirmPin) {
            setMsg({ kind: 'err', text: 'PINs do not match.' });
            return;
        }
        setBusy(true);
        await setPin(pin);
        setBusy(false);
        setPinInput('');
        setConfirmPin('');
        setMsg({ kind: 'ok', text: 'App lock enabled.' });
    };

    const disable = async () => {
        setMsg(null);
        setBusy(true);
        const ok = await disablePin(currentPin);
        setBusy(false);
        setCurrentPin('');
        setMsg(ok
            ? { kind: 'ok', text: 'App lock disabled.' }
            : { kind: 'err', text: 'Incorrect PIN.' });
    };

    const inputCls =
        'w-full py-2 px-3 rounded-lg bg-telegram-hover/40 border border-telegram-border focus:border-telegram-primary outline-none text-sm';

    return (
        <section className="space-y-3">
            <h3 className="text-xs font-semibold text-telegram-subtext uppercase tracking-wider flex items-center gap-2">
                <Lock className="w-3.5 h-3.5" />
                App Lock
            </h3>

            <p className="text-xs text-telegram-subtext">
                Require a PIN to open the app. This is a local guard only — it does not
                encrypt your Telegram session on disk.
            </p>

            {!enabled ? (
                <div className="space-y-2">
                    <input
                        type="password"
                        inputMode="numeric"
                        placeholder="New PIN (min 4)"
                        value={pin}
                        onChange={(e) => setPinInput(e.target.value)}
                        className={inputCls}
                    />
                    <input
                        type="password"
                        inputMode="numeric"
                        placeholder="Confirm PIN"
                        value={confirmPin}
                        onChange={(e) => setConfirmPin(e.target.value)}
                        className={inputCls}
                    />
                    <button
                        onClick={enable}
                        disabled={busy}
                        className="w-full py-2 rounded-lg font-semibold bg-telegram-primary text-black disabled:opacity-40 flex items-center justify-center gap-2"
                    >
                        <ShieldCheck className="w-4 h-4" /> Enable App Lock
                    </button>
                </div>
            ) : (
                <div className="space-y-2">
                    <input
                        type="password"
                        inputMode="numeric"
                        placeholder="Current PIN"
                        value={currentPin}
                        onChange={(e) => setCurrentPin(e.target.value)}
                        className={inputCls}
                    />
                    <button
                        onClick={disable}
                        disabled={busy}
                        className="w-full py-2 rounded-lg font-semibold border border-telegram-border hover:bg-telegram-hover disabled:opacity-40 flex items-center justify-center gap-2"
                    >
                        <ShieldOff className="w-4 h-4" /> Disable App Lock
                    </button>
                </div>
            )}

            {msg && (
                <p className={`text-xs ${msg.kind === 'ok' ? 'text-green-400' : 'text-red-400'}`}>
                    {msg.text}
                </p>
            )}
        </section>
    );
}
