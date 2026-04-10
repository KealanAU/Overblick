<script setup>
import { ref, watch, computed } from 'vue'
import { getCommits } from '../services/github'
import { getTimeLogs, getTimeSummary, updateTimeLog, deleteTimeLog } from '../services/time'
import { summarizeActivity } from '../services/ai'
import { useAiSettings } from '../composables/useAiSettings.js'
import { timeAgo } from '../utils/time.js'

const props = defineProps({ repos: { type: Array, required: true } })

const { aiProvider, claudeKey, claudeModel, openaiKey, openaiModel, ollamaUrl, ollamaModel } = useAiSettings()

const summary        = ref(null)   // SummaryResult | null
const summaryError   = ref('')
const summarizing    = ref(false)
const summaryDays    = ref(7)
const summaryVisible = ref(false)

const dayItems = [
  { label: 'Last 7 days',  value: 7  },
  { label: 'Last 14 days', value: 14 },
  { label: 'Last 30 days', value: 30 },
]

function formatMinutesBrief(m) {
  if (!m) return '—'
  const h = Math.floor(m / 60)
  const rem = m % 60
  return h === 0 ? `${m}m` : rem ? `${h}h ${rem}m` : `${h}h`
}

async function summarize() {
  summaryError.value   = ''
  summary.value        = null
  summarizing.value    = true
  summaryVisible.value = true
  try {
    const apiKey  = aiProvider.value === 'claude' ? claudeKey.value
                  : aiProvider.value === 'openai'  ? openaiKey.value
                  : null
    const model   = aiProvider.value === 'claude' ? claudeModel.value
                  : aiProvider.value === 'openai'  ? openaiModel.value
                  : ollamaModel.value
    const baseUrl = aiProvider.value === 'ollama' ? ollamaUrl.value : null

    summary.value = await summarizeActivity(
      aiProvider.value,
      apiKey,
      model,
      baseUrl,
      summaryDays.value,
    )
  } catch (e) {
    summaryError.value = String(e)
  } finally {
    summarizing.value = false
  }
}

const allCommitsWithRepo = ref([])
const repoStats = ref([])
const timeLogs = ref([])
const timeSummary = ref({ today_minutes: 0, week_minutes: 0 })
const loading = ref(false)

const editingId = ref(null)
const editForm = ref({ duration_minutes: 0, comment: '', repo_id: null })

const repoSelectItems = computed(() => [
  { label: '— no repo —', value: null },
  ...props.repos.map(r => ({ label: r.name, value: r.id })),
])

function formatMinutes(m) {
  const h = Math.floor(m / 60)
  const rem = m % 60
  return h === 0 ? `${m}m` : rem ? `${h}h ${rem}m` : `${h}h`
}

async function loadAll() {
  loading.value = true
  try {
    const [logs, summary, ...commitResults] = await Promise.all([
      getTimeLogs(50),
      getTimeSummary(),
      ...props.repos.map(repo =>
        getCommits(repo.id, 500)
          .then(commits => commits.map(c => ({ ...c, repoName: repo.name })))
          .catch(() => [])
      )
    ])
    timeLogs.value = logs
    timeSummary.value = summary
    if (!props.repos.length) return
    const flat = commitResults.flat().sort((a, b) => b.committed_at - a.committed_at)
    allCommitsWithRepo.value = flat.slice(0, 30)
    repoStats.value = props.repos.map((repo, i) => ({
      repo,
      totalCommits: commitResults[i].length,
      latestCommit: commitResults[i][0] || null
    })).sort((a, b) => {
      const aTime = a.latestCommit?.committed_at || 0
      const bTime = b.latestCommit?.committed_at || 0
      return bTime - aTime
    })
  } finally {
    loading.value = false
  }
}

watch(() => props.repos, loadAll, { immediate: true, deep: true })


function startEdit(log) {
  editingId.value = log.id
  editForm.value = { duration_minutes: log.duration_minutes, comment: log.comment, repo_id: log.repo_id ?? null, logged_at: log.logged_at }
}

function cancelEdit() { editingId.value = null }

async function saveEdit(id) {
  await updateTimeLog(
    id,
    editForm.value.duration_minutes,
    editForm.value.comment,
    editForm.value.repo_id,
    editForm.value.logged_at,
  )
  editingId.value = null
  await loadAll()
}

async function deleteLog(id) {
  await deleteTimeLog(id)
  if (editingId.value === id) editingId.value = null
  await loadAll()
}
</script>

