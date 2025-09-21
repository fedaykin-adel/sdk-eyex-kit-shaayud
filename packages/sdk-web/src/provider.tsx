// packages/sdk-web/src/provider.tsx
"use client";

import React, { createContext, useContext, useEffect, useMemo, useRef, useState } from "react";
import { fremenRecord } from "./client/captura";

type SdkProviderOptions = {
  host: string;
  endpoint?: string; // default: /identity/injest
  autoMouse?: boolean | { maxMs?: number; targetFps?: number; sendIntervalMs?: number };
};

type TrackPayload = Record<string, any>;
type SdkApi = {
  track: (type: string, payload?: TrackPayload) => void;
  identify: (user: { userId?: string; traits?: Record<string, any> }) => void;
  setContext: (ctx: Record<string, any>) => void;
  flush: () => Promise<void>;
  getSessionId: () => string | null;
};

// ✅ CONTEXTO precisa existir neste escopo (fora do componente)
const SdkContext = createContext<SdkApi | null>(null);

// SSR-safe: só usa navigator se existir
async function sendEvent(host: string, endpoint: string, sessionId: string, body: any) {
  const url = new URL(endpoint, host).toString();
  try {
    if (typeof navigator !== "undefined" && "sendBeacon" in navigator) {
      const blob = new Blob([JSON.stringify(body)], { type: "application/json" });
      const ok = (navigator as any).sendBeacon(url, blob);
      if (ok) return;
    }
  } catch {}
  await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json", "x-session-id": sessionId },
    body: JSON.stringify(body),
    credentials: "omit",
    keepalive: true,
  });
}

export function SdkProvider({ options, children }: { options: SdkProviderOptions; children: React.ReactNode }) {
  const { host, endpoint = "/identity/injest", autoMouse = false } = options;

  const initializedRef = useRef(false);
  const stopMouseRef = useRef<null | (() => void)>(null);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const shaayudIdRef = useRef<string | null>(null);
  const contextRef = useRef<Record<string, any>>({});
  const identityRef = useRef<{ userId?: string; traits?: Record<string, any> }>({});

  useEffect(() => {
    if (initializedRef.current) return;
    initializedRef.current = true;

    let canceled = false;
    (async () => {
      const { stopMouse, shaayud_id, session_id } = await fremenRecord({
        host,
        endpoint,
        eventType: "page_view",
        enableMouse: autoMouse,
      });
      if (canceled) return;
      stopMouseRef.current = stopMouse;
      shaayudIdRef.current = shaayud_id;
      setSessionId(session_id);
    })();

    return () => {
      canceled = true;
      stopMouseRef.current?.();
    };
  }, [host, endpoint, autoMouse]);

  const api: SdkApi = useMemo(() => {
    return {
      track: (type, payload) => {
        if (!sessionId) return;
        const event = {
          shaayud_id: shaayudIdRef.current,
          session_id: sessionId,
          event_id: `evt:${shaayudIdRef.current}:${sessionId}:${Date.now()}`,
          event_type: type,
          ...payload,
          context: contextRef.current,
          identity: identityRef.current,
          front_path: typeof location !== "undefined" ? location.pathname + location.search : "",
          front_referrer: typeof document !== "undefined" ? document.referrer : "",
          front_url: typeof location !== "undefined" ? location.href : "",
        };
        void sendEvent(host, endpoint, sessionId, event);
      },

      identify: (user) => {
        identityRef.current = { ...identityRef.current, ...user };
        if (!sessionId) return;
        void sendEvent(host, endpoint, sessionId, {
          shaayud_id: shaayudIdRef.current,
          session_id: sessionId,
          event_id: `evt:${shaayudIdRef.current}:${sessionId}:${Date.now()}:identify`,
          event_type: "identify",
          userId: identityRef.current.userId,
          traits: identityRef.current.traits,
        });
      },

      setContext: (ctx) => {
        contextRef.current = { ...contextRef.current, ...ctx };
      },

      flush: async () => {},

      getSessionId: () => sessionId,
    };
  }, [sessionId, host, endpoint]);

  // ✅ Use o nome do contexto correto aqui
  return <SdkContext.Provider value={api}>{children}</SdkContext.Provider>;
}

// Hook
export function useSdk(): SdkApi {
  const v = useContext(SdkContext);
  if (!v) throw new Error("useSdk must be used within <SdkProvider/>");
  return v;
}
