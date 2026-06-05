export function formatEieError(error: unknown) {
  if (error instanceof Error) {
    return error.message
  }

  if (typeof error !== "string") {
    return String(error)
  }

  try {
    const parsed = JSON.parse(error) as { message?: string }
    return parsed.message ?? error
  } catch {
    return error
  }
}
