// adapta Node/Express e Hono para um único fluxo de ingest
// import { ingest } from "@shaayud/sdk-core";
function loadCore(): any {
  try {
    // evita que o bundler (SWC/Webpack) resolva no build
    // @ts-ignore
    const rq = eval('require');
    return rq('@shaayud/sdk-core');
  } catch {
    return null;
  }
}
import geoip from "geoip-lite";

/** Detecta se o objeto é um Hono Context */
function isHono(x: any): boolean {
  return !!(x?.req && typeof x.req?.header === "function" && typeof x.req?.url === "string");
}

/** Lê um header de forma unificada (Node/Hono) */
function getHeader(reqLike: any, name: string): string | undefined {
  if (isHono(reqLike)) return reqLike.req.header(name) ?? undefined;
  const h = reqLike?.headers || {};
  const v = h[name] ?? h[name?.toLowerCase?.()];
  return Array.isArray(v) ? v[0] : v;
}

/** Retorna método HTTP (Node/Hono) */
function getMethod(reqLike: any): string {
  if (isHono(reqLike)) return reqLike.req.method || "UNKNOWN";
  return reqLike?.method || "UNKNOWN";
}

/** Retorna URL/caminho (Node/Hono) */
function getUrl(reqLike: any): string {
  if (isHono(reqLike)) return reqLike.req.url || "";
  return reqLike?.originalUrl || reqLike?.url || "";
}

/** Retorna corpo JSON (Node/Hono) */
async function getJsonBody(reqLike: any): Promise<any> {
  if (isHono(reqLike)) return await reqLike.req.json().catch(() => ({}));
  return typeof reqLike?.body === "object" && reqLike.body ? reqLike.body : {};
}

/** Normaliza IP para formato padrão */
function normalizeIp(ip: string | undefined): string {
  if (!ip) return "unknown";
  if (ip === "::1") return "127.0.0.1";
  if (ip.startsWith("::ffff:")) return ip.replace("::ffff:", "");
  return ip;
}

/** Converte headers para objeto simples (Node/Hono) */
function toPlainHeaders(reqLike: any): Record<string, string> {
  if (isHono(reqLike)) {
    const out: Record<string, string> = {};
    const headers = reqLike.req.raw.headers as Headers;
    headers.forEach((v: string, k: string) => (out[k.toLowerCase()] = v));
    return out;
  }
  const h = reqLike?.headers;
  if (!h) return {};
  if (typeof h.entries === "function") return Object.fromEntries(Array.from(h.entries()));
  const out: Record<string, string> = {};
  for (const k of Object.keys(h)) {
    const v = (h as any)[k];
    out[k.toLowerCase()] = Array.isArray(v) ? v.join(",") : String(v);
  }
  return out;
}

/** Verifica se IP é privado */
const PRIVATE_BLOCKS: RegExp[] = [
  /^10\./,
  /^127\./,
  /^172\.(1[6-9]|2\d|3[0-1])\./,
  /^192\.168\./,
  /^::1$/,
  /^fc00:/,
  /^fe80:/,
  /^::ffff:127\./
];
function isPrivate(ip: string): boolean {
  return PRIVATE_BLOCKS.some((r) => r.test(ip));
}

/** Faz parse do X-Forwarded-For */
function parseXff(xff: string): string[] {
  return xff.split(",").map((s) => s.trim()).filter(Boolean);
}

/** Escolhe o primeiro IP público válido */
function pickPublicIp(ips: string[]): string | undefined {
  return ips.find((ip) => !isPrivate(ip));
}

/** Obtém IP público do cliente (Node/Hono) */
function getPublicClientIp(reqLike: any): string {
  const headers = toPlainHeaders(reqLike);
  const list: string[] = [];
  const xff = headers["x-forwarded-for"];
  if (xff) list.push(...parseXff(xff));
  const realIp = headers["x-real-ip"];
  if (realIp) list.push(realIp);
  if (!isHono(reqLike)) {
    if (reqLike.connection?.remoteAddress) list.push(reqLike.connection.remoteAddress);
    if (reqLike.socket?.remoteAddress) list.push(reqLike.socket.remoteAddress);
    if (reqLike.ip) list.push(reqLike.ip);
  }
  const normalized = list.map(normalizeIp).filter(Boolean);
  return pickPublicIp(normalized) || normalized[0] || "unknown";
}

/** Faz lookup geo por IP */
function geoLookup(ip: string) {
  const hit = geoip.lookup(ip);
  if (!hit) return null;
  const [lat, lng] = hit.ll || [null, null];
  return {
    country: hit.country || null,
    region: hit.region || null,
    city: hit.city || null,
    latitude: lat,
    longitude: lng,
    timezone: hit.timezone || null
  };
}

/** Extrai/gera sessionId a partir de header/body */
function getSessionId(headers: Record<string, string>, body: any, deviceId: string, tsSec: number): string {
  const fromHeader = headers["x-session-id"] || headers["sessionid"] || headers["session-id"];
  const fromBody = body?.session_id || body?.sessionId;
  return (fromHeader as string) || fromBody || `sess:${deviceId}:${tsSec}`;
}

/** Gera eventId a partir de identidade/sessão */
function getEventId(identityId: string, sessionId: string, tsSec: number): string {
  return `evt:${identityId}:${sessionId}:${tsSec}`;
}

/** Extrai tipo do evento */
function getEventType(body: any, method: string, path: string): string {
  const fromBody = body?.event_type || body?.eventType;
  if (fromBody) return fromBody;
  return `${method} ${path}`;
}

/** Extrai fingerprint do body */
function getFingerprint(body: any): any {
  const fp = body?.fingerprint;
  if (!fp) return {};
  if (typeof fp === "string") {
    try {
      return JSON.parse(fp);
    } catch {
      return { raw: fp };
    }
  }
  return fp;
}

