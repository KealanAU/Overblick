import { ref, watch } from 'vue'

const STORAGE_KEY = 'overblick:selectedRepoId'

const selectedRepo = ref(null)  // WatchedRepo: { id, path, name, added_at } | null
const activeView = ref('commits')  // 'commits' | 'issues' | 'prs'

watch(selectedRepo, (repo) => {
  if (repo) localStorage.setItem(STORAGE_KEY, String(repo.id))
  else localStorage.removeItem(STORAGE_KEY)
})

export function useRepo() {
  return { selectedRepo, activeView }
}

export function getStoredRepoId() {
  const v = localStorage.getItem(STORAGE_KEY)
  return v ? Number(v) : null
}
