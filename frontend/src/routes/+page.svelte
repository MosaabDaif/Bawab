<script lang="ts">
  import { scanStudent } from '$lib/api';

  let studentId = '';
  let loading = false;
  let toast: { message: string; kind: 'in' | 'out' | 'error' } | null = null;
  let toastTimer: ReturnType<typeof setTimeout>;

  function showToast(message: string, kind: 'in' | 'out' | 'error') {
    clearTimeout(toastTimer);
    toast = { message, kind };
    toastTimer = setTimeout(() => (toast = null), 3500);
  }

  async function handleScan() {
    const id = studentId.trim();
    if (!id || loading) return;

    loading = true;
    try {
      const result = await scanStudent(id);
      if (result.action === 'IN') {
        showToast(`Checked in ✓  (ID: ${result.student_id})`, 'in');
      } else {
        showToast(`Checked out   (ID: ${result.student_id})`, 'out');
      }
      studentId = '';
    } catch (e) {
      showToast((e as Error).message, 'error');
    } finally {
      loading = false;
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleScan();
  }
</script>

<svelte:head>
  <title>Library Check-In</title>
</svelte:head>

<div class="page">
  <div class="checkin-card card">
    <div class="logo">📚</div>
    <h1>Library Check-In</h1>
    <p class="subtitle">Scan or type your student ID</p>

    <div class="input-row">
      <!-- inputmode="numeric" shows the numeric keyboard on mobile -->
      <input
        type="tel"
        inputmode="numeric"
        pattern="[0-9]*"
        placeholder="Student ID"
        autocomplete="off"
        bind:value={studentId}
        on:keydown={onKeydown}
        disabled={loading}
        autofocus
      />
    </div>

    <button
      class="btn btn-primary btn-block scan-btn"
      on:click={handleScan}
      disabled={loading || !studentId.trim()}
    >
      {loading ? 'Processing…' : 'Check In / Out'}
    </button>

    <p class="hint">Enter 4–12 digit student ID</p>
  </div>

  <a class="admin-link" href="/admin">Admin →</a>
</div>

{#if toast}
  <div class="toast toast-{toast.kind}" role="status">
    {toast.message}
  </div>
{/if}

<style>
  .page {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100dvh;
    padding: 1rem;
    gap: 1rem;
  }

  .checkin-card {
    width: 100%;
    max-width: 420px;
    text-align: center;
    padding: 2.5rem 2rem;
  }

  .logo {
    font-size: 3.5rem;
    margin-bottom: 0.5rem;
  }

  h1 {
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--text);
    margin-bottom: 0.35rem;
  }

  .subtitle {
    color: var(--text-muted);
    margin-bottom: 1.75rem;
    font-size: 0.95rem;
  }

  .input-row {
    margin-bottom: 1rem;
  }

  .input-row input {
    font-size: 1.6rem;
    text-align: center;
    letter-spacing: 0.1em;
    padding: 1rem;
    border-radius: 12px;
  }

  .scan-btn {
    font-size: 1.1rem;
    padding: 1rem;
    border-radius: 12px;
    min-height: 56px;
  }

  .hint {
    margin-top: 0.75rem;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .admin-link {
    font-size: 0.85rem;
    color: var(--text-muted);
    text-decoration: none;
    opacity: 0.7;
  }
  .admin-link:hover { opacity: 1; }

  /* ── Toast ─────────────────────────────────────────────────────── */
  .toast {
    position: fixed;
    bottom: 2rem;
    left: 50%;
    transform: translateX(-50%);
    padding: 1rem 2rem;
    border-radius: 12px;
    font-size: 1.05rem;
    font-weight: 600;
    box-shadow: var(--shadow-lg);
    z-index: 100;
    white-space: nowrap;
    animation: slide-up 0.2s ease;
  }

  .toast-in    { background: #dcfce7; color: #15803d; }
  .toast-out   { background: #fee2e2; color: #b91c1c; }
  .toast-error { background: #fef3c7; color: #92400e; }

  @keyframes slide-up {
    from { opacity: 0; transform: translateX(-50%) translateY(12px); }
    to   { opacity: 1; transform: translateX(-50%) translateY(0); }
  }
</style>
