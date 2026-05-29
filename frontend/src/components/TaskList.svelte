<script>
  import { getTasks, createTask, updateTask, deleteTask } from '../lib/api.js';

  let { showToast } = $props();

  let tasks = $state([]);
  let loading = $state(true);
  let showForm = $state(false);
  let editingTask = $state(null);
  let filter = $state('all');

  // Form state
  let title = $state('');
  let description = $state('');
  let priority = $state('medium');

  // Load tasks on mount
  $effect(() => {
    loadTasks();
  });

  async function loadTasks() {
    loading = true;
    try {
      const data = await getTasks(filter !== 'all' ? { status: filter } : {});
      tasks = data.tasks || data || [];
    } catch (err) {
      showToast('Gagal memuat tasks: ' + err.message, 'error');
    } finally {
      loading = false;
    }
  }

  async function handleCreateTask(e) {
    e.preventDefault();
    try {
      await createTask({ title, description, priority });
      showToast('Task berhasil dibuat! ✅', 'success');
      resetForm();
      await loadTasks();
    } catch (err) {
      showToast(err.message, 'error');
    }
  }

  async function handleUpdateStatus(task, newStatus) {
    try {
      await updateTask(task.id, { status: newStatus });
      showToast(`Status diubah ke ${newStatus}`, 'success');
      await loadTasks();
    } catch (err) {
      showToast(err.message, 'error');
    }
  }

  async function handleDelete(task) {
    if (!confirm(`Hapus task "${task.title}"?`)) return;
    try {
      await deleteTask(task.id);
      showToast('Task dihapus', 'success');
      await loadTasks();
    } catch (err) {
      showToast(err.message, 'error');
    }
  }

  function resetForm() {
    title = '';
    description = '';
    priority = 'medium';
    showForm = false;
    editingTask = null;
  }

  function getStatusClass(status) {
    return status?.replace('_', '-') || 'pending';
  }

  function getPriorityClass(priority) {
    return priority || 'medium';
  }

  function formatDate(dateStr) {
    if (!dateStr) return '';
    return new Date(dateStr).toLocaleDateString('id-ID', {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  // Re-fetch when filter changes
  $effect(() => {
    filter;
    loadTasks();
  });
</script>

<div class="tasks-page animate-fade-in">
  <!-- Header Bar -->
  <div class="tasks-header">
    <div class="tasks-title-section">
      <h2>📋 Tasks</h2>
      <span class="task-count">{tasks.length} task{tasks.length !== 1 ? 's' : ''}</span>
    </div>
    <div class="tasks-actions">
      <!-- Filter -->
      <select class="input" bind:value={filter}>
        <option value="all">Semua</option>
        <option value="pending">Pending</option>
        <option value="in_progress">In Progress</option>
        <option value="completed">Completed</option>
      </select>
      <!-- New Task Button -->
      <button class="btn btn-primary" onclick={() => (showForm = !showForm)}>
        {showForm ? '✕ Tutup' : '+ Task Baru'}
      </button>
    </div>
  </div>

  <!-- Create Task Form -->
  {#if showForm}
    <div class="card create-form animate-fade-in">
      <h3>✨ Task Baru</h3>
      <form onsubmit={handleCreateTask}>
        <div class="form-grid">
          <div class="input-group">
            <label for="task-title">Judul</label>
            <input
              id="task-title"
              class="input"
              type="text"
              placeholder="Belajar Kubernetes..."
              bind:value={title}
              required
            />
          </div>
          <div class="input-group">
            <label for="task-priority">Prioritas</label>
            <select id="task-priority" class="input" bind:value={priority}>
              <option value="low">🟢 Low</option>
              <option value="medium">🟡 Medium</option>
              <option value="high">🔴 High</option>
            </select>
          </div>
        </div>
        <div class="input-group" style="margin-top: 16px;">
          <label for="task-desc">Deskripsi</label>
          <textarea
            id="task-desc"
            class="input"
            placeholder="Deskripsi task..."
            bind:value={description}
          ></textarea>
        </div>
        <div class="form-actions">
          <button class="btn btn-secondary" type="button" onclick={resetForm}>Batal</button>
          <button class="btn btn-primary" type="submit">💾 Simpan</button>
        </div>
      </form>
    </div>
  {/if}

  <!-- Task List -->
  {#if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Memuat tasks...</p>
    </div>
  {:else if tasks.length === 0}
    <div class="empty-state card">
      <div class="empty-icon">📭</div>
      <h3>Belum ada task</h3>
      <p>Klik "Task Baru" untuk membuat task pertama.</p>
    </div>
  {:else}
    <div class="task-list">
      {#each tasks as task, i}
        <div class="task-card card" style="animation-delay: {i * 50}ms;">
          <div class="task-card-header">
            <div class="task-meta">
              <span class="badge badge-{getStatusClass(task.status)}">{task.status?.replace('_', ' ') || 'pending'}</span>
              <span class="badge badge-{getPriorityClass(task.priority)}">{task.priority || 'medium'}</span>
            </div>
            <div class="task-card-actions">
              {#if task.status !== 'completed'}
                <button
                  class="btn btn-sm btn-secondary"
                  onclick={() => handleUpdateStatus(task, task.status === 'pending' ? 'in_progress' : 'completed')}
                >
                  {task.status === 'pending' ? '▶ Start' : '✓ Done'}
                </button>
              {/if}
              <button class="btn btn-sm btn-danger" onclick={() => handleDelete(task)}>🗑</button>
            </div>
          </div>
          <h3 class="task-title">{task.title}</h3>
          {#if task.description}
            <p class="task-description">{task.description}</p>
          {/if}
          <div class="task-footer">
            <span class="task-date">{formatDate(task.created_at)}</span>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .tasks-page {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .tasks-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 16px;
  }

  .tasks-title-section {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .tasks-title-section h2 {
    font-size: 1.5rem;
    font-weight: 700;
  }

  .task-count {
    font-size: 0.8rem;
    color: var(--text-muted);
    padding: 3px 10px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-full);
  }

  .tasks-actions {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  /* Create Form */
  .create-form {
    padding: 28px;
  }

  .create-form h3 {
    font-size: 1.1rem;
    margin-bottom: 20px;
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 16px;
  }

  .form-actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    margin-top: 20px;
  }

  /* Task Cards */
  .task-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .task-card {
    padding: 20px;
    animation: fadeIn 0.4s ease forwards;
    opacity: 0;
  }

  .task-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .task-meta {
    display: flex;
    gap: 8px;
  }

  .task-card-actions {
    display: flex;
    gap: 6px;
  }

  .task-title {
    font-size: 1.1rem;
    font-weight: 600;
    margin-bottom: 6px;
  }

  .task-description {
    color: var(--text-secondary);
    font-size: 0.9rem;
    margin-bottom: 12px;
  }

  .task-footer {
    display: flex;
    justify-content: flex-end;
  }

  .task-date {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  /* Empty & Loading States */
  .loading-state,
  .empty-state {
    text-align: center;
    padding: 60px 24px;
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    color: var(--text-secondary);
  }

  .empty-icon {
    font-size: 3rem;
    margin-bottom: 12px;
  }

  .empty-state h3 {
    font-size: 1.2rem;
    margin-bottom: 8px;
  }

  .empty-state p {
    color: var(--text-secondary);
  }

  @media (max-width: 640px) {
    .tasks-header {
      flex-direction: column;
      align-items: stretch;
    }

    .tasks-actions {
      flex-direction: column;
    }

    .form-grid {
      grid-template-columns: 1fr;
    }

    .task-card-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }
  }
</style>
