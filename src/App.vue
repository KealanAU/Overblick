<script setup>
import { ref, onMounted } from 'vue'
import { listWatchedRepos, addWatchedRepo } from './services/repos'
import { logTime, refreshWidgetData } from './services/time'
import { open } from '@tauri-apps/plugin-dialog'
import { useRepo, getStoredRepoId } from './composables/useRepo'
import { useIdentity } from './composables/useIdentity'
import { useTheme } from './composables/useTheme'
import CommitList from './components/CommitList.vue'
import Overview from './components/Overview.vue'
import SettingsPanel from './components/SettingsPanel.vue'

const { selectedRepo } = useRepo()
const { username } = useIdentity()
const toast = useToast()

// Initialise theme — applies stored colours on startup
useTheme()

const repos = ref([])
const currentView = ref('overview')

const showAddMinutes = ref(false)
const minutesForm = ref({ duration_minutes: 30, comment: '', repo_id: null })
const minutesSaving = ref(false)

async function loadRepos() {
  try {
    repos.value = await listWatchedRepos()
    const storedId = getStoredRepoId()
    if (storedId) {
      const match = repos.value.find(r => r.id === storedId)
      if (match) {
        selectedRepo.value = match
        currentView.value = 'repo'
      }
    }
  } catch (err) {
    toast.add({ title: 'Error', description: String(err), color: 'error' })
  }
}

async function addRepo() {
  try {
    const selected = await open({ directory: true, title: 'Select a Git Repository' })
    if (!selected) return
    await addWatchedRepo(selected)
    toast.add({ title: 'Repo added', color: 'success', icon: 'i-lucide-check' })
    await loadRepos()
  } catch (err) {
    toast.add({ title: 'Error', description: String(err), color: 'error' })
  }
}

function openAddMinutes() {
  minutesForm.value = { duration_minutes: 30, comment: '', repo_id: selectedRepo.value?.id ?? null }
  showAddMinutes.value = true
}

async function saveMinutes() {
  if (!minutesForm.value.duration_minutes || minutesForm.value.duration_minutes <= 0) return
  minutesSaving.value = true
  try {
    await logTime(
      minutesForm.value.duration_minutes,
      minutesForm.value.repo_id,
      minutesForm.value.comment,
    )
    await refreshWidgetData()
    toast.add({ title: 'Time logged', color: 'success', icon: 'i-lucide-check' })
    showAddMinutes.value = false
  } catch (err) {
    toast.add({ title: 'Error', description: String(err), color: 'error' })
  } finally {
    minutesSaving.value = false
  }
}

function selectRepo(repo) {
  selectedRepo.value = repo
  currentView.value = 'repo'
}

function goToOverview() {
  selectedRepo.value = null
  currentView.value = 'overview'
}

function goToSettings() {
  selectedRepo.value = null
  currentView.value = 'settings'
}

onMounted(loadRepos)
</script>

<template>
  <UApp>
    <div class="h-screen flex overflow-hidden">

        <!-- Sidebar -->
        <aside class="w-64 flex-shrink-0 flex flex-col bg-elevated border-r border-default overflow-hidden">
          <div class="px-4 pt-4 pb-3 flex-shrink-0">
            <div class="flex items-center gap-2 mb-3">
              <h1 class="text-base font-semibold">Överblick</h1>
            </div>
            <UButton
              label="Add Minutes"
              icon="i-lucide-clock"
              size="sm"
              variant="soft"
              block
              @click="openAddMinutes"
            />
          </div>

          <nav class="flex-1 overflow-y-auto px-2 pb-2">
            <button
              class="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors text-left mb-2"
              :class="currentView === 'overview'
                ? 'bg-primary/10 text-primary font-medium'
                : 'text-muted hover:text-default hover:bg-elevated'"
              @click="goToOverview"
            >
              <UIcon name="i-lucide-layout-dashboard" class="size-4 shrink-0" />
              <span>Overview</span>
            </button>

            <p v-if="repos.length === 0" class="text-xs text-muted px-2 py-3">
              No repos yet
            </p>

            <template v-else>
              <div v-for="repo in repos" :key="repo.id" class="mb-0.5">
                <button
                  class="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors text-left"
                  :class="selectedRepo?.id === repo.id
                    ? 'bg-primary/10 text-primary font-medium'
                    : 'hover:bg-elevated text-default hover:text-default'"
                  @click="selectRepo(repo)"
                >
                  <UIcon name="i-lucide-folder-git-2" class="size-4 flex-shrink-0" />
                  <span class="truncate">{{ repo.name }}</span>
                </button>
              </div>
            </template>

            <button
              class="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors text-left text-muted hover:text-default hover:bg-elevated mt-1"
              @click="addRepo"
            >
              <UIcon name="i-lucide-plus" class="size-4 shrink-0" />
              <span>Add Repo</span>
            </button>
          </nav>

          <!-- Settings pinned to bottom of sidebar -->
          <div class="px-2 pb-2 border-t border-default pt-2 flex-shrink-0">
            <button
              class="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors text-left"
              :class="currentView === 'settings'
                ? 'bg-primary/10 text-primary font-medium'
                : 'text-muted hover:text-default hover:bg-elevated'"
              @click="goToSettings"
            >
              <UIcon name="i-lucide-settings" class="size-4 shrink-0" />
              <span>Settings</span>
              <span v-if="username" class="ml-auto text-xs text-muted truncate max-w-[5rem]">{{ username }}</span>
            </button>
          </div>
        </aside>

        <!-- Main content -->
        <main class="flex-1 overflow-auto bg-elevated">
          <Transition name="fade" mode="out-in">
            <Overview v-if="currentView === 'overview'" :repos="repos" key="overview" />
            <SettingsPanel v-else-if="currentView === 'settings'" key="settings" />
            <CommitList v-else-if="currentView === 'repo' && selectedRepo" :repo-id="selectedRepo.id" :key="selectedRepo.id" />
            <div
              v-else
              key="empty"
              class="h-full flex flex-col items-center justify-center gap-3 text-center px-8"
            >
              <UIcon name="i-lucide-git-branch" class="size-12 text-muted" />
              <p class="text-sm font-medium text-default">Select a repository</p>
              <p class="text-xs text-muted">Choose a repo from the sidebar to get started</p>
            </div>
          </Transition>
        </main>

    </div>

    <!-- Add Minutes modal -->
    <UModal v-model:open="showAddMinutes" title="Log Time">
      <template #body>
        <div class="flex flex-col gap-3">
          <UFormField label="Duration (minutes)">
            <UInput
              v-model.number="minutesForm.duration_minutes"
              type="number"
              min="1"
              placeholder="30"
              class="w-full"
              @keydown.enter="saveMinutes"
            />
          </UFormField>
          <UFormField label="Comment">
            <UInput
              v-model="minutesForm.comment"
              placeholder="What were you working on?"
              class="w-full"
              @keydown.enter="saveMinutes"
            />
          </UFormField>
          <UFormField label="Repository (optional)">
            <USelect
              v-model="minutesForm.repo_id"
              :items="[{ label: '— none —', value: null }, ...repos.map(r => ({ label: r.name, value: r.id }))]"
              class="w-full"
            />
          </UFormField>
        </div>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton label="Cancel" variant="ghost" @click="showAddMinutes = false" />
          <UButton label="Log Time" :loading="minutesSaving" @click="saveMinutes" />
        </div>
      </template>
    </UModal>

  </UApp>
</template>
