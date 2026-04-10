import { invoke } from '@tauri-apps/api/core'

export async function listWatchedRepos() {
  return invoke('list_watched_repos')
}

export async function addWatchedRepo(path) {
  return invoke('add_watched_repo', { path })
}

export async function removeWatchedRepo(id) {
  return invoke('remove_watched_repo', { id })
}

export async function getRecentEvents(limit) {
  return invoke('get_recent_events', { limit })
}

export async function addMockEvent(repoId) {
  return invoke('add_mock_event', { repoId })
}
