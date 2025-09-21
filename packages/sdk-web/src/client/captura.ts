import FingerprintJS from "@fingerprintjs/fingerprintjs";
import { ensureShaayudId, ensureSessionId, nowSec } from "./ids";
import { startMouseCapture, StopFn } from "../capture/mouse";

export type CapturaOptions = {
  host: string;
  endpoint?: string;
  eventType?: string; // "page_view"
  enableMouse?: boolean | { maxMs?: number; targetFps?: number; sendIntervalMs?: number };
};

export async function fremenRecord(opts: CapturaOptions): Promise<{ stopMouse: StopFn | null; shaayud_id: string; session_id: string; }> {
  const { host, endpoint = "/identity/injest", eventType = "page_view", enableMouse = false } = opts;

  const fp = await FingerprintJS.load();
  const result = await fp.get();
  const deviceId = (result as any).visitorId || "unknown-device";

  const shaayud_id = ensureShaayudId();
  const session_id = ensureSessionId(deviceId);
  const tsSec = nowSec();
  const event_id = `evt:${shaayud_id}:${session_id}:${tsSec}`;

  const front_url  = typeof location !== "undefined" ? location.href : "";
  const front_path = typeof location !== "undefined" ? (location.pathname + location.search) : "";
  const front_referrer = typeof document !== "undefined" ? document.referrer : "";

  const url = new URL(endpoint, host).toString();
  await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json", "x-session-id": session_id },
    body: JSON.stringify({
      shaayud_id,
      fingerprint: result,
      session_id,
      event_id,
      event_type: eventType,
      front_url,
      front_path,
      front_referrer
    }),
    keepalive: true
  });

  let stopMouse: StopFn | null = null;
  if (enableMouse) {
    const cfg = typeof enableMouse === "object" ? enableMouse : {};
    stopMouse = startMouseCapture({
      url, shaayud_id, session_id,
      maxMs: cfg.maxMs ?? 8000,
      targetFps: cfg.targetFps ?? 30,
      sendIntervalMs: cfg.sendIntervalMs ?? 0,
    });
  }

  return { stopMouse, shaayud_id, session_id };
}
