import assert from "node:assert/strict";
import test from "node:test";
import {
  acceptSuggestionState,
  albumContentsQuery,
  applySuggestionFieldToReviewForm,
  applyGalleryQuery,
  beginDetailLoad,
  beginReviewFieldGeneration,
  buildBatchReviewPayloads,
  clearGalleryAlbumFilter,
  clearGalleryMinRatingFilter,
  clearGalleryProviderFilter,
  clearGalleryReviewFilter,
  clearGalleryTagFilter,
  clearGalleryTextFilter,
  clearAlbumQuery,
  clearSelectedAlbumState,
  clearCurationStateForLibrarySwitch,
  clearLibraryWorkspaceState,
  clearSelectionForLibrarySwitch,
  collectExpandableVersionIds,
  completeDetailLoad,
  completeReviewFieldGeneration,
  countActiveTasks,
  createReviewFormState,
  defaultAlbumAddSourceQuery,
  defaultSettingsSection,
  defaultGalleryQuery,
  failReviewFieldGeneration,
  failDetailLoad,
  filterAlbumAddCandidates,
  flattenVisibleVersionTree,
  formatAspectRatio,
  formatVersionTreeSummary,
  galleryAlbumFilterIds,
  addReviewFormTag,
  isReviewFieldGenerating,
  libraryMaintenanceActions,
  markAssetReviewPending,
  moveItem,
  moveQueuedTaskOrder,
  openAlbumQuery,
  parseTaskDraftImport,
  pendingReviewItems,
  removeReviewFormTag,
  removeSuggestionState,
  reorderByIds,
  resetGalleryQuery,
  removeGalleryAlbumFilter,
  reviewFormTags,
  selectedOrCurrentIds,
  selectAlbumState,
  setGalleryAlbumFilter,
  setGalleryUnassignedAlbumFilter,
  sortedNonEmptyProviders,
  toggleGalleryAlbumFilter,
  toggleGalleryProvider,
  toggleGalleryTag,
  toggleSelection,
} from "../.test-dist/workbench-state.js";

test("settings defaults to libraries section", () => {
  assert.equal(defaultSettingsSection, "libraries");
});

test("settings sections include providers diagnostics", () => {
  const section = "providers";
  assert.equal(section, "providers");
});

test("acceptSuggestionState applies metadata and removes pending item", () => {
  const assets = [
    {
      id: "asset-1",
      title: null,
      category: null,
      rating: 3,
      tags: [],
    },
  ];
  const suggestions = [
    {
      id: "suggestion-1",
      assetId: "asset-1",
      title: "Final Title",
      category: "study",
      tags: ["reviewed"],
    },
  ];

  const next = acceptSuggestionState(assets, suggestions, suggestions[0]);

  assert.equal(next.assets[0].title, "Final Title");
  assert.equal(next.assets[0].category, "study");
  assert.deepEqual(next.assets[0].tags, ["reviewed"]);
  assert.equal(next.suggestions.length, 0);
});

test("removeSuggestionState removes only the completed suggestion", () => {
  const suggestions = [
    {
      id: "suggestion-1",
      assetId: "asset-1",
      title: null,
      category: null,
      tags: [],
    },
    {
      id: "suggestion-2",
      assetId: "asset-2",
      title: null,
      category: null,
      tags: [],
    },
  ];

  const next = removeSuggestionState(suggestions, "suggestion-1");

  assert.deepEqual(
    next.map((suggestion) => suggestion.id),
    ["suggestion-2"],
  );
});

test("selection helpers toggle ids and fall back to current item", () => {
  assert.deepEqual(toggleSelection([], "suggestion-1"), ["suggestion-1"]);
  assert.deepEqual(toggleSelection(["suggestion-1"], "suggestion-1"), []);
  assert.deepEqual(selectedOrCurrentIds(["a", "b"], "c"), ["a", "b"]);
  assert.deepEqual(selectedOrCurrentIds([], "c"), ["c"]);
  assert.deepEqual(selectedOrCurrentIds([], null), []);
});

