import assert from "node:assert/strict";
import test from "node:test";
import {
  acceptSuggestionState,
  beginDetailLoad,
  completeDetailLoad,
  defaultGalleryQuery,
  failDetailLoad,
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
