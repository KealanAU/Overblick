import { invoke } from '@tauri-apps/api/core'

export async function summarizeActivity(provider, apiKey, model, baseUrl, days) {
  return invoke('summarize_activity', { provider, apiKey, model, baseUrl, days })
}
