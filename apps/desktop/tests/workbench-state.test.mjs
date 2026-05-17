import assert from "node:assert/strict";
import test from "node:test";
import {
  acceptSuggestionState,
  applySuggestionFieldToReviewForm,
  applyGalleryQuery,
  beginDetailLoad,
  beginReviewFieldGeneration,
  buildBatchReviewPayloads,
  clearAlbumQuery,
  clearCurationStateForLibrarySwitch,
  clearSelectionForLibrarySwitch,
  completeDetailLoad,
  completeReviewFieldGeneration,
  createReviewFormState,
  defaultGalleryQuery,
  failReviewFieldGeneration,
  failDetailLoad,
  formatAspectRatio,
  addReviewFormTag,
  isReviewFieldGenerating,
  markAssetReviewPending,
  moveItem,
  openAlbumQuery,
  removeReviewFormTag,
  removeSuggestionState,
  reorderByIds,
  resetGalleryQuery,
  reviewFormTags,
  selectedOrCurrentIds,
  toggleGalleryProvider,
  toggleGalleryTag,
  toggleSelection,
  updateQueueJobStatus,
} from "../.test-dist/workbench-state.js";

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

test("updateQueueJobStatus updates only the targeted job", () => {
  const queue = [
    { id: "job-1", status: "running" },
    { id: "job-2", status: "queued" },
  ];

  const next = updateQueueJobStatus(queue, "job-1", "completed");

  assert.equal(next[0].status, "completed");
  assert.equal(next[1].status, "queued");
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

test("album query helpers set and clear selected album", () => {
  const albumQuery = openAlbumQuery(defaultGalleryQuery, "album-1");
  assert.equal(albumQuery.albumId, "album-1");
  assert.equal(clearAlbumQuery(albumQuery).albumId, null);
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
