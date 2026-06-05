export function formatDownloadProgress(
  receivedBytes: number,
  totalBytes?: number,
) {
  if (!totalBytes || totalBytes <= 0) {
    return "Downloading"
  }

  const percent = Math.min(100, Math.round((receivedBytes / totalBytes) * 100))
  return `${percent}%`
}
