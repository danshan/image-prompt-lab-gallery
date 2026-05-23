export type VersionTreeNodeState = {
  versionId: string;
  children: VersionTreeNodeState[];
};

export type VisibleVersionTreeNode<TNode extends VersionTreeNodeState> = {
  node: TNode;
  depth: number;
  parentId: string | null;
};

export type VersionTreeSummaryState = {
  versionCount: number;
  versionTreeBranchCount?: number;
};

export function collectExpandableVersionIds<TNode extends VersionTreeNodeState>(nodes: TNode[]): string[] {
  return nodes.flatMap((node) => [
    ...(node.children.length > 0 ? [node.versionId] : []),
    ...collectExpandableVersionIds(node.children),
  ]);
}

export function flattenVisibleVersionTree<TNode extends VersionTreeNodeState>(
  nodes: TNode[],
  expandedIds: Set<string>,
  depth = 0,
  parentId: string | null = null,
): VisibleVersionTreeNode<TNode>[] {
  return nodes.flatMap((node) => {
    const current = { node, depth, parentId };
    if (!expandedIds.has(node.versionId)) {
      return [current];
    }
    return [
      current,
      ...flattenVisibleVersionTree(node.children as TNode[], expandedIds, depth + 1, node.versionId),
    ];
  });
}

export function formatVersionTreeSummary(asset: VersionTreeSummaryState) {
  const versionText = `${asset.versionCount} version${asset.versionCount === 1 ? "" : "s"}`;
  const branchCount = asset.versionTreeBranchCount ?? 0;
  if (branchCount === 0) {
    return versionText;
  }
  return `${versionText} / ${branchCount} branch${branchCount === 1 ? "" : "es"}`;
}
