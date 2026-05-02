<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import Chart from 'chart.js/auto';
  import {
    getToken,
    clearToken,
    getInside,
    getLog,
    getAnalyticsDaily,
    getAnalyticsPeak,
    getAnalyticsWeekday,
    downloadCsv,
    type InsideRecord,
    type ScanRecord,
    type DailyCount,
    type HourCount,
    type WeekdayAvg
  } from '$lib/api';

  // ── DOM refs ────────────────────────────────────────────────────────
  let dailyCanvas: HTMLCanvasElement;
  let weekdayCanvas: HTMLCanvasElement;
  let peakCanvas: HTMLCanvasElement;

  // ── Chart instances ─────────────────────────────────────────────────
  let dailyChart: Chart | null = null;
  let weekdayChart: Chart | null = null;
  let peakChart: Chart | null = null;

  // ── Data ────────────────────────────────────────────────────────────
  let insideList: InsideRecord[] = [];
  let logList: ScanRecord[] = [];
  let metrics = { totalWeek: 0, avgPerDay: '0.0', peakHour: '—' };
  let loadError = '';
  let csvLoading = false;

  const DAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
  let refreshTimer: ReturnType<typeof setInterval>;

  // ── Lifecycle ───────────────────────────────────────────────────────
  onMount(async () => {
    if (!getToken()) {
      goto('/admin');
      return;
    }
    await loadAll();
    // Refresh "who is inside" every 30 seconds
    refreshTimer = setInterval(async () => {
      try {
        insideList = await getInside();
      } catch {
        // silently ignore refresh errors
      }
    }, 30_000);
  });

  onDestroy(() => {
    clearInterval(refreshTimer);
    dailyChart?.destroy();
    weekdayChart?.destroy();
    peakChart?.destroy();
  });

  // ── Data loading ────────────────────────────────────────────────────
  async function loadAll() {
    loadError = '';
    try {
      const [daily, peak, weekday, inside, log] = await Promise.all([
        getAnalyticsDaily(),
        getAnalyticsPeak(),
        getAnalyticsWeekday(),
        getInside(),
        getLog()
      ]);

      insideList = inside;
      logList = log.slice(0, 20);

      buildMetrics(daily, peak);
      buildDailyChart(daily);
      buildWeekdayChart(weekday);
      buildPeakChart(peak);
    } catch (e) {
      const msg = (e as Error).message;
      if (msg === 'AUTH_REQUIRED') {
        clearToken();
        goto('/admin');
      } else {
        loadError = msg;
      }
    }
  }

  // ── Metrics ─────────────────────────────────────────────────────────
  function buildMetrics(daily: DailyCount[], peak: HourCount[]) {
    const total = daily.reduce((s, d) => s + d.count, 0);
    const avg = total / 7;
    const topHour = peak.reduce(
      (best, h) => (h.count > best.count ? h : best),
      { hour: '', count: -1 }
    );
    metrics = {
      totalWeek: total,
      avgPerDay: avg.toFixed(1),
      peakHour: topHour.count >= 0 ? `${topHour.hour}:00` : '—'
    };
  }

  // ── Charts ───────────────────────────────────────────────────────────

  const CHART_DEFAULTS = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: { legend: { display: false } }
  };

  function buildDailyChart(data: DailyCount[]) {
    dailyChart?.destroy();
    dailyChart = new Chart(dailyCanvas, {
      type: 'line',
      data: {
        labels: data.map((d) => formatDay(d.day)),
        datasets: [
          {
            label: 'Visitors',
            data: data.map((d) => d.count),
            borderColor: '#2563eb',
            backgroundColor: 'rgba(37,99,235,0.08)',
            fill: true,
            tension: 0.35,
            pointRadius: 4,
            pointBackgroundColor: '#2563eb'
          }
        ]
      },
      options: {
        ...CHART_DEFAULTS,
        scales: {
          y: { beginAtZero: true, ticks: { stepSize: 1 } }
        }
      }
    });
  }

  function buildWeekdayChart(data: WeekdayAvg[]) {
    weekdayChart?.destroy();
    weekdayChart = new Chart(weekdayCanvas, {
      type: 'bar',
      data: {
        labels: data.map((d) => DAYS[parseInt(d.weekday)] ?? d.weekday),
        datasets: [
          {
            label: 'Avg Visitors',
            data: data.map((d) => +d.avg_visitors.toFixed(1)),
            backgroundColor: '#7c3aed',
            borderRadius: 6
          }
        ]
      },
      options: {
        ...CHART_DEFAULTS,
        scales: { y: { beginAtZero: true } }
      }
    });
  }

  function buildPeakChart(data: HourCount[]) {
    peakChart?.destroy();
    peakChart = new Chart(peakCanvas, {
      type: 'bar',
      data: {
        labels: data.map((d) => `${d.hour}:00`),
        datasets: [
          {
            label: 'Scans',
            data: data.map((d) => d.count),
            backgroundColor: '#0891b2',
            borderRadius: 4
          }
        ]
      },
      options: {
        ...CHART_DEFAULTS,
        scales: {
          y: { beginAtZero: true, ticks: { stepSize: 1 } }
        }
      }
    });
  }

  // ── Actions ─────────────────────────────────────────────────────────
  function logout() {
    clearToken();
    goto('/admin');
  }

  async function handleExport() {
    csvLoading = true;
    try {
      await downloadCsv();
    } catch (e) {
      loadError = (e as Error).message;
    } finally {
      csvLoading = false;
    }
  }

  // ── Formatters ───────────────────────────────────────────────────────
  function formatDay(iso: string) {
    const [, month, day] = iso.split('-');
    return `${month}/${day}`;
  }

  function formatTs(iso: string) {
    try {
      return new Date(iso).toLocaleString(undefined, {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      });
    } catch {
      return iso;
    }
  }
