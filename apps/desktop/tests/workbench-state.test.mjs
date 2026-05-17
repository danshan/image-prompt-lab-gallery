import assert from "node:assert/strict";
import test from "node:test";
import {
  acceptSuggestionState,
  applyGalleryQuery,
  beginDetailLoad,
  clearSelectionForLibrarySwitch,
  completeDetailLoad,
  defaultGalleryQuery,
  failDetailLoad,
  formatAspectRatio,
  rejectSuggestionState,
  resetGalleryQuery,
  toggleGalleryProvider,
  toggleGalleryTag,
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

test("rejectSuggestionState removes only the rejected suggestion", () => {
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

  const next = rejectSuggestionState(suggestions, "suggestion-1");

  assert.deepEqual(
    next.map((suggestion) => suggestion.id),
    ["suggestion-2"],
  );
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