test("reorder helpers keep unknown ids out and preserve remaining items", () => {
  const items = [{ id: "a" }, { id: "b" }, { id: "c" }];
  assert.deepEqual(reorderByIds(items, ["c", "a"]).map((item) => item.id), ["c", "a", "b"]);
  assert.deepEqual(moveItem(items, 0, 2).map((item) => item.id), ["b", "c", "a"]);
  assert.equal(moveItem(items, -1, 2), items);
});

test("markAssetReviewPending raises pending count without touching other assets", () => {
  const next = markAssetReviewPending(
    [
      { id: "asset-1", title: null, category: null, rating: null, tags: [], reviewPendingCount: 0 },
      { id: "asset-2", title: null, category: null, rating: null, tags: [], reviewPendingCount: 2 },
    ],
    "asset-1",
  );
  assert.equal(next[0].reviewPendingCount, 1);
  assert.equal(next[1].reviewPendingCount, 2);
});

test("parseTaskDraftImport keeps multi-line prompts inside one task", () => {
  const drafts = parseTaskDraftImport(
    JSON.stringify({
      tasks: [
        {
          prompt: "line one\nline two\nline three",
          provider: "fake",
          parameters: { size: "1024x1024" },
        },
      ],
    }),
  );

  assert.equal(drafts.length, 1);
  assert.equal(drafts[0].prompt, "line one\nline two\nline three");
  assert.equal(drafts[0].provider, "fake");
  assert.equal(drafts[0].parametersJson, "{\n  \"size\": \"1024x1024\"\n}");
});

test("moveQueuedTaskOrder only reorders queued tasks", () => {
  const tasks = [
    { id: "running", status: "running" },
    { id: "queued-a", status: "queued" },
    { id: "failed", status: "failed_final" },
    { id: "queued-b", status: "queued" },
  ];

  assert.deepEqual(moveQueuedTaskOrder(tasks, "queued-b", -1), ["queued-b", "queued-a"]);
  assert.deepEqual(moveQueuedTaskOrder(tasks, "running", 1), ["queued-a", "queued-b"]);
});

test("derived workbench helpers keep expensive render inputs stable", () => {
  assert.equal(
    countActiveTasks([
      { id: "queued", status: "queued" },
      { id: "retrying", status: "retry_waiting" },
      { id: "done", status: "completed" },
    ]),
    2,
  );
  assert.deepEqual(
    pendingReviewItems([
      { id: "pending", status: "pending_review" },
      { id: "accepted", status: "accepted" },
    ]).map((item) => item.id),
    ["pending"],
  );
  assert.deepEqual(
    sortedNonEmptyProviders([
      { provider: "fake" },
      { provider: null },
      { provider: "codex-cli" },
      { provider: "fake" },
    ]),
    ["codex-cli", "fake"],
  );
});

test("gallery query helpers toggle providers and tags", () => {
  const withProvider = toggleGalleryProvider(defaultGalleryQuery, "fake");
  assert.deepEqual(withProvider.providers, ["fake"]);
  assert.deepEqual(toggleGalleryProvider(withProvider, "fake").providers, []);

  const withTag = toggleGalleryTag(defaultGalleryQuery, "neon");
  assert.deepEqual(withTag.tags, ["neon"]);
  assert.deepEqual(toggleGalleryTag(withTag, "neon").tags, []);
});

test("resetGalleryQuery returns independent arrays", () => {
  const first = resetGalleryQuery();
  const second = resetGalleryQuery();
  first.providers.push("fake");

  assert.deepEqual(second.providers, []);
});

test("gallery album filters are independent from albums selection", () => {
  const selectedAlbumId = selectAlbumState(null, "album-1");
  const galleryQuery = setGalleryAlbumFilter(defaultGalleryQuery, ["album-2", "album-3"]);

  assert.equal(selectedAlbumId, "album-1");
  assert.deepEqual(galleryAlbumFilterIds(galleryQuery), ["album-2", "album-3"]);
  assert.equal(clearSelectedAlbumState(), null);
  assert.deepEqual(clearGalleryAlbumFilter(galleryQuery).albumFilter, { mode: "any" });
});

