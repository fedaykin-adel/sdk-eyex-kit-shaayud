import * as pako from "pako";
import { nowSec } from "../client/ids";

export type StopFn = () => void;

export function startMouseCapture(cfg: {
  url: string;
  shaayud_id: string;
  session_id: string;
  maxMs?: number;
  targetFps?: number;
  sendIntervalMs?: number;
}): StopFn {
  const { url, shaayud_id, session_id, maxMs = 8000, targetFps = 30, sendIntervalMs = 0 } = cfg;

  const pts: number[] = [];
  const clicks: Array<{x:number;y:number;t:number;b:number}> = [];
  const wheel = { ticks: 0, dy_sum: 0 };

  let start = performance.now();
  let lastFrame = 0;
  const frameInterval = 1000 / targetFps;

  function onMove(e: PointerEvent) {
    const now = performance.now();
    if (now - lastFrame < frameInterval) return;
    lastFrame = now;
    const w = Math.max(1, window.innerWidth), h = Math.max(1, window.innerHeight);
    const x = Math.max(0, Math.min(1, e.clientX / w));
    const y = Math.max(0, Math.min(1, e.clientY / h));
    const t = Math.min(maxMs, now - start);
    pts.push(x, y, t);
    if (t >= maxMs) flush("time");
  }
  function onClick(e: PointerEvent) { clicks.push({ x: e.clientX, y: e.clientY, t: Date.now(), b: e.button }); }
  function onWheel(e: WheelEvent) { wheel.ticks += 1; wheel.dy_sum += e.deltaY; }

  function flush(_why: "time"|"vis"|"manual") {
    if (!pts.length) return;
    const ts_end = Date.now();
    const f32 = new Float32Array(pts);
    const def = pako.deflate(new Uint8Array(f32.buffer));
    const b64 = btoa(String.fromCharCode(...def));
    const front_path = typeof location !== "undefined" ? location.pathname + location.search : "";
    const event_id = `evt:${shaayud_id}:${session_id}:${nowSec()}:mouse`;

    const payload = {
      shaayud_id, session_id, event_id,
      event_type: "mouse_batch" as const,
      front_path,
      ts_start: ts_end - Math.min(maxMs, Math.floor(pts[pts.length - 1])),
      ts_end,
      viewport: { w: window.innerWidth || 1, h: window.innerHeight || 1 },
      points_deflate_b64: b64,
      clicks: clicks.length ? clicks.slice(0, 50) : undefined,
      wheel: wheel.ticks ? { ...wheel } : undefined
    };
    console.log(payload)

    const blob = new Blob([JSON.stringify(payload)], { type: "application/json" });
    // let ok = false;
    // if (typeof navigator !== "undefined" && "sendBeacon" in navigator) {
    //     ok = (navigator as any).sendBeacon(url, blob);
    //     console.log(ok)
    // }
    // console.log(ok)
    // if (!ok) {
      fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'x-session-id': session_id },
        body: JSON.stringify(payload),
        credentials: 'omit',        
        keepalive: true
        });
    // }

    pts.length = 0; clicks.length = 0; wheel.ticks = 0; wheel.dy_sum = 0;
    start = performance.now(); lastFrame = 0;
  }

  function onVis() { if (document.visibilityState === "hidden") flush("vis"); }
  function onHide() { flush("vis"); }

  window.addEventListener("pointermove", onMove, { passive: true });
  window.addEventListener("pointerdown", onClick, { passive: true });
  window.addEventListener("wheel", onWheel, { passive: true });
  document.addEventListener("visibilitychange", onVis);
  window.addEventListener("pagehide", onHide);

  let intervalId: any = 0;
  if (sendIntervalMs > 0) intervalId = setInterval(() => flush("manual"), sendIntervalMs);

  return () => {
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerdown", onClick);
    window.removeEventListener("wheel", onWheel);
    document.removeEventListener("visibilitychange", onVis);
    window.removeEventListener("pagehide", onHide);
    if (intervalId) clearInterval(intervalId);
    flush("manual");
  };
}