/** Extrai deviceId do fingerprint */
function getDeviceIdFromFingerprint(fp: any): string | undefined {
  return fp?.id || fp?.visitorId || fp?.deviceId || fp?.device_id;
}

/** Extrai identidade do body */
function getIdentityId(body: any): string {
  return body?.shaayud_id || body?.shaayudId || "";
}

/** Extrai user-agent */
function getUserAgent(reqLike: any): string {
  return getHeader(reqLike, "user-agent") || "unknown";
}

/** Meta de backend: armazena/obtém dados da rota no reqLike (Node/Hono) */
const META_KEY = "__backend";
function setBackendMeta(reqLike: any, meta: { route: string; method: string; host: string }) {
  if (isHono(reqLike)) reqLike.set(META_KEY, meta);
  else reqLike[META_KEY] = meta;
}
function getBackendMeta(reqLike: any) {
  if (isHono(reqLike)) {
    const m = reqLike.get(META_KEY) || {};
    return {
      backend_path: m.route || getUrl(reqLike),
      backend_method: m.method || getMethod(reqLike),
      backend_host: m.host || (getHeader(reqLike, "host") || "")
    };
  }
  const m = reqLike[META_KEY] || {};
  return {
    backend_path: m.route || (reqLike.url || reqLike.originalUrl || ""),
    backend_method: m.method || (reqLike.method || "UNKNOWN"),
    backend_host: m.host || (reqLike.headers?.host || "")
  };
}

/** Extrai front meta */
function getFrontMetaFromBody(body: any) {
  return {
    front_url: body?.front_url ?? undefined,
    front_path: body?.front_path ?? undefined,
    front_referrer: body?.front_referrer ?? undefined
  };
}

/** Extrai mouse batch do body */
function getMouseFromBody(body: any) {
  return {
    ts_start: body?.ts_start ?? undefined,
    ts_end: body?.ts_end ?? undefined,
    viewport: body?.viewport ?? undefined,
    points_deflate_b64: body?.points_deflate_b64 ?? undefined,
    clicks: Array.isArray(body?.clicks) ? body.clicks : undefined,
    wheel: body?.wheel ?? undefined
  };
}

/** Ingest unificado (Node/Hono) */
export async function ingestPayload(reqLike: any): Promise<boolean> {
    const core = loadCore();
  const timestamp = new Date();
  const tsIso = timestamp.toISOString();
  const tsSec = Math.floor(timestamp.getTime() / 1000);

  const headersPlain = toPlainHeaders(reqLike);
  const method = getMethod(reqLike);
  const path = getUrl(reqLike);
  const body = await getJsonBody(reqLike);

  const shaayudId = getIdentityId(body);
  const fingerprint = getFingerprint(body);
  const deviceId = getDeviceIdFromFingerprint(fingerprint) || "unknown-device";

  const publicIp = getPublicClientIp(reqLike);
  const geo = geoLookup(publicIp) || undefined;
  const be = getBackendMeta(reqLike);
  const fe = getFrontMetaFromBody(body);
  const userAgent = getUserAgent(reqLike);

  const sessionId = getSessionId(headersPlain, body, deviceId, tsSec);
  const eventId = getEventId(shaayudId, sessionId, tsSec);
  const eventType = getEventType(body, method, path);
  const mouseBatch = getMouseFromBody(body);

  const payload = {
    shaayud_id: shaayudId,
    fingerprint,
    ip: publicIp,
    user_agent: userAgent,
    header: {
      ...headersPlain,
      "x-client-ip": publicIp,
      "x-geo-country": geo?.country ?? "",
      "x-geo-region": geo?.region ?? "",
      "x-geo-city": geo?.city ?? "",
      "x-geo-timezone": geo?.timezone ?? ""
    },
    timestamp: tsIso,
    method,
    path,
    session_id: sessionId,
    event_id: eventId,
    event_type: eventType,
    geo,
    backend_path: be.backend_path,
    backend_method: be.backend_method,
    backend_host: be.backend_host,
    front_url: fe.front_url,
    front_path: fe.front_path,
    front_referrer: fe.front_referrer,
    ...mouseBatch
  };

  const res = await core.ingest(JSON.stringify(payload));
  return typeof res === "string" && res === "ok";
}

/** Middleware Node/Express para carimbar meta de backend */
export function traceBackendRoute(
  skipPaths: (string | RegExp)[] = ["/identity/ingest"]
) {
  const normalize = (p: string) =>
    String(p).toLowerCase().split("?")[0].replace(/\/+$/, "");

  return (req: any, _res: any, next: any) => {
    // só page-view / navegação
    if ((req.method || "GET").toUpperCase() !== "GET") return next();

    const path = normalize(req.originalUrl || req.url || "");
    const shouldSkip = skipPaths.some((sp) =>
      sp instanceof RegExp
        ? sp.test(path)
        : path === normalize(String(sp)) || path.startsWith(normalize(String(sp)) + "/")
    );
    if (shouldSkip) return next();

    setBackendMeta(req, {
      route: path,                      // ex.: /products/camiseta-preta
      method: req.method || "UNKNOWN",  // GET
      host: req.headers?.host || "",
    });

    next();
  };
}

/** Middleware Hono para carimbar meta de backend */
export function traceBackendRouteHono(skipPaths: string[] = ["/identity/ingest"]) {
  return async (c: any, next: any) => {
    const path = getUrl(c);
    if (!skipPaths.some((p) => path.startsWith(p))) {
      setBackendMeta(c, {
        route: path,
        method: getMethod(c),
        host: getHeader(c, "host") || ""
      });
    }
    await next();
  };
}