test("gallery album filter supports toggle and unassigned mode", () => {
  const oneAlbum = toggleGalleryAlbumFilter(defaultGalleryQuery, "album-1");
  const twoAlbums = toggleGalleryAlbumFilter(oneAlbum, "album-2");
  const removed = toggleGalleryAlbumFilter(twoAlbums, "album-1");
  const unassigned = setGalleryUnassignedAlbumFilter(twoAlbums);

  assert.deepEqual(galleryAlbumFilterIds(twoAlbums), ["album-1", "album-2"]);
  assert.deepEqual(galleryAlbumFilterIds(removed), ["album-2"]);
  assert.deepEqual(unassigned.albumFilter, { mode: "unassigned" });
  assert.equal(unassigned.sort, "newest");
});

test("gallery filter helpers clear one active filter at a time", () => {
  const query = {
    ...defaultGalleryQuery,
    text: "botanical",
    providers: ["fake", "codex-cli"],
    minRating: 4,
    reviewStatus: "pending",
    tags: ["neon", "macro"],
    albumFilter: { mode: "inAny", albumIds: ["album-1", "album-2"] },
    sort: "albumOrder",
  };

  assert.equal(clearGalleryTextFilter(query).text, "");
  assert.deepEqual(clearGalleryProviderFilter(query, "fake").providers, ["codex-cli"]);
  assert.equal(clearGalleryMinRatingFilter(query).minRating, null);
  assert.equal(clearGalleryReviewFilter(query).reviewStatus, "any");
  assert.deepEqual(clearGalleryTagFilter(query, "neon").tags, ["macro"]);
  assert.equal(clearGalleryAlbumFilter(query).sort, "newest");

  const oneAlbumLeft = removeGalleryAlbumFilter(query, "album-1");
  assert.deepEqual(oneAlbumLeft.albumFilter, { mode: "inAny", albumIds: ["album-2"] });
  assert.equal(oneAlbumLeft.sort, "newest");
  assert.deepEqual(removeGalleryAlbumFilter(oneAlbumLeft, "album-2").albumFilter, { mode: "any" });
});

test("album contents and add drawer queries are separate query objects", () => {
  const galleryQuery = {
    ...defaultGalleryQuery,
    providers: ["fake"],
  };
  const albumQuery = albumContentsQuery("album-1", "manual");
  const addQuery = defaultAlbumAddSourceQuery();

  assert.deepEqual(galleryQuery.providers, ["fake"]);
  assert.deepEqual(albumQuery.albumFilter, { mode: "inAny", albumIds: ["album-1"] });
  assert.equal(albumQuery.sort, "albumOrder");
  assert.deepEqual(addQuery.albumFilter, { mode: "any" });
  assert.notEqual(albumQuery, addQuery);
});

test("album add candidates exclude current album members", () => {
  const assets = [
    { id: "in-album", albums: [{ id: "album-1" }] },
    { id: "other", albums: [{ id: "album-2" }] },
    { id: "free", albums: [] },
  ];

  assert.deepEqual(
    filterAlbumAddCandidates(assets, "album-1").map((asset) => asset.id),
    ["other", "free"],
  );

  const albumQuery = openAlbumQuery(defaultGalleryQuery, "album-1");
  assert.deepEqual(albumQuery.albumFilter, { mode: "inAny", albumIds: ["album-1"] });
  assert.deepEqual(clearAlbumQuery(albumQuery).albumFilter, { mode: "any" });
});

test("review form state is initialized from suggestion and parses tags", () => {
  const form = createReviewFormState({
    id: "suggestion-1",
    title: "Suggested title",
    description: "Suggested description",
    schemaPrompt: "{\"OUTPUT\":{\"mood\":\"editorial\"}}",
    category: "study",
    tags: ["botanical", "neon", "botanical"],
  });

  assert.equal(form.suggestionId, "suggestion-1");
  assert.equal(form.title, "Suggested title");
  assert.equal(form.description, "Suggested description");
  assert.equal(form.schemaPrompt, "{\"OUTPUT\":{\"mood\":\"editorial\"}}");
  assert.equal(form.category, "study");
  assert.equal(form.generation.title.loading, false);
  assert.deepEqual(form.tags, ["botanical", "neon"]);
  const added = addReviewFormTag({ ...form, tagInput: " neon " }, " neon ");
  assert.deepEqual(added.tags, ["botanical", "neon"]);
  assert.deepEqual(addReviewFormTag(added, "study").tags, ["botanical", "neon", "study"]);
  assert.deepEqual(removeReviewFormTag(added, "neon").tags, ["botanical"]);
  assert.deepEqual(reviewFormTags({ ...form, tags: [" botanical ", "", "neon"] }), ["botanical", "neon"]);
});

