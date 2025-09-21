const KEY_ID = "shaayud_id";
const KEY_SESS = "shaayud_session_id";

export const nowSec = () => Math.floor(Date.now() / 1000);

export function ensureShaayudId(): string {
  let v = localStorage.getItem(KEY_ID);
  if (!v) { v = crypto.randomUUID(); localStorage.setItem(KEY_ID, v); }
  return v;
}

export function ensureSessionId(deviceId: string): string {
  let v = sessionStorage.getItem(KEY_SESS);
  if (!v) { v = `sess:${deviceId}:${nowSec()}`; sessionStorage.setItem(KEY_SESS, v); }
  return v;
}