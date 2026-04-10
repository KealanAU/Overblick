<script setup>
import { ref, onMounted } from 'vue'
import { getRecentEvents, addMockEvent, listWatchedRepos } from '../services/repos'
import { timeAgo } from '../utils/time.js'

const toast = useToast()

const events = ref([])
const repos = ref([])
const loading = ref(false)

function eventBadgeColor(type) {
  if (type === 'commit') return 'success'
  return 'neutral'
}

async function loadEvents() {
  loading.value = true
  try {
    events.value = await getRecentEvents(50)
  } catch (err) {
    toast.add({ title: 'Failed to load events', description: String(err), color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    loading.value = false
  }
}

async function loadRepos() {
  try {
    repos.value = await listWatchedRepos()
  } catch {
    // silently ignore — repos are only needed for the mock button
  }
}

async function addMockEvent() {
  if (repos.value.length === 0) return
  try {
    await addMockEvent(repos.value[0].id)
    await loadEvents()
  } catch (err) {
    toast.add({ title: 'Failed to add mock event', description: String(err), color: 'error', icon: 'i-lucide-alert-circle' })
  }
}

onMounted(async () => {
  await Promise.all([loadEvents(), loadRepos()])
})
</script>

<template>
  <div class="mt-3 space-y-3">
    <div class="flex items-center justify-between">
      <h2 class="text-sm font-semibold">Recent Activity</h2>
      <div class="flex items-center gap-2">
        <UButton
          v-if="repos.length > 0"
          label="Add Mock Event"
          size="sm"
          variant="outline"
          icon="i-lucide-flask-conical"
          @click="addMockEvent"
        />
        <UButton
          icon="i-lucide-refresh-cw"
          size="sm"
          variant="ghost"
          :loading="loading"
          @click="loadEvents"
        />
      </div>
    </div>

    <div v-if="loading && events.length === 0" class="flex justify-center py-8">
      <UIcon name="i-lucide-loader-circle" class="size-5 text-muted animate-spin" />
    </div>

    <div
      v-else-if="events.length === 0"
      class="flex flex-col items-center justify-center gap-2 py-10 text-center"
    >
      <UIcon name="i-lucide-activity" class="size-8 text-muted" />
      <p class="text-sm text-muted">No activity recorded yet.</p>
      <p class="text-xs text-muted">Events will appear here as git activity is detected.</p>
    </div>

    <div v-else class="space-y-2">
      <div
        v-for="event in events"
        :key="event.id"
        class="flex items-start gap-3 rounded-lg border border-default bg-elevated px-3 py-2"
      >
        <UBadge
          :label="event.event_type"
          :color="eventBadgeColor(event.event_type)"
          variant="subtle"
          size="sm"
          class="mt-0.5 shrink-0"
        />
        <div class="min-w-0 flex-1">
          <p class="text-sm font-medium truncate">{{ event.repo_name }}</p>
          <p class="text-xs text-muted truncate">{{ event.description }}</p>
        </div>
        <span class="text-xs text-muted shrink-0 mt-0.5">{{ timeAgo(event.occurred_at) }}</span>
      </div>
    </div>
  </div>
</template>