test("review field generation tracks loading per field and updates only the completed field", () => {
  const form = createReviewFormState({
    id: "suggestion-1",
    title: "Old title",
    description: "Old description",
    schemaPrompt: "{}",
    category: null,
    tags: [],
  });

  const loading = beginReviewFieldGeneration(form, "title", "request-1");
  assert.equal(isReviewFieldGenerating(loading, "title"), true);
  assert.equal(isReviewFieldGenerating(loading, "description"), false);

  const completed = completeReviewFieldGeneration(
    loading,
    "suggestion-1",
    "title",
    "request-1",
    "New title",
    "/tmp/imglab-codex-metadata-1.log",
  );

  assert.equal(completed.title, "New title");
  assert.equal(completed.description, "Old description");
  assert.equal(completed.generation.title.loading, false);
  assert.equal(completed.generation.title.logPath, "/tmp/imglab-codex-metadata-1.log");
});

test("history field pick updates only the review draft", () => {
  const form = createReviewFormState({
    id: "suggestion-1",
    title: "Current",
    description: "Current description",
    schemaPrompt: "{}",
    category: null,
    tags: ["current"],
  });
  const history = {
    id: "suggestion-0",
    title: "History title",
    description: "History description",
    schemaPrompt: "{\"a\":1}",
    category: "study",
    tags: ["history"],
  };

  assert.equal(applySuggestionFieldToReviewForm(form, history, "title").title, "History title");
  assert.deepEqual(applySuggestionFieldToReviewForm(form, history, "tags").tags, ["history"]);
  assert.equal(applySuggestionFieldToReviewForm(form, history, "category").category, "study");
});

test("batch review payloads use current draft for active suggestion", () => {
  const suggestions = [
    {
      id: "suggestion-1",
      assetId: "asset-1",
      title: "Stored title",
      description: "Stored description",
      schemaPrompt: "{}",
      category: null,
      tags: ["stored"],
    },
    {
      id: "suggestion-2",
      assetId: "asset-2",
      title: "Other",
      description: null,
      schemaPrompt: null,
      category: "study",
      tags: ["other"],
    },
  ];
  const form = {
    ...createReviewFormState(suggestions[0]),
    title: "Draft title",
    tags: ["draft"],
  };

  const payloads = buildBatchReviewPayloads(suggestions, ["suggestion-1", "suggestion-2"], form);

  assert.equal(payloads[0].title, "Draft title");
  assert.deepEqual(payloads[0].tags, ["draft"]);
  assert.equal(payloads[1].title, "Other");
});

test("review field generation failure preserves the draft", () => {
  const form = beginReviewFieldGeneration(
    createReviewFormState({
      id: "suggestion-1",
      title: "Draft title",
      description: "Draft description",
      schemaPrompt: "{}",
      category: null,
      tags: [],
    }),
    "description",
    "request-1",
  );

  const failed = failReviewFieldGeneration(
    form,
    "suggestion-1",
    "description",
    "request-1",
    "codex failed",
  );

  assert.equal(failed.description, "Draft description");
  assert.equal(failed.generation.description.loading, false);
  assert.equal(failed.generation.description.error, "codex failed");
});

test("review field generation ignores stale responses", () => {
  const form = beginReviewFieldGeneration(
    createReviewFormState({
      id: "suggestion-1",
      title: "Draft title",
      description: "",
      schemaPrompt: "{}",
      category: null,
      tags: [],
    }),
    "title",
    "request-1",
  );

  const wrongSuggestion = completeReviewFieldGeneration(
    form,
    "suggestion-2",
    "title",
    "request-1",
    "Wrong title",
  );
  const wrongRequest = completeReviewFieldGeneration(
    form,
    "suggestion-1",
    "title",
    "request-2",
    "Wrong title",
  );

  assert.equal(wrongSuggestion.title, "Draft title");
  assert.equal(wrongSuggestion.generation.title.loading, true);
  assert.equal(wrongRequest.title, "Draft title");
  assert.equal(wrongRequest.generation.title.loading, true);
});

