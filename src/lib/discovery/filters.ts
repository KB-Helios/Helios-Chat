import type { FitFilter, FitModelQuery, FitSort } from "./types"

export function createDefaultFitQuery(): FitModelQuery {
  return {
    fit: "runnable",
    includeTooTight: false,
    limit: 50,
    sort: "score",
  }
}

export function normalizeFitQuery(
  input: Partial<FitModelQuery>,
): FitModelQuery {
  const fit: FitFilter = input.fit ?? "runnable"
  const sort: FitSort = input.sort ?? "score"
  const search = input.search?.trim()

  return {
    fit,
    includeTooTight: fit === "all" || fit === "tooTight",
    limit: Math.min(Math.max(input.limit ?? 50, 1), 100),
    search: search ? search : undefined,
    sort,
  }
}
