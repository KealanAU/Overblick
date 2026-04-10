import { invoke } from '@tauri-apps/api/core'

export async function getTimeLogs(limit) {
  return invoke('get_time_logs', { limit })
}

export async function getTimeSummary() {
  return invoke('get_time_summary')
}

export async function updateTimeLog(id, durationMinutes, comment, repoId, loggedAt) {
  return invoke('update_time_log', { id, durationMinutes, comment, repoId, loggedAt })
}

export async function deleteTimeLog(id) {
  return invoke('delete_time_log', { id })
}

export async function logTime(durationMinutes, repoId, comment) {
  return invoke('log_time', { durationMinutes, repoId, comment })
}

export async function refreshWidgetData() {
  return invoke('refresh_widget_data')
}