test("detail load helpers model loading lifecycle", () => {
  assert.deepEqual(beginDetailLoad("asset-1"), {
    assetId: "asset-1",
    detail: null,
    loading: true,
    error: null,
  });
  assert.deepEqual(completeDetailLoad("asset-1", { id: "asset-1" }), {
    assetId: "asset-1",
    detail: { id: "asset-1" },
    loading: false,
    error: null,
  });
  assert.deepEqual(failDetailLoad("asset-1", "boom"), {
    assetId: "asset-1",
    detail: null,
    loading: false,
    error: "boom",
  });
});

test("library switching clears stale detail while preserving query object", () => {
  const query = {
    ...defaultGalleryQuery,
    text: "botanical",
    providers: ["fake"],
  };
  const detail = clearSelectionForLibrarySwitch();

  assert.deepEqual(detail, {
    assetId: null,
    detail: null,
    loading: false,
    error: null,
  });
  assert.deepEqual(query, {
    ...defaultGalleryQuery,
    text: "botanical",
    providers: ["fake"],
  });
});

test("library switching clears album and review state", () => {
  assert.deepEqual(clearCurationStateForLibrarySwitch(), {
    selectedAlbumId: null,
    selectedSuggestionId: null,
    reviewForm: null,
  });
});

test("close current library clears workspace selections", () => {
  const cleared = clearLibraryWorkspaceState();
  assert.equal(cleared.selectedAssetId, "");
  assert.deepEqual(cleared.selectedGalleryAssetIds, []);
  assert.equal(cleared.detailState.assetId, null);
  assert.equal(cleared.selectedAlbumId, null);
  assert.equal(cleared.selectedSuggestionId, null);
  assert.deepEqual(cleared.selectedSuggestionIds, []);
  assert.equal(cleared.reviewForm, null);
  assert.equal(cleared.selectedTaskId, null);
});

test("missing library paths only keep close action enabled", () => {
  assert.deepEqual(libraryMaintenanceActions("/tmp/missing", ["/tmp/missing"]), {
    canClose: true,
    canExport: false,
    canReveal: false,
  });
  assert.deepEqual(libraryMaintenanceActions("/tmp/present", ["/tmp/missing"]), {
    canClose: true,
    canExport: true,
    canReveal: true,
  });
});

test("applyGalleryQuery filters by text, tags, review status, and rating", () => {
  const assets = [
    {
      title: "Neon Botanical Study",
      category: "study",
      rating: 5,
      status: "generated",
      provider: "Midjourney",
      modelLabel: "v6",
      prompt: "macro botanical prompt with neon linework",
      tags: ["botanical", "neon"],
      reviewPendingCount: 1,
      createdAt: "1",
      updatedAt: "2",
    },
    {
      title: "Rainy Tokyo Night",
      category: "city",
      rating: 4,
      status: "curated",
      provider: "DALL-E 3",
      modelLabel: "standard",
      prompt: "rainy city prompt",
      tags: ["city", "rain"],
      reviewPendingCount: 0,
      createdAt: "2",
      updatedAt: "3",
    },
  ];

  const results = applyGalleryQuery(assets, {
    ...defaultGalleryQuery,
    text: "botanical",
    minRating: 5,
    reviewStatus: "pending",
    tags: ["neon"],
  });

  assert.equal(results.length, 1);
  assert.equal(results[0].title, "Neon Botanical Study");
});

