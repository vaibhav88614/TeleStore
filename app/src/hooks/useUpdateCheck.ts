import { useCallback } from 'react';

// Auto-update is intentionally disabled in this personalized build.
//
// The upstream project ships a Tauri updater that trusts a single minisign
// public key controlled by the original author and pulls release artifacts
// from their GitHub repo. Leaving that enabled would let an external party
// push arbitrary code (including the ad gateway that was removed) to this
// build. This hook is now an inert stub so the UI wiring keeps working while
// no network update check ever runs.
//
// To re-enable auto-update for your OWN distribution:
//   1. Generate your own key:  npx @tauri-apps/cli signer generate
//   2. Put the public key + your release endpoint back in tauri.conf.json
//   3. Re-add tauri_plugin_updater in src-tauri/src/lib.rs and the
//      "updater:default" capability, then restore the original plugin-based
//      implementation of this hook.

interface UpdateState {
    checking: boolean;
    available: boolean;
    downloading: boolean;
    progress: number;
    error: string | null;
    version: string | null;
}

const INERT_STATE: UpdateState = {
    checking: false,
    available: false,
    downloading: false,
    progress: 0,
    error: null,
    version: null,
};

export function useUpdateCheck() {
    const noop = useCallback(async () => {}, []);
    const dismissUpdate = useCallback(() => {}, []);

    return {
        ...INERT_STATE,
        checkForUpdates: noop,
        downloadAndInstall: noop,
        dismissUpdate,
    };
}
