import React, { useEffect, useMemo, useState } from "react";
import ReactDOM from "react-dom/client";
import { convertFileSrc, invoke as tauriInvoke } from "@tauri-apps/api/core";
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
  updateGalleryQuery,
  updateQueueJobStatus,
  type DetailLoadState,
  type GalleryQueryState,
  type GallerySort,
  type JobStatus,
  type ReviewStatusFilter,
} from "./workbench-state";
import "./styles.css";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

type View = "gallery" | "albums" | "review" | "queue" | "settings";

type Library = {
  id: string;
  name: string;
  rootPath: string;
  hidden: boolean;
  schemaVersion: number;
};

type LibraryStatus = {
  storageSizeBytes: number;
  integrityStatus: string;
  integrityIssueCount: number;
};

type GalleryAsset = {
  id: string;
  title: string | null;
  category: string | null;
  rating: number | null;
  status: string;
  provider: string | null;
  modelLabel: string | null;
  prompt: string | null;
  tags: string[];
  reviewPendingCount: number;
  currentVersionId: string | null;
  imagePath: string | null;
  width: number | null;
  height: number | null;
  versionLabel: string | null;
  versionCount: number;
  createdAt: string;
  updatedAt: string;
};

type AssetView = {
  id: string;
  title: string | null;
  category: string | null;
  rating: number | null;
  status: string;
};

type Version = {
  id: string;
  assetId: string;
  parentVersionId: string | null;
  generationEventId: string | null;
  filePath: string;
  sha256: string;
  checksumAlgorithm: string;
  checksum: string;
  mimeType: string;
};

type GenerationEvent = {
  id: string;
  assetId: string | null;
  outputVersionId: string | null;
  provider: string;
  providerModel: string;
  operationType: string;
  prompt: string;
  parametersJson: string;
  status: string;
};

type LineageEntry = {
  version: Version;
  generationEvent: GenerationEvent | null;
};

type Album = {
  id: string;
  name: string;
  kind: "manual" | "smart";
  count?: number;
};

type FileContext = {
  filename: string;
  relativeLocation: string;
  mimeType: string;
  sizeBytes: number | null;
  width: number | null;
  height: number | null;
  checksumAlgorithm: string;
  checksum: string;
  integrityStatus: string;
};

type AssetDetail = {
  id: string;
  title: string | null;
  description: string | null;
  category: string | null;
  rating: number | null;
  status: string;
  createdAt: string;
  updatedAt: string;
  prompt: string | null;
  negativePrompt: string | null;
  provider: string | null;
  modelLabel: string | null;
  parametersJson: string | null;
  tags: string[];
  albums: Album[];
  reviewPendingCount: number;
  versions: Version[];
  lineage: LineageEntry[];
  file: FileContext | null;
};

type Suggestion = {
  id: string;
  assetId: string;
  title: string | null;
  description: string | null;
  tags: string[];
  category: string | null;
  status: string;
};

type QueueJob = {
  id: string;
  prompt: string;
  provider: string;
  status: JobStatus;
  logPath?: string;
  error?: string | null;
  versions?: Version[];
};

type GenerationJob = {
  id: string;
  provider: string;
  prompt: string;
  status: JobStatus;
  logPath: string;
  error: string | null;
  versions: Version[];
};

type CommandError = {
  code?: string;
  message?: string;
  recoverable?: boolean;
};

const mockLibrary: Library = {
  id: "library-local",
  name: "MyImageLab.library",
  rootPath: "/Users/demo/ImagePromptLab",
  hidden: false,
  schemaVersion: 1,
};

const mockLibraries: Library[] = [
  mockLibrary,
  {
    id: "library-reference",
    name: "Reference Sets",
    rootPath: "/Users/demo/ReferenceSets",
    hidden: false,
    schemaVersion: 1,
  },
];

const mockLibraryStatus: LibraryStatus = {
  storageSizeBytes: 153223987,
  integrityStatus: "healthy",
  integrityIssueCount: 0,
};

const mockGallery: GalleryAsset[] = [
  {
    id: "asset-botanical",
    title: "Neon Botanical Study",
    category: "study",
    rating: 5,
    status: "generated",
    provider: "Midjourney",
    modelLabel: "v6.0",
    prompt:
      "botanical study of exotic plants and flowers, neon line art glow, dark background, ultra detailed",
    tags: ["botanical", "neon", "study"],
    reviewPendingCount: 1,
    currentVersionId: "version-botanical-3",
    imagePath: null,
    width: 1024,
    height: 1024,
    versionLabel: "v3",
    versionCount: 3,
    createdAt: "Today, 9:15 AM",
    updatedAt: "Today, 9:15 AM",
  },
  {
    id: "asset-alpine",
    title: "Alpine Reflections",
    category: "landscape",
    rating: 4,
    status: "generated",
    provider: "DALL-E 3",
    modelLabel: "standard",
    prompt: "alpine lake reflection at sunrise, clear air, cinematic landscape study",
    tags: ["landscape", "mountain", "lake"],
    reviewPendingCount: 1,
    currentVersionId: "version-alpine-2",
    imagePath: null,
    width: 1792,
    height: 1024,
    versionLabel: "v2",
    versionCount: 2,
    createdAt: "Today, 9:10 AM",
    updatedAt: "Today, 9:10 AM",
  },
  {
    id: "asset-atrium",
    title: "Solarpunk Atrium",
    category: "interior",
    rating: 5,
    status: "curated",
    provider: "Stable Diffusion XL",
    modelLabel: "sdxl",
    prompt: "solarpunk atrium interior with daylight, plants, and modular architecture",
    tags: ["solarpunk", "interior", "architecture"],
    reviewPendingCount: 0,
    currentVersionId: "version-atrium-4",
    imagePath: null,
    width: 1344,
    height: 1024,
    versionLabel: "v4",
    versionCount: 4,
    createdAt: "Yesterday, 7:45 PM",
    updatedAt: "Yesterday, 7:45 PM",
  },
  {
    id: "asset-canyon",
    title: "Canyon Flow",
    category: "abstract",
    rating: 4,
    status: "generated",
    provider: "Midjourney",
    modelLabel: "v6.0",
    prompt: "abstract canyon flow, layered geology, carved shapes, warm light",
    tags: ["canyon", "abstract", "geology"],
    reviewPendingCount: 0,
    currentVersionId: "version-canyon-1",
    imagePath: null,
    width: null,
    height: null,
    versionLabel: "v1",
    versionCount: 1,
    createdAt: "Yesterday, 5:32 PM",
    updatedAt: "Yesterday, 5:32 PM",
  },
  {
    id: "asset-orbital",
    title: "Orbital Outpost",
    category: "sci-fi",
    rating: 4,
    status: "generated",
    provider: "Stable Diffusion XL",
    modelLabel: "sdxl",
    prompt: "orbital outpost above a blue planet, hard surface sci-fi concept art",
    tags: ["sci-fi", "space", "outpost"],
    reviewPendingCount: 1,
    currentVersionId: "version-orbital-2",
    imagePath: null,
    width: 1024,
    height: 1024,
    versionLabel: "v2",
    versionCount: 2,
    createdAt: "Monday, 2:18 PM",
    updatedAt: "Monday, 2:18 PM",
  },
  {
    id: "asset-tokyo",
    title: "Rainy Tokyo Night",
    category: "city",
    rating: 5,
    status: "curated",
    provider: "DALL-E 3",
    modelLabel: "standard",
    prompt: "rainy tokyo night street, reflections, cinematic city photography",
    tags: ["city", "night", "rain"],
    reviewPendingCount: 0,
    currentVersionId: "version-tokyo-3",
    imagePath: null,
    width: 1024,
    height: 1536,
    versionLabel: "v3",
    versionCount: 3,
    createdAt: "Monday, 1:03 PM",
    updatedAt: "Monday, 1:03 PM",
  },
];