test("applyGalleryQuery filters by album union and unassigned state", () => {
  const assets = [
    {
      title: "A",
      category: null,
      rating: null,
      status: "generated",
      provider: "fake",
      modelLabel: null,
      tags: [],
      reviewPendingCount: 0,
      albums: [{ id: "album-a" }],
      createdAt: "1",
      updatedAt: "1",
    },
    {
      title: "B",
      category: null,
      rating: null,
      status: "generated",
      provider: "fake",
      modelLabel: null,
      tags: [],
      reviewPendingCount: 0,
      albums: [{ id: "album-b" }],
      createdAt: "2",
      updatedAt: "2",
    },
    {
      title: "Free",
      category: null,
      rating: null,
      status: "generated",
      provider: "fake",
      modelLabel: null,
      tags: [],
      reviewPendingCount: 0,
      albums: [],
      createdAt: "3",
      updatedAt: "3",
    },
  ];

  const union = applyGalleryQuery(assets, {
    ...defaultGalleryQuery,
    albumFilter: { mode: "inAny", albumIds: ["album-a", "album-b"] },
  });
  const unassigned = applyGalleryQuery(assets, {
    ...defaultGalleryQuery,
    albumFilter: { mode: "unassigned" },
  });

  assert.deepEqual(union.map((asset) => asset.title).sort(), ["A", "B"]);
  assert.deepEqual(unassigned.map((asset) => asset.title), ["Free"]);
});

test("applyGalleryQuery matches prompt text", () => {
  const assets = [
    {
      title: null,
      category: null,
      rating: null,
      status: "generated",
      provider: "fake",
      modelLabel: "fake-image",
      prompt: "tiny icon sheet with transparent background",
      tags: [],
      reviewPendingCount: 0,
      createdAt: "1",
      updatedAt: "1",
    },
  ];

  const results = applyGalleryQuery(assets, {
    ...defaultGalleryQuery,
    text: "transparent",
  });

  assert.equal(results.length, 1);
});

test("applyGalleryQuery sorts by rating descending", () => {
  const assets = [
    {
      title: "Lower",
      category: null,
      rating: 2,
      status: "generated",
      provider: "fake",
      modelLabel: null,
      tags: [],
      reviewPendingCount: 0,
      createdAt: "1",
      updatedAt: "1",
    },
    {
      title: "Higher",
      category: null,
      rating: 5,
      status: "generated",
      provider: "fake",
      modelLabel: null,
      tags: [],
      reviewPendingCount: 0,
      createdAt: "2",
      updatedAt: "2",
    },
  ];

  const results = applyGalleryQuery(assets, {
    ...defaultGalleryQuery,
    sort: "ratingDesc",
  });

  assert.equal(results[0].title, "Higher");
});

test("formatAspectRatio reduces valid dimensions", () => {
  assert.equal(formatAspectRatio(1024, 1024), "1:1");
  assert.equal(formatAspectRatio(1792, 1024), "7:4");
  assert.equal(formatAspectRatio(1024, 1536), "2:3");
});

test("formatAspectRatio returns unavailable for missing or invalid dimensions", () => {
  assert.equal(formatAspectRatio(null, 1024), "Unavailable");
  assert.equal(formatAspectRatio(1024, null), "Unavailable");
  assert.equal(formatAspectRatio(0, 1024), "Unavailable");
  assert.equal(formatAspectRatio(1024, -1), "Unavailable");
});

test("version tree helpers flatten visible nodes and summarize branches", () => {
  const tree = [
    {
      versionId: "root",
      children: [
        {
          versionId: "child-a",
          children: [{ versionId: "grandchild", children: [] }],
        },
        { versionId: "child-b", children: [] },
      ],
    },
  ];

  assert.deepEqual(collectExpandableVersionIds(tree), ["root", "child-a"]);
  assert.deepEqual(
    flattenVisibleVersionTree(tree, new Set(["root"])).map((entry) => ({
      id: entry.node.versionId,
      depth: entry.depth,
      parentId: entry.parentId,
    })),
    [
      { id: "root", depth: 0, parentId: null },
      { id: "child-a", depth: 1, parentId: "root" },
      { id: "child-b", depth: 1, parentId: "root" },
    ],
  );
  assert.deepEqual(
    flattenVisibleVersionTree(tree, new Set(["root", "child-a"])).map((entry) => entry.node.versionId),
    ["root", "child-a", "grandchild", "child-b"],
  );
  assert.equal(formatVersionTreeSummary({ versionCount: 4, versionTreeBranchCount: 2 }), "4 versions / 2 branches");
  assert.equal(formatVersionTreeSummary({ versionCount: 1, versionTreeBranchCount: 0 }), "1 version");
});