<template>
  <div class="flex flex-col h-full overflow-y-auto p-4 gap-6">

    <!-- Header -->
    <div class="flex items-start justify-between gap-3">
      <div>
        <h2 class="text-base font-semibold">Overview</h2>
        <p class="text-xs text-muted mt-0.5">Activity across all repositories</p>
      </div>
      <div class="flex items-center gap-2 shrink-0">
        <USelect v-model="summaryDays" :items="dayItems" size="xs" class="w-32" />
        <UButton
          icon="i-lucide-sparkles"
          size="xs"
          variant="soft"
          :loading="summarizing"
          @click="summarize"
        >
          Summarise
        </UButton>
      </div>
    </div>

    <!-- AI summary -->
    <div
      v-if="summaryVisible"
      class="rounded-lg border border-default bg-elevated overflow-hidden"
    >
      <!-- Card header -->
      <div class="flex items-center justify-between px-4 py-2.5 border-b border-default">
        <div class="flex items-center gap-1.5 text-xs font-semibold text-muted uppercase tracking-wider">
          <UIcon name="i-lucide-sparkles" class="size-3.5" />
          AI Summary
        </div>
        <UButton
          icon="i-lucide-x"
          variant="ghost"
          size="xs"
          color="neutral"
          :padded="false"
          @click="summaryVisible = false"
        />
      </div>

      <!-- Loading -->
      <div v-if="summarizing" class="flex items-center gap-2 px-4 py-4">
        <UIcon name="i-lucide-loader-circle" class="size-4 text-muted animate-spin" />
        <span class="text-xs text-muted">Thinking…</span>
      </div>

      <!-- Error -->
      <p v-else-if="summaryError" class="px-4 py-3 text-xs text-red-500">{{ summaryError }}</p>

      <!-- Result -->
      <template v-else-if="summary">
        <!-- Overall sentence -->
        <p v-if="summary.overall" class="px-4 py-3 text-sm text-muted border-b border-default">
          {{ summary.overall }}
        </p>

        <!-- Empty -->
        <p v-if="!summary.rows.length" class="px-4 py-3 text-sm text-muted">
          No activity found for this period.
        </p>

        <!-- Table -->
        <table v-else class="w-full text-sm">
          <thead>
            <tr class="border-b border-default text-left">
              <th class="px-4 py-2 text-xs font-semibold text-muted uppercase tracking-wider">Repository</th>
              <th class="px-4 py-2 text-xs font-semibold text-muted uppercase tracking-wider text-right">Commits</th>
              <th class="px-4 py-2 text-xs font-semibold text-muted uppercase tracking-wider text-right">Time</th>
              <th class="px-4 py-2 text-xs font-semibold text-muted uppercase tracking-wider">Highlights</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="row in summary.rows"
              :key="row.repo"
              class="border-b border-default last:border-0 hover:bg-accented/40 transition-colors"
            >
              <td class="px-4 py-2.5 font-medium truncate max-w-[140px]">{{ row.repo }}</td>
              <td class="px-4 py-2.5 text-right tabular-nums">
                <UBadge :label="String(row.commits)" variant="outline" size="sm" />
              </td>
              <td class="px-4 py-2.5 text-right tabular-nums text-muted">
                {{ formatMinutesBrief(row.time_minutes) }}
              </td>
              <td class="px-4 py-2.5 text-muted text-xs">{{ row.highlight || '—' }}</td>
            </tr>
          </tbody>
        </table>
      </template>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex justify-center py-8">
      <UIcon name="i-lucide-loader-circle" class="size-5 text-muted animate-spin" />
    </div>

    <template v-else>

      <!-- Summary stats row -->
      <div class="grid grid-cols-2 gap-3">
        <div class="rounded-lg border border-default bg-elevated px-4 py-3">
          <p class="text-2xl font-bold">{{ repos.length }}</p>
          <p class="text-xs text-muted mt-0.5">Repositories</p>
        </div>
        <div class="rounded-lg border border-default bg-elevated px-4 py-3">
          <p class="text-2xl font-bold">{{ repoStats.reduce((sum, s) => sum + s.totalCommits, 0) }}</p>
          <p class="text-xs text-muted mt-0.5">Total Commits</p>
        </div>
        <div class="rounded-lg border border-default bg-elevated px-4 py-3">
          <p class="text-2xl font-bold">{{ formatMinutes(timeSummary.today_minutes) }}</p>
          <p class="text-xs text-muted mt-0.5">Today</p>
        </div>
        <div class="rounded-lg border border-default bg-elevated px-4 py-3">
          <p class="text-2xl font-bold">{{ formatMinutes(timeSummary.week_minutes) }}</p>
          <p class="text-xs text-muted mt-0.5">This Week</p>
        </div>
      </div>

      <!-- Repo cards -->
      <div>
        <h3 class="text-xs font-semibold text-muted uppercase tracking-wider mb-2">Repositories</h3>
        <div class="space-y-2">
          <div
            v-for="stat in repoStats"
            :key="stat.repo.id"
            class="flex items-center gap-3 rounded-lg border border-default bg-elevated px-3 py-2.5"
          >
            <UIcon name="i-lucide-folder-git-2" class="size-4 text-muted shrink-0" />
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate">{{ stat.repo.name }}</p>
              <p class="text-xs text-muted truncate">
                {{ stat.totalCommits }} commits
                <template v-if="stat.latestCommit">
                  · last {{ timeAgo(stat.latestCommit.committed_at) }}
                </template>
              </p>
            </div>
            <UBadge :label="String(stat.totalCommits)" variant="soft" size="sm" />
          </div>
        </div>
      </div>

      <!-- Recent activity feed -->
      <div v-if="allCommitsWithRepo.length">
        <h3 class="text-xs font-semibold text-muted uppercase tracking-wider mb-2">Recent Activity</h3>
        <div class="rounded-lg border border-default overflow-hidden">
          <div
            v-for="commit in allCommitsWithRepo"
            :key="`${commit.repo_id}-${commit.id}`"
            class="flex items-center gap-3 px-3 py-2 border-b border-default last:border-0 hover:bg-elevated transition-colors"
          >
            <UBadge :label="commit.short_hash" variant="outline" size="sm" class="font-mono shrink-0" />
            <div class="flex-1 min-w-0">
              <p class="text-sm truncate">{{ commit.message }}</p>
              <div class="flex items-center gap-2 mt-0.5">
                <span class="text-xs text-primary/80 font-medium">{{ commit.repoName }}</span>
                <span class="text-xs text-muted">{{ commit.author }}</span>
              </div>
            </div>
            <span class="text-xs text-muted shrink-0">{{ timeAgo(commit.committed_at) }}</span>
          </div>
        </div>
      </div>

      <!-- Time logs -->
      <div v-if="timeLogs.length">
        <h3 class="text-xs font-semibold text-muted uppercase tracking-wider mb-2">Time Logged</h3>
        <div class="rounded-lg border border-default overflow-hidden">
          <template v-for="log in timeLogs" :key="log.id">
            <!-- Inline edit form -->
            <div
              v-if="editingId === log.id"
              class="px-3 py-2 border-b border-default last:border-0 bg-elevated"
            >
              <div class="flex items-center gap-2">
                <UInput
                  v-model.number="editForm.duration_minutes"
                  type="number"
                  size="xs"
                  class="w-20"
                />
                <span class="text-xs text-muted shrink-0">min</span>
                <UInput
                  v-model="editForm.comment"
                  size="xs"
                  class="flex-1"
                  placeholder="Comment"
                  @keydown.enter="saveEdit(log.id)"
                  @keydown.esc="cancelEdit"
                />
                <USelect
                  v-model="editForm.repo_id"
                  :items="repoSelectItems"
                  size="xs"
                  class="w-32"
                />
                <UButton icon="i-lucide-check" size="xs" variant="soft" @click="saveEdit(log.id)" />
                <UButton icon="i-lucide-x" size="xs" variant="ghost" @click="cancelEdit" />
              </div>
            </div>
            <!-- Normal row -->
            <div
              v-else
              class="flex items-center gap-3 px-3 py-2 border-b border-default last:border-0 hover:bg-elevated transition-colors group"
            >
              <UBadge :label="formatMinutes(log.duration_minutes)" variant="soft" color="primary" size="sm" class="shrink-0 font-mono" />
              <div class="flex-1 min-w-0">
                <p class="text-sm truncate">{{ log.comment || '—' }}</p>
                <span v-if="log.repo_name" class="text-xs text-primary/80 font-medium">{{ log.repo_name }}</span>
              </div>
              <span class="text-xs text-muted shrink-0">{{ timeAgo(log.logged_at) }}</span>
              <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
                <UButton
                  icon="i-lucide-pencil"
                  size="xs"
                  variant="ghost"
                  @click="startEdit(log)"
                />
                <UButton
                  icon="i-lucide-trash-2"
                  size="xs"
                  variant="ghost"
                  color="error"
                  @click="deleteLog(log.id)"
                />
              </div>
            </div>
          </template>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="repos.length === 0" class="flex flex-col items-center justify-center py-12 gap-2">
        <UIcon name="i-lucide-inbox" class="size-10 text-muted" />
        <p class="text-sm text-muted">No repositories added yet</p>
      </div>

    </template>
  </div>
</template>
