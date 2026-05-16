import assert from "node:assert/strict";
import test from "node:test";
import {
  acceptSuggestionState,
  rejectSuggestionState,
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
