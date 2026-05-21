export type ProviderState = {
  provider: string | null;
};

export type ReviewStatusState = {
  status: string;
};

export function pendingReviewItems<TItem extends ReviewStatusState>(items: TItem[]): TItem[] {
  return items.filter((item) => item.status === "pending_review");
}

export function sortedNonEmptyProviders<TItem extends ProviderState>(items: TItem[]): string[] {
  return Array.from(
    new Set(items.map((item) => item.provider).filter((provider): provider is string => Boolean(provider))),
  ).sort();
}

export function toggleSelection(selectedIds: string[], id: string): string[] {
  return selectedIds.includes(id)
    ? selectedIds.filter((selectedId) => selectedId !== id)
    : [...selectedIds, id];
}

export function selectedOrCurrentIds(selectedIds: string[], currentId: string | null): string[] {
  if (selectedIds.length > 0) {
    return selectedIds;
  }
  return currentId ? [currentId] : [];
}

export function reorderByIds<TItem extends { id: string }>(items: TItem[], orderedIds: string[]): TItem[] {
  const byId = new Map(items.map((item) => [item.id, item]));
  const ordered = orderedIds.flatMap((id) => {
    const item = byId.get(id);
    return item ? [item] : [];
  });
  const rest = items.filter((item) => !orderedIds.includes(item.id));
  return [...ordered, ...rest];
}

export function moveItem<TItem>(items: TItem[], fromIndex: number, toIndex: number): TItem[] {
  if (
    fromIndex < 0 ||
    toIndex < 0 ||
    fromIndex >= items.length ||
    toIndex >= items.length ||
    fromIndex === toIndex
  ) {
    return items;
  }
  const next = [...items];
  const [item] = next.splice(fromIndex, 1);
  next.splice(toIndex, 0, item);
  return next;
}