</script>

<svelte:head>
  <title>Dashboard — Library</title>
</svelte:head>

<!-- ── Header ──────────────────────────────────────────────────────────── -->
<header class="header">
  <div class="header-inner">
    <span class="header-title">📚 Library Dashboard</span>
    <div class="header-actions">
      <button class="btn btn-ghost btn-sm" on:click={loadAll}>Refresh</button>
      <button class="btn btn-danger btn-sm" on:click={logout}>Log out</button>
    </div>
  </div>
</header>

<main class="container" style="padding-top: 1.5rem; padding-bottom: 3rem;">
  {#if loadError}
    <div class="alert-error">{loadError}</div>
  {/if}

  <!-- ── Metric cards ─────────────────────────────────────────────────── -->
  <section class="metrics">
    <div class="metric-card card">
      <div class="metric-value">{metrics.totalWeek}</div>
      <div class="metric-label">Visitors this week</div>
    </div>
    <div class="metric-card card">
      <div class="metric-value">{metrics.avgPerDay}</div>
      <div class="metric-label">Avg per day</div>
    </div>
    <div class="metric-card card">
      <div class="metric-value">{metrics.peakHour}</div>
      <div class="metric-label">Peak hour today</div>
    </div>
    <div class="metric-card card metric-inside">
      <div class="metric-value">{insideList.length}</div>
      <div class="metric-label">Currently inside</div>
    </div>
  </section>

  <!-- ── Charts ──────────────────────────────────────────────────────── -->
  <section class="charts-grid">
    <div class="card chart-card">
      <h2>Visitors per day <span class="chart-sub">(last 7 days)</span></h2>
      <div class="chart-wrap">
        <canvas bind:this={dailyCanvas}></canvas>
      </div>
    </div>

    <div class="card chart-card">
      <h2>Avg visitors by weekday</h2>
      <div class="chart-wrap">
        <canvas bind:this={weekdayCanvas}></canvas>
      </div>
    </div>

    <div class="card chart-card chart-full">
      <h2>Scans by hour <span class="chart-sub">(today)</span></h2>
      <div class="chart-wrap chart-wrap-wide">
        <canvas bind:this={peakCanvas}></canvas>
      </div>
    </div>
  </section>

  <!-- ── Bottom panels ────────────────────────────────────────────────── -->
  <section class="bottom-grid">

    <!-- Currently inside -->
    <div class="card">
      <div class="panel-header">
        <h2>Currently inside</h2>
        <span class="count-pill">{insideList.length}</span>
      </div>
      {#if insideList.length === 0}
        <p class="empty">Nobody is currently inside.</p>
      {:else}
        <ul class="student-list">
          {#each insideList as rec}
            <li>
              <span class="student-id">{rec.student_id}</span>
              <span class="student-time">{formatTs(rec.timestamp)}</span>
            </li>
          {/each}
        </ul>
      {/if}
      <p class="refresh-note">Auto-refreshes every 30 s</p>
    </div>

    <!-- Recent activity -->
    <div class="card">
      <div class="panel-header">
        <h2>Recent activity</h2>
        <button
          class="btn btn-ghost btn-sm"
          on:click={handleExport}
          disabled={csvLoading}
        >
          {csvLoading ? 'Exporting…' : '⬇ Export CSV'}
        </button>
      </div>

      {#if logList.length === 0}
        <p class="empty">No scans yet.</p>
      {:else}
        <div class="log-table-wrap">
          <table class="log-table">
            <thead>
              <tr>
                <th>ID</th>
                <th>Student</th>
                <th>Action</th>
                <th>Time</th>
              </tr>
            </thead>
            <tbody>
              {#each logList as row}
                <tr>
                  <td class="mono">{row.id}</td>
                  <td class="mono">{row.student_id}</td>
                  <td>
                    <span class="badge badge-{row.action.toLowerCase()}">{row.action}</span>
                  </td>
                  <td class="time-cell">{formatTs(row.timestamp)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>

  </section>
</main>

<style>
  /* ── Header ───────────────────────────────────────────────────────── */
  .header {
    background: var(--card);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    z-index: 10;
  }
  .header-inner {
    max-width: 1100px;
    margin: 0 auto;
    padding: 0.85rem 1rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }
  .header-title {
    font-weight: 700;
    font-size: 1.1rem;
    color: var(--text);
  }
  .header-actions { display: flex; gap: 0.5rem; }

  /* ── Error alert ──────────────────────────────────────────────────── */
  .alert-error {
    background: #fee2e2;
    color: #991b1b;
    border: 1px solid #fca5a5;
    border-radius: var(--radius);
    padding: 0.75rem 1rem;
    margin-bottom: 1.25rem;
    font-size: 0.9rem;
  }

  /* ── Metrics ──────────────────────────────────────────────────────── */
  .metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
  }
  .metric-card {
    text-align: center;
    padding: 1.25rem 1rem;
  }
  .metric-value {
    font-size: 2.25rem;
    font-weight: 800;
    color: var(--primary);
    line-height: 1.1;
  }
  .metric-label {
    font-size: 0.8rem;
    color: var(--text-muted);
    margin-top: 0.3rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .metric-inside .metric-value { color: var(--success); }

  /* ── Charts ───────────────────────────────────────────────────────── */
  .charts-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
  }
  .chart-card h2 {
    font-size: 0.95rem;
    font-weight: 600;
    margin-bottom: 1rem;
  }
  .chart-sub {
    font-weight: 400;
    color: var(--text-muted);
  }
  .chart-wrap {
    position: relative;
    height: 220px;
  }
  .chart-wrap-wide {
    height: 200px;
  }
  .chart-full {
    grid-column: 1 / -1;
  }

  /* ── Bottom panels ────────────────────────────────────────────────── */
  .bottom-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1rem;
    align-items: start;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
    gap: 0.5rem;
  }
  .panel-header h2 {
    font-size: 0.95rem;
    font-weight: 600;
  }

  .count-pill {
    background: var(--primary);
    color: #fff;
    font-size: 0.75rem;
    font-weight: 700;
    padding: 0.15rem 0.55rem;
    border-radius: 999px;
  }

  /* ── Currently inside list ────────────────────────────────────────── */
  .student-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-height: 320px;
    overflow-y: auto;
  }
  .student-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0.75rem;
    background: #f8fafc;
    border-radius: 6px;
    font-size: 0.875rem;
  }
  .student-id { font-weight: 600; font-family: monospace; }
  .student-time { color: var(--text-muted); font-size: 0.8rem; }

  .refresh-note {
    margin-top: 0.75rem;
    font-size: 0.75rem;
    color: var(--text-muted);
    text-align: right;
  }

  .empty {
    color: var(--text-muted);
    font-size: 0.9rem;
    padding: 0.5rem 0;
  }

  /* ── Log table ────────────────────────────────────────────────────── */
  .log-table-wrap {
    overflow-x: auto;
    max-height: 360px;
    overflow-y: auto;
  }
  .log-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
  }
  .log-table th {
    text-align: left;
    padding: 0.4rem 0.6rem;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    border-bottom: 2px solid var(--border);
    position: sticky;
    top: 0;
    background: var(--card);
  }
  .log-table td {
    padding: 0.45rem 0.6rem;
    border-bottom: 1px solid var(--border);
    vertical-align: middle;
  }
  .log-table tr:last-child td { border-bottom: none; }
  .mono { font-family: monospace; }
  .time-cell { color: var(--text-muted); white-space: nowrap; }
</style>
