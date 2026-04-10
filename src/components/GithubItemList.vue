<script setup>
import { ref, computed, watch } from 'vue'
import { getPullRequests, getIssues, syncGithubData } from '../services/github'
import { useIdentity } from '../composables/useIdentity'
import { timeAgo } from '../utils/time.js'

const props = defineProps({
  repoId: { type: Number, required: true },
  type: { type: String, required: true }
})

const toast = useToast()
const { username, githubToken } = useIdentity()

const isPR = computed(() => props.type === 'pr')

const items = ref([])
const loading = ref(false)
const syncing = ref(false)
const mineOnly = ref(true)

const visibleItems = computed(() =>
  mineOnly.value && username.value
    ? items.value.filter(i => i.author.toLowerCase() === username.value.toLowerCase())
    : items.value
)

function badgeColor(state) {
  if (!isPR.value) return state === 'open' ? 'success' : 'neutral'
  if (state === 'open') return 'info'
  if (state === 'merged') return 'primary'
  return 'neutral'
}

function badgeLabel(state) {
  if (!isPR.value) return state === 'open' ? 'open' : 'closed'
  return state
}

async function loadItems() {
  loading.value = true
  try {
    items.value = await (isPR.value ? getPullRequests(props.repoId) : getIssues(props.repoId))
  } catch (err) {
    toast.add({ title: `Failed to load ${isPR.value ? 'pull requests' : 'issues'}`, description: String(err), color: 'error' })
  } finally {
    loading.value = false
  }
}

async function syncGithub() {
  syncing.value = true
  try {
    const result = await syncGithubData(props.repoId, githubToken.value || null)
    toast.add({ title: `Synced ${result.issues_synced} issues, ${result.prs_synced} PRs`, color: 'success' })
    await loadItems()
  } catch (err) {
    toast.add({ title: 'Sync failed', description: String(err), color: 'error' })
  } finally {
    syncing.value = false
  }
}

watch(() => props.repoId, loadItems, { immediate: true })
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="flex items-center justify-between mb-3 shrink-0">
      <h2 class="text-sm font-semibold">{{ isPR ? 'Pull Requests' : 'Issues' }}</h2>
      <div class="flex items-center gap-2">
        <UButton
          size="xs"
          variant="ghost"
          icon="i-lucide-user"
          label="Mine"
          :color="mineOnly ? 'primary' : 'neutral'"
          @click="mineOnly = !mineOnly"
        />
        <UButton
          size="sm"
          icon="i-lucide-github"
          :loading="syncing"
          @click="syncGithub"
        >
          Sync from GitHub
        </UButton>
      </div>
    </div>

    <div v-if="loading && items.length === 0" class="flex flex-1 items-center justify-center">
      <UIcon name="i-lucide-loader-circle" class="animate-spin text-muted" />
    </div>

    <div v-else-if="visibleItems.length === 0" class="flex flex-1 flex-col items-center justify-center gap-2">
      <UIcon :name="isPR ? 'i-lucide-git-pull-request' : 'i-lucide-circle-dot'" class="size-10 text-muted" />
      <p class="text-sm text-muted">{{ mineOnly ? `No ${isPR ? 'pull requests' : 'issues'} by you.` : `No ${isPR ? 'pull requests' : 'issues'} synced yet.` }}</p>
      <p class="text-xs text-muted">{{ mineOnly ? `Filtering by author: ${username || '(no username set)'}` : 'Click Sync from GitHub — requires a GitHub remote.' }}</p>
    </div>

    <div v-else class="overflow-y-auto flex-1">
      <div
        v-for="item in visibleItems"
        :key="item.id"
        class="flex items-start gap-3 px-3 py-2.5 border-b border-default last:border-0 hover:bg-elevated transition-colors"
      >
        <div class="flex flex-col items-start gap-1 shrink-0 mt-0.5">
          <UBadge
            :color="badgeColor(item.state)"
            :label="badgeLabel(item.state)"
            variant="subtle"
            size="sm"
          />
          <span class="text-xs text-muted font-mono">#{{ item.number }}</span>
        </div>

        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium truncate">{{ item.title }}</p>
          <div class="flex items-center gap-1 mt-0.5">
            <UIcon name="i-lucide-user" class="size-3 text-muted" />
            <span class="text-xs text-muted">{{ item.author }}</span>
          </div>
        </div>

        <span class="text-xs text-muted shrink-0">{{ timeAgo(item.created_at) }}</span>
      </div>
    </div>
  </div>
</template>