const mockDetail: AssetDetail = {
  id: "asset-botanical",
  title: "Neon Botanical Study",
  description: "Elegant high-contrast botanical illustration.",
  category: "study",
  rating: 5,
  status: "generated",
  createdAt: "Today, 9:15 AM",
  updatedAt: "Today, 9:15 AM",
  prompt:
    "botanical study of exotic plants and flowers, neon line art glow, dark background, ultra detailed, high contrast, vibrant colors, elegant composition, digital illustration",
  negativePrompt: null,
  provider: "Midjourney",
  modelLabel: "v6.0",
  parametersJson: "{\"aspect_ratio\":\"1:1\",\"seed\":2917384512,\"cfg_scale\":7}",
  tags: ["botanical", "neon", "study", "digital art"],
  albums: [
    { id: "album-nature", name: "Nature Studies", kind: "manual" },
    { id: "album-neon", name: "Neon & Glow", kind: "manual" },
  ],
  reviewPendingCount: 1,
  versions: [
    {
      id: "version-botanical-3",
      assetId: "asset-botanical",
      parentVersionId: "version-botanical-2",
      generationEventId: "event-botanical-3",
      filePath: "originals/2026/05/0f18b4ef-8d2d-49bc-a2ef-8d8582386a20.png",
      sha256: "4f72bd81d8c5f1a7f4e4d5e9c4a1b258",
      checksumAlgorithm: "MD5",
      checksum: "4f72bd81d8c5f1a7f4e4d5e9c4a1b258",
      mimeType: "image/png",
    },
    {
      id: "version-botanical-2",
      assetId: "asset-botanical",
      parentVersionId: "version-botanical-1",
      generationEventId: "event-botanical-2",
      filePath: "originals/2026/05/3f2f8444-8cc2-4e35-91a7-806d24213b10.png",
      sha256: "97290b8d8c5f1a7f4e4d5e9c4a1b258",
      checksumAlgorithm: "MD5",
      checksum: "97290b8d8c5f1a7f4e4d5e9c4a1b258",
      mimeType: "image/png",
    },
  ],
  lineage: [
    {
      version: {
        id: "version-botanical-3",
        assetId: "asset-botanical",
        parentVersionId: "version-botanical-2",
        generationEventId: "event-botanical-3",
        filePath: "originals/2026/05/0f18b4ef-8d2d-49bc-a2ef-8d8582386a20.png",
        sha256: "4f72bd81d8c5f1a7f4e4d5e9c4a1b258",
        checksumAlgorithm: "MD5",
        checksum: "4f72bd81d8c5f1a7f4e4d5e9c4a1b258",
        mimeType: "image/png",
      },
      generationEvent: {
        id: "event-botanical-3",
        assetId: "asset-botanical",
        outputVersionId: "version-botanical-3",
        provider: "Midjourney",
        providerModel: "v6.0",
        operationType: "image_to_image",
        prompt: "botanical study of exotic plants and flowers",
        parametersJson: "{\"seed\":2917384512}",
        status: "completed",
      },
    },
  ],
  file: {
    filename: "0f18b4ef-8d2d-49bc-a2ef-8d8582386a20.png",
    relativeLocation: "originals/2026/05/0f18b4ef-8d2d-49bc-a2ef-8d8582386a20.png",
    mimeType: "image/png",
    sizeBytes: 1240000,
    width: 1024,
    height: 1024,
    checksumAlgorithm: "MD5",
    checksum: "4f72bd81d8c5f1a7f4e4d5e9c4a1b258",
    integrityStatus: "verified",
  },
};

const mockSuggestions: Suggestion[] = [
  {
    id: "suggestion-1",
    assetId: "asset-botanical",
    title: "Neon Botanical Study",
    description: "High-contrast botanical line art.",
    tags: ["botanical", "neon", "study"],
    category: "study",
    status: "pending_review",
  },
];

const mockQueue: QueueJob[] = [
  {
    id: "job-1",
    provider: "codex-cli",
    prompt: "Retro UI poster with a glass scanner bed and annotated prompt tokens.",
    status: "queued",
  },
];

