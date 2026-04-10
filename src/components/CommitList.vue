<script setup>
import { ref, computed, watch } from 'vue'
import { getCommits, syncRepoHistory } from '../services/github'
import CalendarView from './CalendarView.vue'
import { useIdentity } from '../composables/useIdentity'
import { timeAgo } from '../utils/time.js'

const props = defineProps({ repoId: { type: Number, required: true } })

const toast = useToast()
const { username } = useIdentity()

const commits = ref([])
const loading = ref(false)
const syncing = ref(false)
const viewMode = ref('list')
const mineOnly = ref(true)

// Extract username from GitHub-style emails:
// "username@users.noreply.github.com" or "id+username@users.noreply.github.com"
function emailUsername(email) {
  const local = email.split('@')[0] ?? ''
  return local.includes('+') ? local.split('+')[1] : local
}

const visibleCommits = computed(() =>
  mineOnly.value && username.value
    ? commits.value.filter(c => {
        const u = username.value.toLowerCase()
        return emailUsername(c.author_email).toLowerCase() === u ||
               c.author.toLowerCase() === u
      })
    : commits.value
)

const groupedCommits = computed(() => {
  if (!visibleCommits.value.length) return []

  function getMonday(date) {
    const d = new Date(date)
    d.setHours(0, 0, 0, 0)
    const dow = (d.getDay() + 6) % 7
    d.setDate(d.getDate() - dow)
    return d
  }

  const groups = new Map()
  for (const c of visibleCommits.value) {
    const d = new Date(c.committed_at * 1000)
    const monday = getMonday(d)
    const key = monday.toISOString().split('T')[0]
    if (!groups.has(key)) groups.set(key, { monday, commits: [] })
    groups.get(key).commits.push(c)
  }

  const nowMonday = getMonday(new Date())
  const sorted = [...groups.entries()].sort((a, b) => b[0].localeCompare(a[0]))

  return sorted.map(([key, { monday, commits }]) => {
    const sunday = new Date(monday)
    sunday.setDate(monday.getDate() + 6)
    const diffWeeks = Math.round((nowMonday - monday) / (7 * 86400000))
    let label
    if (diffWeeks === 0) label = 'This Week'
    else if (diffWeeks === 1) label = 'Last Week'
    else {
      const fmt = (d) => d.toLocaleDateString('en', { month: 'short', day: 'numeric' })
      label = `${fmt(monday)} – ${fmt(sunday)}`
    }
    return { key, label, commits }
  })
})

async function loadCommits() {
  loading.value = true
  try {
    commits.value = await getCommits(props.repoId, 300)
  } catch (err) {
    toast.add({ title: 'Failed to load commits', description: String(err), color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    loading.value = false
  }
}

async function syncHistory() {
  syncing.value = true
  try {
    const count = await syncRepoHistory(props.repoId)
    toast.add({ title: `Synced ${count} commits`, color: 'success', icon: 'i-lucide-check' })
    await loadCommits()
  } catch (err) {
    toast.add({ title: 'Failed to sync history', description: String(err), color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    syncing.value = false
  }
}

watch(() => props.repoId, loadCommits, { immediate: true })
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="flex items-center justify-between px-3 py-2 shrink-0">
      <h2 class="text-sm font-semibold text-default">Commits</h2>
      <div class="flex items-center gap-2">
        <div class="flex gap-1">
          <UButton
            size="xs" variant="ghost"
            icon="i-lucide-list"
            :color="viewMode === 'list' ? 'primary' : 'neutral'"
            @click="viewMode = 'list'"
          />
          <UButton
            size="xs" variant="ghost"
            icon="i-lucide-calendar-days"
            :color="viewMode === 'calendar' ? 'primary' : 'neutral'"
            @click="viewMode = 'calendar'"
          />
        </div>
        <UButton
          size="xs"
          variant="ghost"
          icon="i-lucide-user"
          label="Mine"
          :color="mineOnly ? 'primary' : 'neutral'"
          @click="mineOnly = !mineOnly"
        />
        <UButton
          label="Sync History"
          size="sm"
          icon="i-lucide-refresh-cw"
          :loading="syncing"
          @click="syncHistory"
        />
      </div>
    </div>

    <div v-if="loading && commits.length === 0" class="flex flex-1 items-center justify-center">
      <UIcon name="i-lucide-loader-circle" class="size-5 text-muted animate-spin" />
    </div>

    <div
      v-else-if="visibleCommits.length === 0"
      class="flex flex-1 flex-col items-center justify-center gap-2 py-10 text-center"
    >
      <UIcon name="i-lucide-git-commit-horizontal" class="size-10 text-muted" />
      <p class="text-sm text-muted">{{ mineOnly ? 'No commits by you.' : 'No commits synced yet.' }}</p>
      <p class="text-xs text-muted">{{ mineOnly ? `Filtering by username: ${username || '(no username set)'}` : 'Click Sync History to load commit history.' }}</p>
    </div>

    <div v-else-if="viewMode === 'list'" class="overflow-y-auto flex-1">
      <div v-for="group in groupedCommits" :key="group.key">
        <!-- Week header -->
        <div class="px-3 py-1.5 text-xs font-semibold text-muted bg-elevated border-b border-default sticky top-0">
          {{ group.label }}
          <span class="font-normal text-muted/70 ml-1">({{ group.commits.length }})</span>
        </div>
        <!-- Commits in this week -->
        <div
          v-for="commit in group.commits"
          :key="commit.id"
          class="flex items-center gap-3 px-3 py-2 border-b border-default last:border-0 hover:bg-elevated transition-colors"
        >
          <UBadge :label="commit.short_hash" variant="outline" size="sm" class="font-mono shrink-0" />
          <div class="flex-1 min-w-0">
            <p class="text-sm truncate">{{ commit.message }}</p>
            <div class="flex items-center gap-1 mt-0.5">
              <UIcon name="i-lucide-user" class="size-3 text-muted" />
              <span class="text-xs text-muted">{{ commit.author }}</span>
            </div>
          </div>
          <span class="text-xs text-muted shrink-0">{{ timeAgo(commit.committed_at) }}</span>
        </div>
      </div>
    </div>

    <CalendarView
      v-else-if="viewMode === 'calendar'"
      :commits="commits"
      class="flex-1 overflow-hidden"
    />
  </div>
</template>
