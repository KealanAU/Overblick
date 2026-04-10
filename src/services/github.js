import { invoke } from '@tauri-apps/api/core'

export async function getIssues(repoId) {
  return invoke('get_issues', { repoId })
}

export async function getPullRequests(repoId) {
  return invoke('get_pull_requests', { repoId })
}

export async function syncGithubData(repoId, token) {
  return invoke('sync_github_data', { repoId, token })
}

export async function getCommits(repoId, limit) {
  return invoke('get_commits', { repoId, limit })
}

export async function syncRepoHistory(repoId) {
  return invoke('sync_repo_history', { repoId })
}
