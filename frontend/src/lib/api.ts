const BASE_URL = import.meta.env.PUBLIC_API_URL ?? 'http://localhost:3000';

// ─── Types ────────────────────────────────────────────────────────────────────

export interface ScanResult {
  action: 'IN' | 'OUT';
  student_id: string;
}

export interface ScanRecord {
  id: number;
  student_id: string;
  action: 'IN' | 'OUT';
  timestamp: string;
}

export interface InsideRecord {
  student_id: string;
  timestamp: string;
}

export interface DailyCount {
  day: string;
  count: number;
}

export interface HourCount {
  hour: string;
  count: number;
}

export interface WeekdayAvg {
  weekday: string;
  avg_visitors: number;
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

function authHeaders(): HeadersInit {
  const token = getToken();
  return token ? { Authorization: `Bearer ${token}` } : {};
}

async function handleResponse<T>(res: Response): Promise<T> {
  let data: unknown;
  try {
    data = await res.json();
  } catch {
    throw new Error(`HTTP ${res.status}`);
  }
  if (!res.ok) {
    if (res.status === 401) throw new Error('AUTH_REQUIRED');
    const msg = (data as { error?: string }).error ?? `HTTP ${res.status}`;
    throw new Error(msg);
  }
  return data as T;
}

// ─── Token helpers ────────────────────────────────────────────────────────────

export function getToken(): string | null {
  if (typeof localStorage === 'undefined') return null;
  return localStorage.getItem('jwt');
}

export function clearToken(): void {
  if (typeof localStorage !== 'undefined') localStorage.removeItem('jwt');
}

// ─── Public endpoints ─────────────────────────────────────────────────────────

export async function scanStudent(studentId: string): Promise<ScanResult> {
  const res = await fetch(`${BASE_URL}/scan`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ student_id: studentId })
  });
  return handleResponse<ScanResult>(res);
}

export async function login(
  username: string,
  password: string
): Promise<{ token: string }> {
  const res = await fetch(`${BASE_URL}/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password })
  });
  return handleResponse<{ token: string }>(res);
}

// ─── Protected endpoints ──────────────────────────────────────────────────────

export async function getInside(): Promise<InsideRecord[]> {
  const res = await fetch(`${BASE_URL}/inside`, { headers: authHeaders() });
  return handleResponse<InsideRecord[]>(res);
}

export async function getLog(): Promise<ScanRecord[]> {
  const res = await fetch(`${BASE_URL}/log`, { headers: authHeaders() });
  return handleResponse<ScanRecord[]>(res);
}

export async function getAnalyticsDaily(): Promise<DailyCount[]> {
  const res = await fetch(`${BASE_URL}/analytics/daily`, {
    headers: authHeaders()
  });
  return handleResponse<DailyCount[]>(res);
}

export async function getAnalyticsPeak(): Promise<HourCount[]> {
  const res = await fetch(`${BASE_URL}/analytics/peak`, {
    headers: authHeaders()
  });
  return handleResponse<HourCount[]>(res);
}

export async function getAnalyticsWeekday(): Promise<WeekdayAvg[]> {
  const res = await fetch(`${BASE_URL}/analytics/weekday`, {
    headers: authHeaders()
  });
  return handleResponse<WeekdayAvg[]>(res);
}

/** Fetch all scans and trigger a CSV download in the browser. */
export async function downloadCsv(): Promise<void> {
  const res = await fetch(`${BASE_URL}/export/csv`, { headers: authHeaders() });
  if (!res.ok) throw new Error('Failed to export CSV');
  const blob = await res.blob();
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'library_scans.csv';
  a.click();
  URL.revokeObjectURL(url);
}
