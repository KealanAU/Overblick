<script setup>
import { ref, onMounted } from 'vue'
import { listWatchedRepos, addWatchedRepo, removeWatchedRepo } from '../services/repos'
import { open } from '@tauri-apps/plugin-dialog'

const toast = useToast()

const repos = ref([])
const loading = ref(false)

async function loadRepos() {
  loading.value = true
  try {
    repos.value = await listWatchedRepos()
  } catch (err) {
    toast.add({ title: 'Failed to load repos', description: String(err), color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    loading.value = false
  }
}

async function addRepo() {
  const selected = await open({ directory: true, title: 'Select a Git Repository' })
  if (!selected) return

  try {
    await addWatchedRepo(selected)
    toast.add({ title: 'Repo added', icon: 'i-lucide-check', color: 'success' })
    await loadRepos()
  } catch (err) {
    toast.add({ title: 'Failed to add repo', description: String(err), color: 'error', icon: 'i-lucide-alert-circle' })
  }
}

async function removeRepo(id) {
  try {
    await removeWatchedRepo(id)
    toast.add({ title: 'Repo removed', icon: 'i-lucide-check', color: 'success' })
    await loadRepos()
  } catch (err) {
    toast.add({ title: 'Failed to remove repo', description: String(err), color: 'error', icon: 'i-lucide-alert-circle' })
  }
}

onMounted(loadRepos)
</script>

<template>
  <div class="mt-3 space-y-3">
    <div class="flex items-center justify-between">
      <h2 class="text-sm font-semibold">Watched Repositories</h2>
      <UButton
        icon="i-lucide-plus"
        label="Add Repo"
        size="sm"
        @click="addRepo"
      />
    </div>

    <div v-if="loading" class="flex justify-center py-8">
      <UIcon name="i-lucide-loader-circle" class="size-5 text-muted animate-spin" />
    </div>

    <div
      v-else-if="repos.length === 0"
      class="flex flex-col items-center justify-center gap-2 py-10 text-center"
    >
      <UIcon name="i-lucide-folder-git-2" class="size-8 text-muted" />
      <p class="text-sm text-muted">No repositories watched yet.</p>
      <p class="text-xs text-muted">Click "Add Repo" to start monitoring a git repository.</p>
    </div>

    <div v-else class="space-y-2">
      <div
        v-for="repo in repos"
        :key="repo.id"
        class="flex items-center gap-3 rounded-lg border border-default bg-elevated px-3 py-2"
      >
        <UIcon name="i-lucide-git-branch" class="size-4 shrink-0 text-muted" />
        <div class="min-w-0 flex-1">
          <p class="text-sm font-semibold truncate">{{ repo.name }}</p>
          <p class="text-xs text-muted truncate">{{ repo.path }}</p>
        </div>
        <UButton
          icon="i-lucide-trash-2"
          variant="ghost"
          color="error"
          size="xs"
          @click="removeRepo(repo.id)"
        />
      </div>
    </div>
  </div>
</template>