function App() {
  const runningInTauri = hasTauriRuntime();
  const [activeView, setActiveView] = useState<View>("gallery");
  const [libraries, setLibraries] = useState<Library[]>(runningInTauri ? [] : mockLibraries);
  const [library, setLibrary] = useState<Library | null>(runningInTauri ? null : mockLibrary);
  const [libraryStatus, setLibraryStatus] = useState<LibraryStatus | null>(
    runningInTauri ? null : mockLibraryStatus,
  );
  const [gallery, setGallery] = useState<GalleryAsset[]>(runningInTauri ? [] : mockGallery);
  const [query, setQuery] = useState<GalleryQueryState>(defaultGalleryQuery);
  const [selectedAssetId, setSelectedAssetId] = useState(runningInTauri ? "" : mockGallery[0].id);
  const [detailState, setDetailState] = useState<DetailLoadState<AssetDetail>>({
    assetId: runningInTauri ? null : mockDetail.id,
    detail: runningInTauri ? null : mockDetail,
    loading: false,
    error: null,
  });
  const [suggestions, setSuggestions] = useState<Suggestion[]>(mockSuggestions);
  const [queue, setQueue] = useState<QueueJob[]>(mockQueue);
  const [prompt, setPrompt] = useState("");
  const [provider, setProvider] = useState("codex-cli");
  const [composerOpen, setComposerOpen] = useState(false);
  const [status, setStatus] = useState(runningInTauri ? "Open or create a library" : "Preview mode");
  const [recoverableError, setRecoverableError] = useState<string | null>(null);
  const [libraryPathInput, setLibraryPathInput] = useState("");
  const [libraryNameInput, setLibraryNameInput] = useState("Image Prompt Lab");

  const displayedGallery = useMemo(
    () => (runningInTauri ? gallery : applyGalleryQuery(mockGallery, query)),
    [runningInTauri, gallery, query],
  );
  const selectedAsset = useMemo(
    () => displayedGallery.find((asset) => asset.id === selectedAssetId) ?? displayedGallery[0] ?? null,
    [displayedGallery, selectedAssetId],
  );
  const pendingSuggestions = suggestions.filter((suggestion) => suggestion.status === "pending_review");
  const availableTags = useMemo(
    () => Array.from(new Set((runningInTauri ? gallery : mockGallery).flatMap((asset) => asset.tags))).sort(),
    [runningInTauri, gallery],
  );

  useEffect(() => {
    if (runningInTauri) {
      void refreshLibraries();
    }
  }, [runningInTauri]);

  useEffect(() => {
    if (runningInTauri && library) {
      void refreshGallery();
    }
  }, [runningInTauri, library?.rootPath, query]);

  useEffect(() => {
    if (!selectedAsset) {
      setDetailState({ assetId: null, detail: null, loading: false, error: null });
      return;
    }
    if (runningInTauri && library) {
      void loadAssetDetail(selectedAsset.id, selectedAsset.currentVersionId);
    } else {
      setDetailState(completeDetailLoad(selectedAsset.id, mockDetailFor(selectedAsset)));
    }
  }, [runningInTauri, library?.rootPath, selectedAsset?.id, selectedAsset?.currentVersionId]);

  async function refreshLibraries() {
    try {
      const libraries = await invokeCommand<Library[]>("list_libraries", { includeHidden: false });
      const nextLibrary = libraries[0] ?? null;
      setLibraries(libraries);
      setLibrary(nextLibrary);
      setLibraryPathInput(nextLibrary?.rootPath ?? libraryPathInput);
      setStatus(nextLibrary ? "Library opened" : "No library registered");
      if (nextLibrary) {
        void refreshLibraryStatus(nextLibrary.rootPath);
      }
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  async function refreshLibraryStatus(rootPath: string) {
    try {
      const nextStatus = await invokeCommand<LibraryStatus>("library_status", { rootPath });
      setLibraryStatus(nextStatus);
    } catch (error) {
      setLibraryStatus(null);
      setRecoverableError(errorMessage(error));
    }
  }

  function switchLibrary(libraryId: string) {
    const nextLibrary = libraries.find((item) => item.id === libraryId) ?? null;
    setLibrary(nextLibrary);
    setSelectedAssetId("");
    setDetailState(clearSelectionForLibrarySwitch());
    setGallery([]);
    setRecoverableError(null);
    setLibraryPathInput(nextLibrary?.rootPath ?? "");
    setStatus(nextLibrary ? "Library switched" : "No library selected");
    if (runningInTauri && nextLibrary) {
      void refreshLibraryStatus(nextLibrary.rootPath);
    } else {
      setLibraryStatus(nextLibrary ? mockLibraryStatus : null);
    }
  }

  async function refreshGallery() {
    if (!library) {
      return;
    }
    try {
      const items = await invokeCommand<GalleryAsset[]>("query_gallery", {
        input: galleryQueryInput(library.rootPath, query),
      });
      setGallery(items);
      setSelectedAssetId((current) => items.find((item) => item.id === current)?.id ?? items[0]?.id ?? "");
      setStatus(`${items.length} item${items.length === 1 ? "" : "s"}`);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function loadAssetDetail(assetId: string, versionId: string | null) {
    if (!library) {
      return;
    }
    setDetailState(beginDetailLoad(assetId));
    try {
      const detail = await invokeCommand<AssetDetail>("get_asset_detail", {
        input: {
          libraryPath: library.rootPath,
          assetId,
          currentVersionId: versionId,
        },
      });
      setDetailState(completeDetailLoad(assetId, detail));
    } catch (error) {
      setDetailState(failDetailLoad(assetId, errorMessage(error)));
    }
  }

  async function createLibrary() {
    if (libraryPathInput.trim().length === 0) {
      setStatus("Library path is required");
      return;
    }
    try {
      const created = await invokeCommand<Library>("create_library", {
        input: {
          rootPath: libraryPathInput,
          name: libraryNameInput.trim() || "Image Prompt Lab",
        },
      });
      setLibraries((current) => [created, ...current.filter((item) => item.id !== created.id)]);
      setLibrary(created);
      setLibraryStatus(null);
      setGallery([]);
      setSelectedAssetId("");
      setStatus("Library created");
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  async function openLibrary() {
    if (libraryPathInput.trim().length === 0) {
      setStatus("Library path is required");
      return;
    }
    try {
      const opened = await invokeCommand<Library>("open_library", {
        rootPath: libraryPathInput,
      });
      setLibraries((current) => [opened, ...current.filter((item) => item.id !== opened.id)]);
      setLibrary(opened);
      setLibraryStatus(null);
      setStatus("Library opened");
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  async function startGeneration(inputVersionId: string | null = null) {
    if (!library || prompt.trim().length === 0) {
      setRecoverableError("Open a real library and enter a prompt before generation.");
      return;
    }
    const optimisticJob: QueueJob = {
      id: crypto.randomUUID(),
      provider,
      prompt,
      status: "running",
    };
    setQueue((current) => [optimisticJob, ...current]);
    setStatus("Starting generation");
    try {
      const started = await invokeCommand<GenerationJob>("start_generation", {
        input: {
          libraryPath: library.rootPath,
          provider,
          prompt,
          negativePrompt: null,
          inputFile: null,
          inputVersionId,
          parametersJson: "{}",
        },
      });
      setQueue((current) =>
        current.map((item) =>
          item.id === optimisticJob.id
            ? {
                ...item,
                id: started.id,
                status: started.status,
                logPath: started.logPath,
              }
            : item,
        ),
      );
      setPrompt("");
      setComposerOpen(false);
      setRecoverableError(null);
      void pollGenerationJob(started.id);
    } catch (error) {
      setQueue((current) => updateQueueJobStatus(current, optimisticJob.id, "failed"));
      setRecoverableError(errorMessage(error));
    }
  }

  async function pollGenerationJob(jobId: string) {
    try {
      const job = await invokeCommand<GenerationJob>("get_generation_job", { jobId });
      setQueue((current) =>
        current.map((item) =>
          item.id === job.id
            ? {
                ...item,
                status: job.status,
                logPath: job.logPath,
                error: job.error,
                versions: job.versions,
              }
            : item,
        ),
      );
      if (job.status === "completed") {
        setStatus(`${job.versions.length} version generated`);
        await refreshGallery();
        return;
      }
      if (job.status === "failed") {
        setRecoverableError(job.error ?? "Generation failed");
        return;
      }
      window.setTimeout(() => {
        void pollGenerationJob(jobId);
      }, 1200);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function updateRating(rating: number) {
    const detail = detailState.detail;
    if (!library || !detail) {
      return;
    }
    try {
      const asset = await invokeCommand<AssetView>("update_asset_metadata", {
        input: {
          libraryPath: library.rootPath,
          assetId: detail.id,
          rating,
          category: detail.category,
          status: detail.status,
        },
      });
      setGallery((current) =>
        current.map((item) => (item.id === asset.id ? { ...item, rating: asset.rating } : item)),
      );
      setDetailState((current) =>
        current.detail ? { ...current, detail: { ...current.detail, rating: asset.rating } } : current,
      );
      setStatus("Rating updated");
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function updateTitle(title: string) {
    const detail = detailState.detail;
    const trimmed = title.trim();
    if (!library || !detail || trimmed.length === 0 || trimmed === detail.title) {
      return;
    }
    try {
      const asset = await invokeCommand<AssetView>("update_asset_metadata", {
        input: {
          libraryPath: library.rootPath,
          assetId: detail.id,
          title: trimmed,
        },
      });
      setGallery((current) =>
        current.map((item) => (item.id === asset.id ? { ...item, title: asset.title } : item)),
      );
      setDetailState((current) =>
        current.detail ? { ...current, detail: { ...current.detail, title: asset.title } } : current,
      );
      setStatus("Title updated");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function addTagToSelectedAsset(tag: string) {
    const detail = detailState.detail;
    const trimmed = tag.trim();
    if (!library || !detail || trimmed.length === 0) {
      return;
    }
    try {
      await invokeCommand("add_tag_to_asset", {
        input: {
          libraryPath: library.rootPath,
          assetId: detail.id,
          tag: trimmed,
        },
      });
      await refreshGallery();
      await loadAssetDetail(detail.id, selectedAsset?.currentVersionId ?? null);
      setStatus("Tag added");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function acceptSuggestion(suggestion: Suggestion) {
    if (!library) {
      return;
    }
    try {
      const asset = await invokeCommand<AssetView>("accept_suggestion", {
        input: {
          libraryPath: library.rootPath,
          suggestionId: suggestion.id,
          title: suggestion.title,
          description: suggestion.description,
          tags: suggestion.tags,
          category: suggestion.category,
        },
      });
      setGallery((current) => {
        const state = acceptSuggestionState(current, suggestions, {
          id: suggestion.id,
          assetId: asset.id,
          title: asset.title,
          description: suggestion.description,
          category: asset.category,
          tags: suggestion.tags,
          status: suggestion.status,
        });
        return state.assets;
      });
      setSuggestions((current) => rejectSuggestionState(current, suggestion.id));
      await refreshGallery();
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function rejectSuggestion(suggestion: Suggestion) {
    if (!library) {
      setSuggestions((current) => rejectSuggestionState(current, suggestion.id));
      return;
    }
    try {
      await invokeCommand("reject_suggestion", {
        libraryPath: library.rootPath,
        suggestionId: suggestion.id,
      });
      setSuggestions((current) => rejectSuggestionState(current, suggestion.id));
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  const detail = detailState.detail;

  return (
    <main className="workbench">
      <Sidebar
        library={library}
        libraries={libraries}
        libraryStatus={libraryStatus}
        activeView={activeView}
        reviewCount={pendingSuggestions.length}
        queueCount={queue.filter((job) => job.status === "queued" || job.status === "running").length}
        onViewChange={setActiveView}
        onLibraryChange={switchLibrary}
      />

      <section className="workspace">
        <WorkspaceToolbar
          activeView={activeView}
          query={query}
          itemCount={displayedGallery.length}
          status={status}
          composerOpen={composerOpen}
          onComposerOpenChange={setComposerOpen}
          onQueryChange={setQuery}
        />

        {composerOpen && (
          <GenerationComposer
            prompt={prompt}
            provider={provider}
            onPromptChange={setPrompt}
            onProviderChange={setProvider}
            onGenerate={() => void startGeneration(null)}
          />
        )}

        {recoverableError && (
          <div className="inline-error">
            <span>{recoverableError}</span>
            <button onClick={() => setRecoverableError(null)}>Dismiss</button>
          </div>
        )}

        {activeView === "gallery" && (
          <GalleryView
            assets={displayedGallery}
            selectedAssetId={selectedAsset?.id ?? ""}
            query={query}
            availableTags={availableTags}
            onSelect={setSelectedAssetId}
            onQueryChange={setQuery}
          />
        )}
        {activeView === "albums" && <AlbumsView gallery={displayedGallery} />}
        {activeView === "review" && (
          <ReviewInbox suggestions={pendingSuggestions} onAccept={acceptSuggestion} onReject={rejectSuggestion} />
        )}
        {activeView === "queue" && <GenerationQueue queue={queue} />}
        {activeView === "settings" && (
          <SettingsView
            library={library}
            libraries={libraries}
            libraryPath={libraryPathInput}
            libraryName={libraryNameInput}
            onLibraryPathChange={setLibraryPathInput}
            onLibraryNameChange={setLibraryNameInput}
            onCreate={createLibrary}
            onOpen={openLibrary}
          />
        )}
      </section>

      <Inspector
        asset={selectedAsset}
        detailState={detailState}
        onUpdateRating={updateRating}
        onUpdateTitle={(title) => void updateTitle(title)}
        onAddTag={(tag) => void addTagToSelectedAsset(tag)}
        onGenerateVariation={() => {
          const versionId = detail?.lineage[0]?.version.id ?? detail?.versions[0]?.id ?? selectedAsset?.currentVersionId ?? null;
          void startGeneration(versionId);
        }}
      />
    </main>
  );
}

function Sidebar({
  library,
  libraries,
  libraryStatus,
  activeView,
  reviewCount,
  queueCount,
  onViewChange,
  onLibraryChange,
}: {
  library: Library | null;
  libraries: Library[];
  libraryStatus: LibraryStatus | null;
  activeView: View;
  reviewCount: number;
  queueCount: number;
  onViewChange: (view: View) => void;
  onLibraryChange: (libraryId: string) => void;
}) {
  return (
    <aside className="sidebar">
      <label className="library-card library-selector-card">
        <span className="database-icon">DB</span>
        <span>
          <strong>{library?.name ?? "No library"}</strong>
          <small>Library</small>
        </span>
        <select
          className="library-picker"
          aria-label="Switch library"
          value={library?.id ?? ""}
          onChange={(event) => onLibraryChange(event.target.value)}
        >
          {libraries.length === 0 ? (
            <option value="">No library registered</option>
          ) : (
            libraries.map((item) => (
              <option key={item.id} value={item.id}>
                {item.name}
              </option>
            ))
          )}
        </select>
        <span className="library-chevron" aria-hidden="true" />
      </label>
      <nav className="nav">
        <NavButton active={activeView === "gallery"} label="Gallery" onClick={() => onViewChange("gallery")} />
        <NavButton active={activeView === "albums"} label="Albums" onClick={() => onViewChange("albums")} />
        <NavButton
          active={activeView === "review"}
          label="Review Inbox"
          count={reviewCount}
          onClick={() => onViewChange("review")}
        />
        <NavButton
          active={activeView === "queue"}
          label="Generation Queue"
          count={queueCount}
          onClick={() => onViewChange("queue")}
        />
        <NavButton active={activeView === "settings"} label="Settings" onClick={() => onViewChange("settings")} />
      </nav>
      <section className="library-status">
        <div>
          <span>Library Status</span>
          <strong className="healthy">Healthy</strong>
        </div>
        <div>
          <span>Storage</span>
          <span>{formatBytes(libraryStatus?.storageSizeBytes ?? null)}</span>
        </div>
        <div>
          <span>Integrity Check</span>
          <strong className={libraryStatus?.integrityIssueCount ? "warning" : "healthy"}>
            {libraryStatus?.integrityIssueCount ? `${libraryStatus.integrityIssueCount} issue(s)` : "All good"}
          </strong>
        </div>
        <button>Run Integrity Check</button>
      </section>
      <small className="app-version">Image Prompt Lab 1.2.0</small>
    </aside>
  );
}

function NavButton({
  active,
  label,
  count,
  onClick,
}: {
  active: boolean;
  label: string;
  count?: number;
  onClick: () => void;
}) {
  return (
    <button className={active ? "nav-button active" : "nav-button"} onClick={onClick}>
      <span>{label}</span>
      {typeof count === "number" && count > 0 && <strong>{count}</strong>}
    </button>
  );
}

function WorkspaceToolbar({
  activeView,
  query,
  itemCount,
  status,
  composerOpen,
  onComposerOpenChange,
  onQueryChange,
}: {
  activeView: View;
  query: GalleryQueryState;
  itemCount: number;
  status: string;
  composerOpen: boolean;
  onComposerOpenChange: (open: boolean) => void;
  onQueryChange: (query: GalleryQueryState) => void;
}) {
  return (
    <header className="workspace-toolbar">
      <div className="search-row">
        <label className="search-box">
          <span>Search</span>
          <input
            value={query.text}
            onChange={(event) => onQueryChange(updateGalleryQuery(query, { text: event.target.value }))}
            placeholder="Search prompts, titles, tags, albums..."
          />
        </label>
        <button className="primary-button" onClick={() => onComposerOpenChange(!composerOpen)}>
          Generate
        </button>
        <button className="icon-button" aria-label="Grid view">
          #
        </button>
        <button className="icon-button" aria-label="List view">
          =
        </button>
      </div>
      <div className="filter-row">
        <SegmentedButton
          label="Provider"
          active={query.providers.length > 0}
          onClick={() => onQueryChange(toggleGalleryProvider(query, "fake"))}
        />
        <select
          className="select-control"
          value={query.minRating ?? ""}
          onChange={(event) =>
            onQueryChange(
              updateGalleryQuery(query, {
                minRating: event.target.value ? Number(event.target.value) : null,
              }),
            )
          }
        >
          <option value="">Rating</option>
          <option value="5">5 stars</option>
          <option value="4">4+ stars</option>
          <option value="3">3+ stars</option>
        </select>
        <select
          className="select-control"
          value={query.reviewStatus}
          onChange={(event) =>
            onQueryChange(updateGalleryQuery(query, { reviewStatus: event.target.value as ReviewStatusFilter }))
          }
        >
          <option value="any">Review</option>
          <option value="pending">Review Pending</option>
        </select>
        <button onClick={() => onQueryChange(resetGalleryQuery())}>Clear All</button>
        <span className="toolbar-status">{status}</span>
        {activeView === "gallery" && (
          <>
            <select
              className="select-control sort-select"
              value={query.sort}
              onChange={(event) => onQueryChange(updateGalleryQuery(query, { sort: event.target.value as GallerySort }))}
            >
              <option value="newest">Sort: Newest</option>
              <option value="oldest">Sort: Oldest</option>
              <option value="ratingDesc">Sort: Rating</option>
              <option value="titleAsc">Sort: Title</option>
              <option value="providerAsc">Sort: Provider</option>
            </select>
            <strong>{itemCount} items</strong>
          </>
        )}
      </div>
    </header>
  );
}

function SegmentedButton({
  label,
  active,
  onClick,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button className={active ? "chip-button active" : "chip-button"} onClick={onClick}>
      {label}
    </button>
  );
}

function GenerationComposer({
  prompt,
  provider,
  onPromptChange,
  onProviderChange,
  onGenerate,
}: {
  prompt: string;
  provider: string;
  onPromptChange: (value: string) => void;
  onProviderChange: (value: string) => void;
  onGenerate: () => void;
}) {
  return (
    <section className="composer">
      <select className="select-control" value={provider} onChange={(event) => onProviderChange(event.target.value)}>
        <option value="codex-cli">codex-cli</option>
        <option value="fake">fake</option>
      </select>
      <input value={prompt} onChange={(event) => onPromptChange(event.target.value)} placeholder="Prompt" />
      <button disabled={prompt.trim().length === 0} onClick={onGenerate}>
        Run
      </button>
    </section>
  );
}

function GalleryView({
  assets,
  selectedAssetId,
  query,
  availableTags,
  onSelect,
  onQueryChange,
}: {
  assets: GalleryAsset[];
  selectedAssetId: string;
  query: GalleryQueryState;
  availableTags: string[];
  onSelect: (id: string) => void;
  onQueryChange: (query: GalleryQueryState) => void;
}) {
  if (assets.length === 0) {
    return <div className="empty-state">No assets match the current query.</div>;
  }
  return (
    <>
      <div className="tag-filter-strip">
        {availableTags.slice(0, 8).map((tag) => (
          <button
            key={tag}
            className={query.tags.includes(tag) ? "tag-chip selected" : "tag-chip"}
            onClick={() => onQueryChange(toggleGalleryTag(query, tag))}
          >
            {tag}
          </button>
        ))}
      </div>
      <section className="gallery-grid">
        {assets.map((asset, index) => (
          <button
            key={asset.id}
            className={asset.id === selectedAssetId ? "asset-card selected" : "asset-card"}
            onClick={() => onSelect(asset.id)}
          >
            <Thumbnail asset={asset} index={index} />
            <span className="asset-title">{asset.title ?? "Untitled"}</span>
            <span className="provider-pill">{asset.provider ?? "Unknown provider"}</span>
            <StarRatingDisplay rating={asset.rating} />
            {asset.reviewPendingCount > 0 && <span className="review-badge">Review pending</span>}
            <span className="card-tags">
              {asset.tags.slice(0, 3).map((tag) => (
                <span key={tag}>{tag}</span>
              ))}
            </span>
            <span className="version-line">
              <span>{asset.versionLabel ?? "v1"}</span>
              <span>{asset.versionCount} version{asset.versionCount === 1 ? "" : "s"}</span>
            </span>
          </button>
        ))}
      </section>
    </>
  );
}

function Thumbnail({ asset, index }: { asset: GalleryAsset; index: number }) {
  const style = thumbnailStyle(index);
  return (
    <span className="thumbnail" style={style}>
      {asset.imagePath && <img alt={asset.title ?? "Generated image"} src={convertImagePath(asset.imagePath)} />}
    </span>
  );
}

function AlbumsView({ gallery }: { gallery: GalleryAsset[] }) {
  const groups = groupByProvider(gallery);
  return (
    <section className="list-view">
      {groups.map((group) => (
        <article className="list-row" key={group.name}>
          <div>
            <h3>{group.name}</h3>
            <p>manual</p>
          </div>
          <strong>{group.count}</strong>
        </article>
      ))}
    </section>
  );
}

function ReviewInbox({
  suggestions,
  onAccept,
  onReject,
}: {
  suggestions: Suggestion[];
  onAccept: (suggestion: Suggestion) => void;
  onReject: (suggestion: Suggestion) => void;
}) {
  if (suggestions.length === 0) {
    return <div className="empty-state">No pending suggestions.</div>;
  }
  return (
    <section className="list-view">
      {suggestions.map((suggestion) => (
        <article className="list-row review-row" key={suggestion.id}>
          <div>
            <h3>{suggestion.title ?? "Untitled suggestion"}</h3>
            <p>{suggestion.tags.join(", ") || "No tags"}</p>
          </div>
          <div className="row-actions">
            <button onClick={() => onAccept(suggestion)}>Accept</button>
            <button onClick={() => onReject(suggestion)}>Reject</button>
          </div>
        </article>
      ))}
    </section>
  );
}

function GenerationQueue({ queue }: { queue: QueueJob[] }) {
  return (
    <section className="list-view">
      {queue.map((job) => (
        <article className="list-row" key={job.id}>
          <div>
            <h3>{job.provider}</h3>
            <p>{job.prompt}</p>
            {job.logPath && <p className="mono-line">{job.logPath}</p>}
            {job.error && <p className="error-text">{job.error}</p>}
          </div>
          <span className={`status ${job.status}`}>{job.status}</span>
        </article>
      ))}
    </section>
  );
}

function SettingsView({
  library,
  libraries,
  libraryPath,
  libraryName,
  onLibraryPathChange,
  onLibraryNameChange,
  onCreate,
  onOpen,
}: {
  library: Library | null;
  libraries: Library[];
  libraryPath: string;
  libraryName: string;
  onLibraryPathChange: (value: string) => void;
  onLibraryNameChange: (value: string) => void;
  onCreate: () => void;
  onOpen: () => void;
}) {
  return (
    <section className="settings-grid">
      <div>
        <h3>Library</h3>
        <p>{library?.rootPath ?? "Not opened"}</p>
      </div>
      <div>
        <h3>Registered Libraries</h3>
        <p>{libraries.length}</p>
      </div>
      <div>
        <h3>Schema</h3>
        <p>{library?.schemaVersion ?? "-"}</p>
      </div>
      <div className="library-form">
        <label>
          <span>Library path</span>
          <input value={libraryPath} onChange={(event) => onLibraryPathChange(event.target.value)} />
        </label>
        <label>
          <span>Library name</span>
          <input value={libraryName} onChange={(event) => onLibraryNameChange(event.target.value)} />
        </label>
        <div className="row-actions">
          <button onClick={onCreate}>Create Library</button>
          <button onClick={onOpen}>Open Library</button>
        </div>
      </div>
    </section>
  );
}

function Inspector({
  asset,
  detailState,
  onUpdateRating,
  onUpdateTitle,
  onAddTag,
  onGenerateVariation,
}: {
  asset: GalleryAsset | null;
  detailState: DetailLoadState<AssetDetail>;
  onUpdateRating: (rating: number) => void;
  onUpdateTitle: (title: string) => void;
  onAddTag: (tag: string) => void;
  onGenerateVariation: () => void;
}) {
  const detail = detailState.detail;
  const [tagInput, setTagInput] = useState("");
  const [tagEditorOpen, setTagEditorOpen] = useState(false);
  const [titleEditing, setTitleEditing] = useState(false);
  const [titleInput, setTitleInput] = useState("");
  useEffect(() => {
    setTagInput("");
    setTagEditorOpen(false);
    setTitleEditing(false);
    setTitleInput(detail?.title ?? asset?.title ?? "");
  }, [detail?.id, detail?.title, asset?.title]);
  const submitTag = () => {
    const trimmed = tagInput.trim();
    if (trimmed.length === 0) {
      return;
    }
    onAddTag(trimmed);
    setTagInput("");
    setTagEditorOpen(false);
  };
  const saveTitle = () => {
    const trimmed = titleInput.trim();
    setTitleEditing(false);
    if (trimmed.length > 0) {
      onUpdateTitle(trimmed);
    }
  };
  if (!asset) {
    return (
      <aside className="inspector">
        <h2>Inspector</h2>
        <div className="empty-state compact">No asset selected.</div>
      </aside>
    );
  }
  if (detailState.loading) {
    return (
      <aside className="inspector">
        <h2>Inspector</h2>
        <div className="empty-state compact">Loading asset detail...</div>
      </aside>
    );
  }
  if (detailState.error || !detail) {
    return (
      <aside className="inspector">
        <h2>Inspector</h2>
        <div className="empty-state compact">{detailState.error ?? "Detail unavailable."}</div>
      </aside>
    );
  }
  return (
    <aside className="inspector">
      <section className="inspector-hero">
        <Thumbnail asset={asset} index={0} />
        <div>
          {titleEditing ? (
            <input
              className="title-input"
              value={titleInput}
              autoFocus
              onChange={(event) => setTitleInput(event.target.value)}
              onBlur={saveTitle}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  saveTitle();
                }
                if (event.key === "Escape") {
                  setTitleInput(detail.title ?? asset.title ?? "");
                  setTitleEditing(false);
                }
              }}
            />
          ) : (
            <h2 className="editable-title" onDoubleClick={() => setTitleEditing(true)}>
              {detail.title ?? asset.title ?? "Untitled"}
            </h2>
          )}
          <StarRatingDisplay rating={detail.rating} showEmpty />
          {detail.reviewPendingCount > 0 && <strong>Review pending</strong>}
          <small>Added: {displayDate(detail.createdAt)}</small>
        </div>
      </section>
      <InspectorSection title="Prompt">
        <p>{detail.prompt ?? "Prompt is unavailable for this version."}</p>
        <button className="text-button">Show full prompt</button>
      </InspectorSection>
      <InspectorSection title="Rating">
        <StarRatingControl rating={detail.rating} onChange={onUpdateRating} />
      </InspectorSection>
      <InspectorSection title="Provider & Model">
        <MetaRow label="Provider" value={detail.provider ?? asset.provider ?? "-"} />
        <MetaRow label="Model" value={detail.modelLabel ?? asset.modelLabel ?? "-"} />
        <MetaRow label="Parameters" value={detail.parametersJson ?? "-"} />
      </InspectorSection>
      <InspectorSection title="Tags">
        <div className="tag-list">
          {detail.tags.map((tag) => (
            <span key={tag}>{tag}</span>
          ))}
          {tagEditorOpen && (
            <input
              className="tag-input"
              value={tagInput}
              autoFocus
              onChange={(event) => setTagInput(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  submitTag();
                }
                if (event.key === "Escape") {
                  setTagInput("");
                  setTagEditorOpen(false);
                }
              }}
              placeholder="Add tag"
            />
          )}
          <button
            className="mini-button"
            disabled={tagEditorOpen && tagInput.trim().length === 0}
            onClick={() => {
              if (tagEditorOpen) {
                submitTag();
              } else {
                setTagEditorOpen(true);
              }
            }}
          >
            +
          </button>
        </div>
      </InspectorSection>
      <InspectorSection title="Albums">
        {detail.albums.length === 0 ? (
          <p>No albums yet.</p>
        ) : (
          detail.albums.map((album) => <MetaRow key={album.id} label={album.kind} value={album.name} />)
        )}
        <button className="text-button">Add to album</button>
      </InspectorSection>
      <InspectorSection title="Versions & Lineage">
        <div className="lineage-list">
          {detail.lineage.length === 0 ? (
            <p>No lineage available.</p>
          ) : (
            detail.lineage.map((entry, index) => (
              <div key={entry.version.id}>
                <strong>{index === 0 ? "Current" : "Parent"}</strong>
                <span>{entry.version.id}</span>
              </div>
            ))
          )}
        </div>
        <button onClick={onGenerateVariation}>Generate variation</button>
      </InspectorSection>
      <InspectorSection title="File">
        {detail.file ? (
          <>
            <MetaRow label="Filename" value={detail.file.filename} />
            <MetaRow label="Location" value={detail.file.relativeLocation} />
            <MetaRow label="Size" value={formatBytes(detail.file.sizeBytes)} />
            <MetaRow label="Dimensions" value={formatDimensions(detail.file)} />
            <MetaRow label="Aspect Ratio" value={formatAspectRatio(detail.file.width, detail.file.height)} />
            <MetaRow label="Integrity" value={detail.file.integrityStatus} />
            <MetaRow label="Checksum" value={formatChecksum(detail.file)} />
          </>
        ) : (
          <p>File context is unavailable.</p>
        )}
      </InspectorSection>
    </aside>
  );
}

function InspectorSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="inspector-section">
      <h3>{title}</h3>
      {children}
    </section>
  );
}

function MetaRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="meta-row">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function galleryQueryInput(libraryPath: string, query: GalleryQueryState) {
  return {
    libraryPath,
    text: query.text.trim() || null,
    providers: query.providers,
    minRating: query.minRating,
    reviewStatus: query.reviewStatus,
    tags: query.tags,
    sort: query.sort,
    albumId: null,
  };
}

function errorMessage(error: unknown) {
  if (typeof error === "object" && error) {
    const commandError = error as CommandError;
    if (commandError.message) {
      return commandError.message;
    }
  }
  return String(error);
}

function convertImagePath(path: string) {
  if (!hasTauriRuntime()) {
    return path;
  }
  return convertFileSrc(path);
}

async function invokeCommand<T>(command: string, args?: Record<string, unknown>) {
  if (!hasTauriRuntime()) {
    throw new Error("This action requires the Tauri desktop runtime. Start with npm run tauri dev.");
  }
  return tauriInvoke<T>(command, args);
}

function hasTauriRuntime() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

function mockDetailFor(asset: GalleryAsset): AssetDetail {
  return {
    ...mockDetail,
    id: asset.id,
    title: asset.title,
    category: asset.category,
    rating: asset.rating,
    status: asset.status,
    provider: asset.provider,
    modelLabel: asset.modelLabel,
    tags: asset.tags,
    reviewPendingCount: asset.reviewPendingCount,
    file: mockDetail.file
      ? {
          ...mockDetail.file,
          width: asset.width,
          height: asset.height,
        }
      : null,
  };
}

function thumbnailStyle(index: number): React.CSSProperties {
  const styles = [
    "linear-gradient(135deg, #0b0b0b, #052f34 45%, #d98b00)",
    "linear-gradient(135deg, #26465a, #f18f6f 50%, #ffd2a6)",
    "linear-gradient(135deg, #d8d6cd, #f7f3ea 45%, #57756b)",
    "linear-gradient(135deg, #8f2717, #ef7847 45%, #4c153d)",
    "linear-gradient(135deg, #121820, #586878 45%, #c7a36f)",
    "linear-gradient(135deg, #081b2d, #184e63 45%, #e07f3b)",
  ];
  return { background: styles[index % styles.length] };
}

function StarRatingDisplay({ rating, showEmpty = false }: { rating: number | null; showEmpty?: boolean }) {
  const value = rating ?? 0;
  if (value === 0 && !showEmpty) {
    return <span className="star-rating empty" aria-label="Not rated">Unrated</span>;
  }
  return (
    <span className="star-rating" aria-label={`${value} of 5 stars`}>
      {[1, 2, 3, 4, 5].map((star) => (
        <span key={star} className={star <= value ? "star filled" : "star"}>
          {star <= value ? "★" : "☆"}
        </span>
      ))}
    </span>
  );
}

function StarRatingControl({
  rating,
  onChange,
}: {
  rating: number | null;
  onChange: (rating: number) => void;
}) {
  const value = rating ?? 0;
  return (
    <div className="star-rating-control" role="radiogroup" aria-label="Rating">
      {[1, 2, 3, 4, 5].map((star) => (
        <button
          key={star}
          className={star <= value ? "star-button active" : "star-button"}
          aria-label={`${star} star${star === 1 ? "" : "s"}`}
          aria-checked={value === star}
          role="radio"
          onClick={() => onChange(star)}
        >
          {star <= value ? "★" : "☆"}
        </button>
      ))}
    </div>
  );
}

function displayDate(value: string) {
  if (/^\d+$/.test(value)) {
    return new Date(Number(value)).toLocaleString();
  }
  return value;
}

function formatBytes(value: number | null) {
  if (!value) {
    return "-";
  }
  if (value >= 1024 * 1024 * 1024) {
    return `${(value / 1024 / 1024 / 1024).toFixed(1)} GB`;
  }
  if (value > 1024 * 1024) {
    return `${(value / 1024 / 1024).toFixed(1)} MB`;
  }
  return `${Math.round(value / 1024)} KB`;
}

function formatDimensions(file: FileContext) {
  return formatResolution(file.width, file.height);
}

function formatResolution(width: number | null, height: number | null) {
  if (!width || !height) {
    return "Unavailable";
  }
  return `${width} x ${height}`;
}

function formatChecksum(file: FileContext) {
  return `${file.checksumAlgorithm}: ${file.checksum}`;
}

function groupByProvider(gallery: GalleryAsset[]) {
  const counts = new Map<string, number>();
  for (const asset of gallery) {
    const key = asset.provider ?? "Unknown provider";
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  return Array.from(counts.entries()).map(([name, count]) => ({ name, count }));
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
