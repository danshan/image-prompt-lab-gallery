import React, { useEffect, useMemo, useRef, useState } from "react";
import ReactDOM from "react-dom/client";
import { convertFileSrc, invoke as tauriInvoke } from "@tauri-apps/api/core";
import {
  acceptSuggestionState,
  addReviewFormTag,
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
  failDetailLoad,
  failReviewFieldGeneration,
  formatAspectRatio,
  isReviewFieldGenerating,
  markAssetReviewPending,
  moveItem,
  openAlbumQuery,
  removeSuggestionState,
  removeReviewFormTag,
  reorderByIds,
  resetGalleryQuery,
  reviewFormTags,
  selectedOrCurrentIds,
  toggleGalleryProvider,
  toggleGalleryTag,
  toggleSelection,
  updateGalleryQuery,
  updateQueueJobStatus,
  type DetailLoadState,
  type GalleryQueryState,
  type GallerySort,
  type JobStatus,
  type ReviewFieldName,
  type ReviewFormState,
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

type AlbumListItem = {
  id: string;
  name: string;
  kind: "manual" | "smart";
  itemCount: number | null;
  sortOrder?: number;
};

type ConfidenceScore = {
  overall: number | null;
  title: number | null;
  description: number | null;
  schemaPrompt: number | null;
  tags: number | null;
  category: number | null;
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
  schemaPrompt: string | null;
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
  schemaPrompt: string | null;
  tags: string[];
  category: string | null;
  status: string;
  confidenceJson?: string;
  createdAt?: string | null;
  reviewedAt?: string | null;
  confidence?: ConfidenceScore;
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

type GeneratedReviewField = {
  field: ReviewFieldName;
  value: string;
  logPath: string;
};

type AppLog = {
  path: string;
  kind: string;
  modifiedAt: string;
  sizeBytes: number;
  preview: string;
};

type AppLogContent = {
  path: string;
  content: string;
  truncated: boolean;
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

const mockSchemaPrompt = `// VERSION: 0.1
// AESTHETIC: botanical neon study
{
  "GLOBAL_SETTINGS": {
    "aspect_ratio": "1:1 square",
    "style": "high-contrast digital illustration",
    "clarity": "sharp foreground, luminous line detail",
    "render_flags": ["sharp_foreground", "micro_texture", "editorial_finish"]
  },
  "ENVIRONMENT": {
    "background": "dark studio backdrop",
    "lighting": "neon rim light with controlled glow",
    "atmosphere": ["botanical glow", "clean negative space"]
  },
  "CORE_ASSETS": {
    "primary_subject": "exotic botanical study",
    "materials": ["luminous line art", "delicate petal texture"],
    "composition": "centered specimen layout"
  },
  "OUTPUT": {
    "mood": "elegant, vivid, editorial",
    "avoid": ["muddy contrast", "generic flower clipart", "fake brand logos"]
  }
}`;

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
  schemaPrompt: mockSchemaPrompt,
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
      checksumAlgorithm: "SHA-256",
      checksum: "4f72bd81d8c5f1a7f4e4d5e9c4a1b2584f72bd81d8c5f1a7f4e4d5e9c4a1b258",
      mimeType: "image/png",
    },
    {
      id: "version-botanical-2",
      assetId: "asset-botanical",
      parentVersionId: "version-botanical-1",
      generationEventId: "event-botanical-2",
      filePath: "originals/2026/05/3f2f8444-8cc2-4e35-91a7-806d24213b10.png",
      checksumAlgorithm: "SHA-256",
      checksum: "97290b8d8c5f1a7f4e4d5e9c4a1b25897290b8d8c5f1a7f4e4d5e9c4a1b258",
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
        checksumAlgorithm: "SHA-256",
        checksum: "4f72bd81d8c5f1a7f4e4d5e9c4a1b2584f72bd81d8c5f1a7f4e4d5e9c4a1b258",
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
    checksumAlgorithm: "SHA-256",
    checksum: "4f72bd81d8c5f1a7f4e4d5e9c4a1b2584f72bd81d8c5f1a7f4e4d5e9c4a1b258",
    integrityStatus: "verified",
  },
};

const mockAlbumList: AlbumListItem[] = [
  { id: "album-nature", name: "Nature Studies", kind: "manual", itemCount: 18 },
  { id: "album-neon", name: "Neon & Glow", kind: "manual", itemCount: 11 },
  { id: "album-product", name: "Product Shots", kind: "manual", itemCount: 6 },
];

const mockSuggestions: Suggestion[] = [
  {
    id: "suggestion-1",
    assetId: "asset-botanical",
    title: "Neon Botanical Study",
    description: "High-contrast botanical line art.",
    schemaPrompt: mockSchemaPrompt,
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
  const [selectedGalleryAssetIds, setSelectedGalleryAssetIds] = useState<string[]>([]);
  const [query, setQuery] = useState<GalleryQueryState>(defaultGalleryQuery);
  const [selectedAssetId, setSelectedAssetId] = useState(runningInTauri ? "" : mockGallery[0].id);
  const [detailState, setDetailState] = useState<DetailLoadState<AssetDetail>>({
    assetId: runningInTauri ? null : mockDetail.id,
    detail: runningInTauri ? null : mockDetail,
    loading: false,
    error: null,
  });
  const [albums, setAlbums] = useState<AlbumListItem[]>(runningInTauri ? [] : mockAlbumList);
  const [selectedAlbumId, setSelectedAlbumId] = useState<string | null>(null);
  const [albumSearchInput, setAlbumSearchInput] = useState("");
  const [albumNameInput, setAlbumNameInput] = useState("");
  const [albumCreateOpen, setAlbumCreateOpen] = useState(false);
  const [albumLoading, setAlbumLoading] = useState(false);
  const [suggestions, setSuggestions] = useState<Suggestion[]>(runningInTauri ? [] : mockSuggestions);
  const [selectedSuggestionId, setSelectedSuggestionId] = useState<string | null>(
    runningInTauri ? null : mockSuggestions[0]?.id ?? null,
  );
  const [selectedSuggestionIds, setSelectedSuggestionIds] = useState<string[]>([]);
  const [suggestionHistory, setSuggestionHistory] = useState<Suggestion[]>([]);
  const [suggestionRegenerating, setSuggestionRegenerating] = useState(false);
  const [reviewForm, setReviewForm] = useState<ReviewFormState | null>(
    runningInTauri && mockSuggestions[0] ? null : createReviewFormState(mockSuggestions[0]),
  );
  const [queue, setQueue] = useState<QueueJob[]>(mockQueue);
  const [prompt, setPrompt] = useState("");
  const [provider, setProvider] = useState("codex-cli");
  const [composerOpen, setComposerOpen] = useState(false);
  const [status, setStatus] = useState(runningInTauri ? "Open or create a library" : "Preview mode");
  const [recoverableError, setRecoverableError] = useState<string | null>(null);
  const [libraryPathInput, setLibraryPathInput] = useState("");
  const [libraryNameInput, setLibraryNameInput] = useState("Image Prompt Lab");
  const [appLogs, setAppLogs] = useState<AppLog[]>([]);
  const [logsLoading, setLogsLoading] = useState(false);
  const [selectedLogPath, setSelectedLogPath] = useState<string | null>(null);
  const [selectedLogContent, setSelectedLogContent] = useState<AppLogContent | null>(null);
  const [logContentLoading, setLogContentLoading] = useState(false);
  const logReadRequestRef = useRef<string | null>(null);

  const displayedGallery = useMemo(
    () => (runningInTauri ? gallery : applyGalleryQuery(mockGallery, query)),
    [runningInTauri, gallery, query],
  );
  const selectedAsset = useMemo(
    () => displayedGallery.find((asset) => asset.id === selectedAssetId) ?? displayedGallery[0] ?? null,
    [displayedGallery, selectedAssetId],
  );
  const pendingSuggestions = suggestions.filter((suggestion) => suggestion.status === "pending_review");
  const selectedSuggestion =
    pendingSuggestions.find((suggestion) => suggestion.id === selectedSuggestionId) ?? pendingSuggestions[0] ?? null;
  const availableTags = useMemo(
    () => Array.from(new Set((runningInTauri ? gallery : mockGallery).flatMap((asset) => asset.tags))).sort(),
    [runningInTauri, gallery],
  );
  const availableCategories = useMemo(
    () =>
      Array.from(
        new Set((runningInTauri ? gallery : mockGallery).map((asset) => asset.category).filter((category): category is string => Boolean(category))),
      ).sort(),
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
    if (runningInTauri && library) {
      void refreshAlbums();
      void refreshSuggestions();
    }
  }, [runningInTauri, library?.rootPath]);

  useEffect(() => {
    if (activeView === "settings") {
      void refreshAppLogs();
    }
  }, [activeView, runningInTauri]);

  useEffect(() => {
    if (!selectedSuggestion) {
      setSelectedSuggestionId(null);
      setReviewForm(null);
      setSuggestionHistory([]);
      return;
    }
    if (reviewForm?.suggestionId !== selectedSuggestion.id) {
      setSelectedSuggestionId(selectedSuggestion.id);
      setReviewForm(createReviewFormState(selectedSuggestion));
    }
    void refreshSuggestionHistory(selectedSuggestion);
  }, [selectedSuggestion?.id]);

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

  async function refreshAlbums() {
    if (!runningInTauri || !library) {
      setAlbums([]);
      return;
    }
    setAlbumLoading(true);
    try {
      const items = await invokeCommand<AlbumListItem[]>("list_albums", { libraryPath: library.rootPath });
      setAlbums(items);
      setSelectedAlbumId((current) => (current && items.some((item) => item.id === current) ? current : null));
      setRecoverableError(null);
    } catch (error) {
      setAlbums([]);
      setRecoverableError(errorMessage(error));
    } finally {
      setAlbumLoading(false);
    }
  }

  async function refreshSuggestions() {
    if (!runningInTauri || !library) {
      setSuggestions([]);
      setSelectedSuggestionId(null);
      setReviewForm(null);
      return;
    }
    try {
      const items = await invokeCommand<Suggestion[]>("list_pending_suggestions", { libraryPath: library.rootPath });
      setSuggestions(items);
      setSelectedSuggestionId((current) => (current && items.some((item) => item.id === current) ? current : items[0]?.id ?? null));
      setRecoverableError(null);
    } catch (error) {
      setSuggestions([]);
      setSelectedSuggestionId(null);
      setReviewForm(null);
      setRecoverableError(errorMessage(error));
    }
  }

  async function refreshSuggestionHistory(suggestion: Suggestion) {
    if (!runningInTauri || !library) {
      setSuggestionHistory(
        mockSuggestions.filter((item) => item.assetId === suggestion.assetId),
      );
      return;
    }
    try {
      const history = await invokeCommand<Suggestion[]>("list_suggestion_history", {
        input: {
          libraryPath: library.rootPath,
          assetId: suggestion.assetId,
        },
      });
      setSuggestionHistory(history);
      setRecoverableError(null);
    } catch (error) {
      setSuggestionHistory([]);
      setRecoverableError(errorMessage(error));
    }
  }

  async function refreshAppLogs() {
    setLogsLoading(true);
    try {
      if (!runningInTauri) {
        setAppLogs([]);
        setSelectedLogPath(null);
        setSelectedLogContent(null);
        setRecoverableError(null);
        return;
      }
      const logs = await invokeCommand<AppLog[]>("list_app_logs");
      setAppLogs(logs);
      setSelectedLogPath((current) => {
        const next = current && logs.some((log) => log.path === current) ? current : logs[0]?.path ?? null;
        if (!next) {
          setSelectedLogContent(null);
        } else if (next !== current) {
          void readAppLog(next);
        }
        return next;
      });
      setRecoverableError(null);
    } catch (error) {
      setAppLogs([]);
      setSelectedLogPath(null);
      setSelectedLogContent(null);
      setRecoverableError(errorMessage(error));
    } finally {
      setLogsLoading(false);
    }
  }

  async function readAppLog(path: string) {
    const requestId = crypto.randomUUID();
    logReadRequestRef.current = requestId;
    setSelectedLogPath(path);
    setLogContentLoading(true);
    try {
      if (!runningInTauri) {
        if (logReadRequestRef.current === requestId) {
          setSelectedLogContent(null);
        }
        setRecoverableError(null);
        return;
      }
      const content = await invokeCommand<AppLogContent>("read_app_log", { input: { path } });
      if (logReadRequestRef.current === requestId && content.path === path) {
        setSelectedLogContent(content);
      }
      setRecoverableError(null);
    } catch (error) {
      if (logReadRequestRef.current === requestId) {
        setSelectedLogContent(null);
      }
      setRecoverableError(errorMessage(error));
    } finally {
      if (logReadRequestRef.current === requestId) {
        setLogContentLoading(false);
      }
    }
  }

  function switchLibrary(libraryId: string) {
    const nextLibrary = libraries.find((item) => item.id === libraryId) ?? null;
    const cleared = clearCurationStateForLibrarySwitch();
    setLibrary(nextLibrary);
    setSelectedAssetId("");
    setDetailState(clearSelectionForLibrarySwitch());
    setGallery([]);
      setAlbums(runningInTauri ? [] : mockAlbumList);
      setSelectedAlbumId(cleared.selectedAlbumId);
      setAlbumSearchInput("");
      setAlbumNameInput("");
      setAlbumCreateOpen(false);
      setSelectedSuggestionId(cleared.selectedSuggestionId);
    setSelectedSuggestionIds([]);
    setSuggestionHistory([]);
    setReviewForm(cleared.reviewForm);
    setSuggestions(runningInTauri ? [] : mockSuggestions);
    setQuery(clearAlbumQuery(query));
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
      setAlbums([]);
      setAlbumSearchInput("");
      setAlbumNameInput("");
      setAlbumCreateOpen(false);
      setSuggestions([]);
      setSelectedAlbumId(null);
      setAlbumSearchInput("");
      setAlbumNameInput("");
      setAlbumCreateOpen(false);
      setSelectedSuggestionId(null);
      setReviewForm(null);
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
      setSelectedAlbumId(null);
      setSelectedSuggestionId(null);
      setReviewForm(null);
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
        await refreshSuggestions();
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

  async function createAlbum() {
    const name = albumNameInput.trim();
    if (!library || name.length === 0) {
      return;
    }
    if (!runningInTauri) {
      const created: AlbumListItem = {
        id: `album-${crypto.randomUUID()}`,
        name,
        kind: "manual",
        itemCount: 0,
        sortOrder: albums.length + 1,
      };
      setAlbums((current) => [created, ...current]);
      setAlbumNameInput("");
      setAlbumSearchInput("");
      setAlbumCreateOpen(false);
      setSelectedAlbumId(created.id);
      setQuery((current) => openAlbumQuery(current, created.id));
      setStatus("Album created");
      setRecoverableError(null);
      return;
    }
    try {
      const created = await invokeCommand<Album>("create_manual_album", {
        input: {
          libraryPath: library.rootPath,
          name,
        },
      });
      setAlbumNameInput("");
      setAlbumSearchInput("");
      setAlbumCreateOpen(false);
      await refreshAlbums();
      setSelectedAlbumId(created.id);
      setQuery((current) => openAlbumQuery(current, created.id));
      setStatus("Album created");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function createSmartAlbum(name: string, smartQueryJson: string) {
    const trimmed = name.trim();
    if (!library || trimmed.length === 0) {
      return;
    }
    if (!runningInTauri) {
      const created: AlbumListItem = {
        id: `album-${crypto.randomUUID()}`,
        name: trimmed,
        kind: "smart",
        itemCount: 0,
        sortOrder: albums.length + 1,
      };
      setAlbums((current) => [created, ...current]);
      setSelectedAlbumId(created.id);
      setQuery((current) => openAlbumQuery(current, created.id));
      return;
    }
    try {
      const created = await invokeCommand<Album>("create_smart_album", {
        input: {
          libraryPath: library.rootPath,
          name: trimmed,
          smartQueryJson,
        },
      });
      setAlbumNameInput("");
      setAlbumSearchInput("");
      setAlbumCreateOpen(false);
      await refreshAlbums();
      setSelectedAlbumId(created.id);
      setQuery((current) => openAlbumQuery(current, created.id));
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  function openAlbum(albumId: string) {
    setSelectedAlbumId(albumId);
    setQuery((current) => updateGalleryQuery(openAlbumQuery(current, albumId), { sort: "albumOrder" }));
    setActiveView("albums");
  }

  function closeAlbum() {
    setSelectedAlbumId(null);
    setQuery((current) => clearAlbumQuery(current));
  }

  async function renameAlbum(albumId: string, name: string) {
    const trimmed = name.trim();
    if (!library || trimmed.length === 0) {
      return;
    }
    if (!runningInTauri) {
      setAlbums((current) => current.map((album) => (album.id === albumId ? { ...album, name: trimmed } : album)));
      return;
    }
    try {
      await invokeCommand<Album>("rename_album", { input: { albumId, name: trimmed } });
      await refreshAlbums();
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function deleteAlbumById(albumId: string) {
    if (!library) {
      return;
    }
    if (!runningInTauri) {
      setAlbums((current) => current.filter((album) => album.id !== albumId));
      if (selectedAlbumId === albumId) {
        closeAlbum();
      }
      return;
    }
    try {
      await invokeCommand("delete_album", { albumId });
      if (selectedAlbumId === albumId) {
        closeAlbum();
      }
      await refreshAlbums();
      await refreshGallery();
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function reorderAlbumsByIds(albumIds: string[]) {
    const next = reorderByIds(albums, albumIds);
    setAlbums(next);
    if (!runningInTauri || !library) {
      return;
    }
    try {
      await invokeCommand("reorder_albums", {
        input: {
          libraryPath: library.rootPath,
          albumIds: next.map((album) => album.id),
        },
      });
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
      await refreshAlbums();
    }
  }

  async function removeAssetFromSelectedAlbum(assetId: string) {
    if (!library || !selectedAlbumId) {
      return;
    }
    if (!runningInTauri) {
      setGallery((current) => current.filter((asset) => asset.id !== assetId));
      return;
    }
    try {
      await invokeCommand("remove_asset_from_album", {
        input: {
          albumId: selectedAlbumId,
          assetId,
        },
      });
      await refreshAlbums();
      await refreshGallery();
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function reorderSelectedAlbumAssets(assetIds: string[]) {
    if (!library || !selectedAlbumId) {
      return;
    }
    setGallery((current) => reorderByIds(current, assetIds));
    if (!runningInTauri) {
      return;
    }
    try {
      await invokeCommand("reorder_album_items", {
        input: {
          albumId: selectedAlbumId,
          assetIds,
        },
      });
      await refreshGallery();
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
      await refreshGallery();
    }
  }

  async function addSelectedGalleryAssetsToAlbum(albumId: string) {
    if (!library || selectedGalleryAssetIds.length === 0) {
      return;
    }
    if (!runningInTauri) {
      setAlbums((current) =>
        current.map((album) =>
          album.id === albumId
            ? { ...album, itemCount: (album.itemCount ?? 0) + selectedGalleryAssetIds.length }
            : album,
        ),
      );
      return;
    }
    try {
      await invokeCommand("batch_add_assets_to_album", {
        input: {
          albumId,
          assetIds: selectedGalleryAssetIds,
        },
      });
      await refreshAlbums();
      await refreshGallery();
      setRecoverableError(null);
      setStatus("Selected assets added to album");
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function addSelectedAssetToAlbum(albumId: string) {
    const detail = detailState.detail;
    if (!library || !detail || albumId.length === 0) {
      return;
    }
    if (!runningInTauri) {
      const album = albums.find((item) => item.id === albumId);
      if (!album) {
        return;
      }
      const alreadyInAlbum = detail.albums.some((item) => item.id === albumId);
      setDetailState((current) =>
        current.detail
          ? {
              ...current,
              detail: {
                ...current.detail,
                albums: alreadyInAlbum
                  ? current.detail.albums
                  : [...current.detail.albums, { id: album.id, name: album.name, kind: album.kind }],
              },
            }
          : current,
      );
      if (!alreadyInAlbum) {
        setAlbums((current) =>
          current.map((item) =>
            item.id === albumId ? { ...item, itemCount: (item.itemCount ?? 0) + 1 } : item,
          ),
        );
      }
      setStatus(alreadyInAlbum ? "Asset already in album" : "Asset added to album");
      setRecoverableError(null);
      return;
    }
    try {
      await invokeCommand("add_asset_to_album", {
        input: {
          albumId,
          assetId: detail.id,
        },
      });
      await refreshAlbums();
      await refreshGallery();
      await loadAssetDetail(detail.id, selectedAsset?.currentVersionId ?? null);
      setStatus("Asset added to album");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  function selectSuggestion(suggestion: Suggestion) {
    setSelectedSuggestionId(suggestion.id);
    setReviewForm(createReviewFormState(suggestion));
  }

  function toggleSuggestionForBatch(suggestionId: string) {
    setSelectedSuggestionIds((current) => toggleSelection(current, suggestionId));
  }

  async function acceptReviewForm() {
    if (!library || !selectedSuggestion || !reviewForm) {
      return;
    }
    const finalForm = addReviewFormTag(reviewForm, reviewForm.tagInput);
    const finalSuggestion: Suggestion = {
      ...selectedSuggestion,
      title: finalForm.title.trim() || null,
      description: finalForm.description.trim() || null,
      schemaPrompt: finalForm.schemaPrompt.trim() || null,
      tags: reviewFormTags(finalForm),
      category:
        finalForm.category.trim() && availableCategories.includes(finalForm.category.trim())
          ? finalForm.category.trim()
          : null,
    };
    await acceptSuggestion(finalSuggestion);
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
          schemaPrompt: suggestion.schemaPrompt,
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
          schemaPrompt: suggestion.schemaPrompt,
          category: asset.category,
          tags: suggestion.tags,
          status: suggestion.status,
        });
        return state.assets;
      });
      setSuggestions((current) => removeSuggestionState(current, suggestion.id));
      await refreshGallery();
      if (detailState.detail?.id === asset.id) {
        await loadAssetDetail(asset.id, selectedAsset?.currentVersionId ?? null);
      }
      await refreshSuggestions();
      setSelectedSuggestionIds((current) => current.filter((id) => id !== suggestion.id));
      setReviewForm(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function batchAcceptReviewSuggestions() {
    if (!library) {
      return;
    }
    const ids = selectedOrCurrentIds(selectedSuggestionIds, selectedSuggestion?.id ?? null);
    if (ids.length === 0) {
      return;
    }
    const finalForm = reviewForm ? addReviewFormTag(reviewForm, reviewForm.tagInput) : null;
    const payloads = buildBatchReviewPayloads(suggestions, ids, finalForm).map((suggestion) => ({
      libraryPath: library.rootPath,
      suggestionId: suggestion.id,
      title: suggestion.title,
      description: suggestion.description ?? null,
      schemaPrompt: suggestion.schemaPrompt ?? null,
      tags: suggestion.tags,
      category:
        suggestion.category && availableCategories.includes(suggestion.category)
          ? suggestion.category
          : null,
    }));
    try {
      await invokeCommand<AssetView[]>("batch_accept_suggestions", {
        input: {
          libraryPath: library.rootPath,
          suggestions: payloads,
        },
      });
      await refreshGallery();
      await refreshSuggestions();
      setSelectedSuggestionIds([]);
      setReviewForm(null);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
      await refreshSuggestions();
    }
  }

  async function batchRejectReviewSuggestions() {
    if (!library) {
      return;
    }
    const ids = selectedOrCurrentIds(selectedSuggestionIds, selectedSuggestion?.id ?? null);
    if (ids.length === 0) {
      return;
    }
    try {
      await invokeCommand("batch_reject_suggestions", {
        input: {
          libraryPath: library.rootPath,
          suggestionIds: ids,
        },
      });
      await refreshGallery();
      await refreshSuggestions();
      setSelectedSuggestionIds([]);
      setReviewForm(null);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
      await refreshSuggestions();
    }
  }

  async function addReviewSelectionToAlbum(albumId: string) {
    if (!library || albumId.length === 0) {
      return;
    }
    const ids = selectedOrCurrentIds(selectedSuggestionIds, selectedSuggestion?.id ?? null);
    const assetIds = suggestions
      .filter((suggestion) => ids.includes(suggestion.id))
      .map((suggestion) => suggestion.assetId);
    if (assetIds.length === 0) {
      return;
    }
    try {
      await invokeCommand("batch_add_assets_to_album", {
        input: {
          albumId,
          assetIds,
        },
      });
      await refreshAlbums();
      setRecoverableError(null);
      setStatus("Review assets added to album");
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  function pickReviewHistoryField(suggestion: Suggestion, field: ReviewFieldName | "tags" | "category") {
    if (!reviewForm) {
      return;
    }
    setReviewForm(applySuggestionFieldToReviewForm(reviewForm, suggestion, field));
  }

  function restoreReviewForm() {
    if (selectedSuggestion) {
      setReviewForm(createReviewFormState(selectedSuggestion));
    }
  }

  async function regenerateReviewField(field: ReviewFieldName) {
    if (!reviewForm || !selectedSuggestion) {
      return;
    }
    if (isReviewFieldGenerating(reviewForm, field)) {
      return;
    }
    const requestId = crypto.randomUUID();
    const suggestionId = selectedSuggestion.id;
    setReviewForm(beginReviewFieldGeneration(reviewForm, field, requestId));
    const asset = gallery.find((item) => item.id === selectedSuggestion.assetId) ?? selectedAsset;
    const sourceText = asset?.prompt ?? selectedSuggestion.title ?? reviewForm.title;
    if (!runningInTauri) {
      const value = previewGeneratedReviewField(field, asset, sourceText);
      setReviewForm((current) =>
        current
          ? completeReviewFieldGeneration(current, suggestionId, field, requestId, value, null)
          : current,
      );
      return;
    }
    if (!library) {
      setReviewForm((current) =>
        current
          ? failReviewFieldGeneration(
              current,
              suggestionId,
              field,
              requestId,
              "Open a real library before regenerating review metadata.",
            )
          : current,
      );
      return;
    }
    try {
      await nextAnimationFrame();
      const result = await invokeCommand<GeneratedReviewField>("generate_review_field", {
        input: {
          libraryPath: library.rootPath,
          assetId: selectedSuggestion.assetId,
          suggestionId,
          field,
          context: reviewFieldContext(reviewForm, selectedSuggestion, asset),
        },
      });
      setReviewForm((current) =>
        current
          ? completeReviewFieldGeneration(
              current,
              suggestionId,
              field,
              requestId,
              result.value,
              result.logPath,
            )
          : current,
      );
      setRecoverableError(null);
      void refreshAppLogs();
    } catch (error) {
      const message = errorMessage(error);
      setReviewForm((current) =>
        current
          ? failReviewFieldGeneration(current, suggestionId, field, requestId, message, null)
          : current,
      );
      setRecoverableError(message);
    }
  }

  async function regenerateFullSuggestion() {
    if (!selectedSuggestion || !reviewForm) {
      return;
    }
    if (suggestionRegenerating) {
      return;
    }
    setSuggestionRegenerating(true);
    setStatus("Regenerating suggestion");
    if (!runningInTauri) {
      const regenerated: Suggestion = {
        ...selectedSuggestion,
        id: `suggestion-${crypto.randomUUID()}`,
        title: `${reviewForm.title || selectedSuggestion.title || "Untitled"} variant`,
        description: reviewForm.description || selectedSuggestion.description,
        schemaPrompt: reviewForm.schemaPrompt || selectedSuggestion.schemaPrompt,
        tags: reviewFormTags(reviewForm),
        category: reviewForm.category || selectedSuggestion.category,
      };
      setSuggestions((current) => [regenerated, ...current]);
      setSuggestionHistory((current) => [regenerated, ...current]);
      setSelectedSuggestionId(regenerated.id);
      setReviewForm(createReviewFormState(regenerated));
      setStatus("Suggestion regenerated");
      setSuggestionRegenerating(false);
      return;
    }
    if (!library) {
      setRecoverableError("Open a real library before regenerating suggestions.");
      setSuggestionRegenerating(false);
      return;
    }
    try {
      const fieldInput = {
        libraryPath: library.rootPath,
        assetId: selectedSuggestion.assetId,
        suggestionId: selectedSuggestion.id,
        field: "title" as ReviewFieldName,
        context: reviewFieldContext(reviewForm, selectedSuggestion, gallery.find((item) => item.id === selectedSuggestion.assetId) ?? selectedAsset),
      };
      const regenerated = await invokeCommand<Suggestion>("regenerate_suggestion", {
        input: fieldInput,
      });
      await refreshSuggestions();
      await refreshSuggestionHistory(regenerated);
      setSelectedSuggestionId(regenerated.id);
      setReviewForm(createReviewFormState(regenerated));
      setStatus("Suggestion regenerated");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    } finally {
      setSuggestionRegenerating(false);
    }
  }

  async function requestAssetReview(asset: GalleryAsset) {
    const existing = pendingSuggestions.find((suggestion) => suggestion.assetId === asset.id);
    if (existing) {
      setSelectedSuggestionId(existing.id);
      setReviewForm(createReviewFormState(existing));
      setActiveView("review");
      return;
    }
    if (!library) {
      const suggestion = suggestionFromAsset(asset);
      setSuggestions((current) => [suggestion, ...current.filter((item) => item.assetId !== asset.id)]);
      setGallery((current) => markAssetReviewPending(current, asset.id));
      setDetailState((current) =>
        current.detail?.id === asset.id
          ? { ...current, detail: { ...current.detail, reviewPendingCount: Math.max(current.detail.reviewPendingCount, 1) } }
          : current,
      );
      setSelectedSuggestionId(suggestion.id);
      setActiveView("review");
      return;
    }
    try {
      const suggestion = await invokeCommand<Suggestion>("create_suggestion", {
        input: {
          libraryPath: library.rootPath,
          assetId: asset.id,
          title: asset.title,
          description: null,
          schemaPrompt: schemaPromptFromAsset(asset, asset.prompt ?? asset.title ?? ""),
          tags: asset.tags,
          category: asset.category && availableCategories.includes(asset.category) ? asset.category : null,
          confidenceJson: JSON.stringify({ source: "manual_re_review" }),
        },
      });
      setGallery((current) => markAssetReviewPending(current, asset.id));
      setSelectedSuggestionId(suggestion.id);
      setActiveView("review");
      await refreshGallery();
      await refreshSuggestions();
      if (detailState.detail?.id === asset.id) {
        await loadAssetDetail(asset.id, asset.currentVersionId);
      }
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
            selectedAssetIds={selectedGalleryAssetIds}
            query={query}
            availableTags={availableTags}
            onSelect={setSelectedAssetId}
            onToggleAssetSelection={(assetId) => setSelectedGalleryAssetIds((current) => toggleSelection(current, assetId))}
            onQueryChange={setQuery}
            onRequestReview={(asset) => void requestAssetReview(asset)}
          />
        )}
        {activeView === "albums" && (
          <AlbumsView
            albums={albums}
            availableTags={availableTags}
            availableCategories={availableCategories}
            availableProviders={Array.from(new Set(gallery.map((asset) => asset.provider).filter((provider): provider is string => Boolean(provider))))}
            selectedAlbumId={selectedAlbumId}
            gallery={displayedGallery}
            loading={albumLoading}
            searchValue={albumSearchInput}
            onSearchChange={setAlbumSearchInput}
            newAlbumName={albumNameInput}
            onNewAlbumNameChange={setAlbumNameInput}
            createOpen={albumCreateOpen}
            onCreateOpenChange={setAlbumCreateOpen}
            onCreateAlbum={() => void createAlbum()}
            onCreateSmartAlbum={(name, queryJson) => void createSmartAlbum(name, queryJson)}
            onOpenAlbum={openAlbum}
            onCloseAlbum={closeAlbum}
            onRenameAlbum={(albumId, name) => void renameAlbum(albumId, name)}
            onDeleteAlbum={(albumId) => void deleteAlbumById(albumId)}
            onReorderAlbums={(albumIds) => void reorderAlbumsByIds(albumIds)}
            onRemoveAsset={(assetId) => void removeAssetFromSelectedAlbum(assetId)}
            onReorderAssets={(assetIds) => void reorderSelectedAlbumAssets(assetIds)}
            selectedGalleryAssetCount={selectedGalleryAssetIds.length}
            onBatchAddSelected={(albumId) => void addSelectedGalleryAssetsToAlbum(albumId)}
            onSelectAsset={setSelectedAssetId}
          />
        )}
        {activeView === "review" && (
          <ReviewInbox
            suggestions={pendingSuggestions}
            selectedSuggestion={selectedSuggestion}
            selectedSuggestionIds={selectedSuggestionIds}
            suggestionHistory={suggestionHistory}
            suggestionRegenerating={suggestionRegenerating}
            form={reviewForm}
            onSelect={selectSuggestion}
            onToggleSelected={toggleSuggestionForBatch}
            onFormChange={setReviewForm}
            availableTags={availableTags}
            availableCategories={availableCategories}
            albums={albums}
            onRestore={restoreReviewForm}
            onRegenerateField={(field) => void regenerateReviewField(field)}
            onRegenerateSuggestion={() => void regenerateFullSuggestion()}
            onPickHistoryField={pickReviewHistoryField}
            onAccept={() => void acceptReviewForm()}
            onBatchAccept={() => void batchAcceptReviewSuggestions()}
            onBatchReject={() => void batchRejectReviewSuggestions()}
            onAddToAlbum={(albumId) => void addReviewSelectionToAlbum(albumId)}
          />
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
            logs={appLogs}
            logsLoading={logsLoading}
            selectedLogPath={selectedLogPath}
            selectedLogContent={selectedLogContent}
            logContentLoading={logContentLoading}
            onRefreshLogs={() => void refreshAppLogs()}
            onSelectLog={(path) => void readAppLog(path)}
          />
        )}
      </section>

      <Inspector
        asset={selectedAsset}
        detailState={detailState}
        onUpdateRating={updateRating}
        onUpdateTitle={(title) => void updateTitle(title)}
        onAddTag={(tag) => void addTagToSelectedAsset(tag)}
        albums={albums}
        onAddToAlbum={(albumId) => void addSelectedAssetToAlbum(albumId)}
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
  selectedAssetIds,
  query,
  availableTags,
  onSelect,
  onToggleAssetSelection,
  onQueryChange,
  onRequestReview,
}: {
  assets: GalleryAsset[];
  selectedAssetId: string;
  selectedAssetIds: string[];
  query: GalleryQueryState;
  availableTags: string[];
  onSelect: (id: string) => void;
  onToggleAssetSelection: (id: string) => void;
  onQueryChange: (query: GalleryQueryState) => void;
  onRequestReview: (asset: GalleryAsset) => void;
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
          <article
            key={asset.id}
            className={asset.id === selectedAssetId ? "asset-card selected" : "asset-card"}
          >
            <button className="asset-card-main" onClick={() => onSelect(asset.id)}>
              <Thumbnail asset={asset} index={index} />
              <span className="asset-title">{asset.title ?? "Untitled"}</span>
            </button>
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
            <button className="card-review-button" onClick={() => onRequestReview(asset)}>
              Review
            </button>
            <label className="checkbox-row card-select-row">
              <input
                type="checkbox"
                checked={selectedAssetIds.includes(asset.id)}
                onChange={() => onToggleAssetSelection(asset.id)}
              />
              <span>Select</span>
            </label>
          </article>
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

function AlbumsView({
  albums,
  availableTags,
  availableCategories,
  availableProviders,
  selectedAlbumId,
  gallery,
  loading,
  searchValue,
  onSearchChange,
  newAlbumName,
  onNewAlbumNameChange,
  createOpen,
  onCreateOpenChange,
  onCreateAlbum,
  onCreateSmartAlbum,
  onOpenAlbum,
  onCloseAlbum,
  onRenameAlbum,
  onDeleteAlbum,
  onReorderAlbums,
  onRemoveAsset,
  onReorderAssets,
  selectedGalleryAssetCount,
  onBatchAddSelected,
  onSelectAsset,
}: {
  albums: AlbumListItem[];
  availableTags: string[];
  availableCategories: string[];
  availableProviders: string[];
  selectedAlbumId: string | null;
  gallery: GalleryAsset[];
  loading: boolean;
  searchValue: string;
  onSearchChange: (value: string) => void;
  newAlbumName: string;
  onNewAlbumNameChange: (value: string) => void;
  createOpen: boolean;
  onCreateOpenChange: (open: boolean) => void;
  onCreateAlbum: () => void;
  onCreateSmartAlbum: (name: string, queryJson: string) => void;
  onOpenAlbum: (albumId: string) => void;
  onCloseAlbum: () => void;
  onRenameAlbum: (albumId: string, name: string) => void;
  onDeleteAlbum: (albumId: string) => void;
  onReorderAlbums: (albumIds: string[]) => void;
  onRemoveAsset: (assetId: string) => void;
  onReorderAssets: (assetIds: string[]) => void;
  selectedGalleryAssetCount: number;
  onBatchAddSelected: (albumId: string) => void;
  onSelectAsset: (assetId: string) => void;
}) {
  const [newAlbumKind, setNewAlbumKind] = useState<"manual" | "smart">("manual");
  const [smartText, setSmartText] = useState("");
  const [smartTags, setSmartTags] = useState("");
  const [smartProviders, setSmartProviders] = useState("");
  const [smartMinRating, setSmartMinRating] = useState("");
  const [smartReviewPending, setSmartReviewPending] = useState(false);
  const [smartCategory, setSmartCategory] = useState("");
  const [smartStatus, setSmartStatus] = useState("");
  const [smartCreatedAtFrom, setSmartCreatedAtFrom] = useState("");
  const [smartCreatedAtTo, setSmartCreatedAtTo] = useState("");
  const [smartSort, setSmartSort] = useState<GallerySort>("newest");
  const selectedAlbum = albums.find((album) => album.id === selectedAlbumId) ?? null;
  const searchNeedle = searchValue.trim().toLocaleLowerCase();
  const visibleAlbums =
    searchNeedle.length === 0
      ? albums
      : albums.filter((album) => album.name.toLocaleLowerCase().includes(searchNeedle));
  const smartPreviewCount = previewSmartAlbumCount(gallery, {
    text: smartText,
    tags: splitCsv(smartTags),
    providers: splitCsv(smartProviders),
    minRating: smartMinRating.trim() ? Number(smartMinRating.trim()) : null,
    reviewPending: smartReviewPending,
    category: smartCategory,
    status: smartStatus,
    createdAtFrom: smartCreatedAtFrom,
    createdAtTo: smartCreatedAtTo,
  });
  return (
    <section className="albums-workspace">
      <div className="workspace-panel album-list-panel">
        <div className="panel-header">
          <div>
            <h3>Albums</h3>
            <p>{loading ? "Loading albums..." : `${albums.length} album${albums.length === 1 ? "" : "s"}`}</p>
          </div>
        </div>
        <div className="album-search-row">
          <input
            value={searchValue}
            onChange={(event) => onSearchChange(event.target.value)}
            placeholder="Search albums"
          />
          <button className="icon-button" aria-label="Create album" onClick={() => onCreateOpenChange(!createOpen)}>
            +
          </button>
          {createOpen && (
            <div className="album-create-popover">
              <label>
                <span>Album name</span>
                <input
                  value={newAlbumName}
                  autoFocus
                  onChange={(event) => onNewAlbumNameChange(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") {
                      onCreateAlbum();
                    }
                    if (event.key === "Escape") {
                      onCreateOpenChange(false);
                    }
                  }}
                  placeholder="New manual album"
                />
              </label>
              <label>
                <span>Kind</span>
                <select className="select-control" value={newAlbumKind} onChange={(event) => setNewAlbumKind(event.target.value as "manual" | "smart")}>
                  <option value="manual">Manual</option>
                  <option value="smart">Smart</option>
                </select>
              </label>
              {newAlbumKind === "smart" && (
                <div className="smart-builder">
                  <label>
                    <span>Text</span>
                    <input value={smartText} onChange={(event) => setSmartText(event.target.value)} />
                  </label>
                  <label>
                    <span>Tags</span>
                    <input list="smart-tag-options" value={smartTags} onChange={(event) => setSmartTags(event.target.value)} placeholder="comma separated" />
                  </label>
                  <datalist id="smart-tag-options">
                    {availableTags.map((tag) => <option key={tag} value={tag} />)}
                  </datalist>
                  <label>
                    <span>Providers</span>
                    <input list="smart-provider-options" value={smartProviders} onChange={(event) => setSmartProviders(event.target.value)} placeholder="comma separated" />
                  </label>
                  <datalist id="smart-provider-options">
                    {availableProviders.map((provider) => <option key={provider} value={provider} />)}
                  </datalist>
                  <label>
                    <span>Min rating</span>
                    <input value={smartMinRating} onChange={(event) => setSmartMinRating(event.target.value)} inputMode="numeric" />
                  </label>
                  <label>
                    <span>Category</span>
                    <select className="select-control" value={smartCategory} onChange={(event) => setSmartCategory(event.target.value)}>
                      <option value="">Any</option>
                      {availableCategories.map((category) => <option key={category} value={category}>{category}</option>)}
                    </select>
                  </label>
                  <label>
                    <span>Status</span>
                    <input value={smartStatus} onChange={(event) => setSmartStatus(event.target.value)} placeholder="generated, curated..." />
                  </label>
                  <label>
                    <span>Created from</span>
                    <input value={smartCreatedAtFrom} onChange={(event) => setSmartCreatedAtFrom(event.target.value)} placeholder="unix ms" />
                  </label>
                  <label>
                    <span>Created to</span>
                    <input value={smartCreatedAtTo} onChange={(event) => setSmartCreatedAtTo(event.target.value)} placeholder="unix ms" />
                  </label>
                  <label>
                    <span>Sort</span>
                    <select className="select-control" value={smartSort} onChange={(event) => setSmartSort(event.target.value as GallerySort)}>
                      <option value="newest">Newest</option>
                      <option value="oldest">Oldest</option>
                      <option value="ratingDesc">Rating</option>
                      <option value="titleAsc">Title</option>
                      <option value="providerAsc">Provider</option>
                    </select>
                  </label>
                  <label className="checkbox-row">
                    <input type="checkbox" checked={smartReviewPending} onChange={(event) => setSmartReviewPending(event.target.checked)} />
                    <span>Review pending</span>
                  </label>
                  <small>{smartPreviewCount} visible assets match this builder.</small>
                </div>
              )}
              <div className="row-actions">
                <button onClick={() => onCreateOpenChange(false)}>Cancel</button>
                <button
                  className="primary-button"
                  disabled={newAlbumName.trim().length === 0}
                  onClick={() => {
                    if (newAlbumKind === "manual") {
                      onCreateAlbum();
                      return;
                    }
                    const query = {
                      ...(smartText.trim() ? { text: smartText.trim() } : {}),
                      ...(smartTags.trim() ? { tags: splitCsv(smartTags) } : {}),
                      ...(smartProviders.trim() ? { providers: splitCsv(smartProviders) } : {}),
                      ...(smartMinRating.trim() ? { minRating: Number(smartMinRating.trim()) } : {}),
                      ...(smartReviewPending ? { reviewStatus: "pending" } : {}),
                      ...(smartCategory ? { category: smartCategory } : {}),
                      ...(smartStatus.trim() ? { status: smartStatus.trim() } : {}),
                      ...(smartCreatedAtFrom.trim() ? { createdAtFrom: smartCreatedAtFrom.trim() } : {}),
                      ...(smartCreatedAtTo.trim() ? { createdAtTo: smartCreatedAtTo.trim() } : {}),
                      sort: smartSort,
                    };
                    onCreateSmartAlbum(newAlbumName, JSON.stringify(query));
                  }}
                >
                  Create
                </button>
              </div>
            </div>
          )}
        </div>
        {albums.length === 0 ? (
          <div className="empty-state compact">No albums yet.</div>
        ) : visibleAlbums.length === 0 ? (
          <div className="empty-state compact">No albums match the search.</div>
        ) : (
          <div className="album-list">
            {visibleAlbums.map((album) => (
              <button
                key={album.id}
                className={album.id === selectedAlbumId ? "album-list-item selected" : "album-list-item"}
                draggable
                onDragStart={(event) => event.dataTransfer.setData("text/album-id", album.id)}
                onDragOver={(event) => event.preventDefault()}
                onDrop={(event) => {
                  event.preventDefault();
                  const draggedId = event.dataTransfer.getData("text/album-id");
                  if (!draggedId || draggedId === album.id) {
                    return;
                  }
                  const fromIndex = albums.findIndex((item) => item.id === draggedId);
                  const toIndex = albums.findIndex((item) => item.id === album.id);
                  const next = moveItem(albums, fromIndex, toIndex);
                  if (next !== albums) {
                    onReorderAlbums(next.map((item) => item.id));
                  }
                }}
                onClick={() => onOpenAlbum(album.id)}
              >
                <span>
                  <strong>{album.name}</strong>
                  <small>{album.kind}</small>
                </span>
                <strong>{album.itemCount ?? "-"}</strong>
              </button>
            ))}
          </div>
        )}
      </div>
      <div className="workspace-panel album-detail-panel">
        <div className="panel-header">
          <div>
            <h3>{selectedAlbum?.name ?? "Select an album"}</h3>
            <p>
              {selectedAlbum
                ? `${selectedAlbum.kind} album · ${gallery.length} visible item${gallery.length === 1 ? "" : "s"}`
                : "Open an album to view its assets."}
            </p>
          </div>
          {selectedAlbum && (
            <div className="row-actions">
              {selectedAlbum.kind === "manual" && (
                <button disabled={selectedGalleryAssetCount === 0} onClick={() => onBatchAddSelected(selectedAlbum.id)}>
                  Add selected ({selectedGalleryAssetCount})
                </button>
              )}
              <button onClick={() => {
                const next = window.prompt("Album name", selectedAlbum.name);
                if (next) {
                  onRenameAlbum(selectedAlbum.id, next);
                }
              }}>
                Rename
              </button>
              <button onClick={() => onDeleteAlbum(selectedAlbum.id)}>Delete</button>
              <button onClick={onCloseAlbum}>All assets</button>
            </div>
          )}
        </div>
        {!selectedAlbum ? (
          <div className="empty-state">Choose an album from the list.</div>
        ) : gallery.length === 0 ? (
          <div className="empty-state">This album is empty.</div>
        ) : (
          <section className="gallery-grid compact-grid">
            {gallery.map((asset, index) => (
              <button
                key={asset.id}
                className="asset-card"
                draggable={selectedAlbum.kind === "manual"}
                onDragStart={(event) => event.dataTransfer.setData("text/asset-id", asset.id)}
                onDragOver={(event) => {
                  if (selectedAlbum.kind === "manual") {
                    event.preventDefault();
                  }
                }}
                onDrop={(event) => {
                  if (selectedAlbum.kind !== "manual") {
                    return;
                  }
                  event.preventDefault();
                  const draggedId = event.dataTransfer.getData("text/asset-id");
                  if (!draggedId || draggedId === asset.id) {
                    return;
                  }
                  const fromIndex = gallery.findIndex((item) => item.id === draggedId);
                  const toIndex = gallery.findIndex((item) => item.id === asset.id);
                  const next = moveItem(gallery, fromIndex, toIndex);
                  onReorderAssets(next.map((item) => item.id));
                }}
                onClick={() => onSelectAsset(asset.id)}
              >
                <Thumbnail asset={asset} index={index} />
                <span className="asset-title">{asset.title ?? "Untitled"}</span>
                <span className="provider-pill">{asset.provider ?? "Unknown provider"}</span>
                <StarRatingDisplay rating={asset.rating} />
                {asset.reviewPendingCount > 0 && <span className="review-badge">Review pending</span>}
                {selectedAlbum.kind === "manual" && (
                  <span
                    className="text-button"
                    onClick={(event) => {
                      event.stopPropagation();
                      onRemoveAsset(asset.id);
                    }}
                  >
                    Remove
                  </span>
                )}
              </button>
            ))}
          </section>
        )}
      </div>
    </section>
  );
}

function ReviewInbox({
  suggestions,
  selectedSuggestion,
  selectedSuggestionIds,
  suggestionHistory,
  suggestionRegenerating,
  form,
  onSelect,
  onToggleSelected,
  onFormChange,
  availableTags,
  availableCategories,
  albums,
  onRestore,
  onRegenerateField,
  onRegenerateSuggestion,
  onPickHistoryField,
  onAccept,
  onBatchAccept,
  onBatchReject,
  onAddToAlbum,
}: {
  suggestions: Suggestion[];
  selectedSuggestion: Suggestion | null;
  selectedSuggestionIds: string[];
  suggestionHistory: Suggestion[];
  suggestionRegenerating: boolean;
  form: ReviewFormState | null;
  onSelect: (suggestion: Suggestion) => void;
  onToggleSelected: (suggestionId: string) => void;
  onFormChange: (form: ReviewFormState) => void;
  availableTags: string[];
  availableCategories: string[];
  albums: AlbumListItem[];
  onRestore: () => void;
  onRegenerateField: (field: ReviewFieldName) => void;
  onRegenerateSuggestion: () => void;
  onPickHistoryField: (suggestion: Suggestion, field: ReviewFieldName | "tags" | "category") => void;
  onAccept: () => void;
  onBatchAccept: () => void;
  onBatchReject: () => void;
  onAddToAlbum: (albumId: string) => void;
}) {
  const [albumToAdd, setAlbumToAdd] = useState("");
  if (suggestions.length === 0) {
    return <div className="empty-state">No pending suggestions.</div>;
  }
  return (
    <section className="review-workspace">
      <div className="workspace-panel review-list-panel">
        <div className="panel-header">
          <div>
            <h3>Pending review</h3>
            <p>{selectedSuggestionIds.length || suggestions.length} selected / {suggestions.length} pending</p>
          </div>
        </div>
        <div className="row-actions review-actions">
          <button onClick={onBatchAccept}>Accept selected</button>
          <button onClick={onBatchReject}>Reject selected</button>
        </div>
        <div className="add-album-row">
          <select className="select-control" value={albumToAdd} onChange={(event) => setAlbumToAdd(event.target.value)}>
            <option value="">Add selected to album</option>
            {albums
              .filter((album) => album.kind === "manual")
              .map((album) => (
                <option key={album.id} value={album.id}>{album.name}</option>
              ))}
          </select>
          <button disabled={!albumToAdd} onClick={() => onAddToAlbum(albumToAdd)}>Add</button>
        </div>
        <div className="review-list">
          {suggestions.map((suggestion) => (
            <div
              className={suggestion.id === selectedSuggestion?.id ? "review-list-item selected" : "review-list-item"}
              key={suggestion.id}
            >
              <input
                type="checkbox"
                checked={selectedSuggestionIds.includes(suggestion.id)}
                onChange={() => onToggleSelected(suggestion.id)}
                aria-label={`Select ${suggestion.title ?? suggestion.id}`}
              />
              <button type="button" onClick={() => onSelect(suggestion)}>
                <strong>{suggestion.title ?? "Untitled suggestion"}</strong>
                <span>{suggestion.category ?? "No category"}</span>
                <small>{suggestion.tags.join(", ") || "No tags"}</small>
              </button>
            </div>
          ))}
        </div>
      </div>
      <div className="workspace-panel review-detail-panel">
        {!selectedSuggestion || !form ? (
          <div className="empty-state">Select a suggestion to review.</div>
        ) : (
          <>
            <div className="panel-header">
              <div>
                <h3>Review metadata</h3>
                <p>Status: {selectedSuggestion.status}</p>
              </div>
              <span className="review-badge">Review pending</span>
            </div>
            <ConfidenceSummary confidence={selectedSuggestion.confidence} />
            <div className="review-form">
              <label>
                <span>
                  Title
                  <ReviewFieldGenerateButton form={form} field="title" onRegenerateField={onRegenerateField} />
                </span>
                <input
                  value={form.title}
                  disabled={isReviewFieldGenerating(form, "title")}
                  onChange={(event) => onFormChange({ ...form, title: event.target.value })}
                />
                <ReviewFieldGenerationStatus form={form} field="title" />
              </label>
              <label>
                <span>Category</span>
                <select
                  className="select-control"
                  value={form.category}
                  onChange={(event) => onFormChange({ ...form, category: event.target.value })}
                >
                  <option value="">No category</option>
                  {availableCategories.map((category) => (
                    <option key={category} value={category}>
                      {category}
                    </option>
                  ))}
                </select>
              </label>
              <label className="wide-field">
                <span>
                  Description
                  <ReviewFieldGenerateButton form={form} field="description" onRegenerateField={onRegenerateField} />
                </span>
                <textarea
                  value={form.description}
                  disabled={isReviewFieldGenerating(form, "description")}
                  onChange={(event) => onFormChange({ ...form, description: event.target.value })}
                />
                <ReviewFieldGenerationStatus form={form} field="description" />
              </label>
              <label className="wide-field">
                <span>
                  JSON Schema Prompt
                  <ReviewFieldGenerateButton form={form} field="schemaPrompt" onRegenerateField={onRegenerateField} />
                </span>
                <textarea
                  className="schema-prompt-input"
                  value={form.schemaPrompt}
                  disabled={isReviewFieldGenerating(form, "schemaPrompt")}
                  onChange={(event) => onFormChange({ ...form, schemaPrompt: event.target.value })}
                  spellCheck={false}
                />
                <ReviewFieldGenerationStatus form={form} field="schemaPrompt" />
              </label>
              <div className="wide-field tag-editor-field">
                <span>Tags</span>
                <div className="tag-chip-editor">
                  {form.tags.map((tag) => (
                    <button
                      key={tag}
                      className="tag-chip removable"
                      onClick={() => onFormChange(removeReviewFormTag(form, tag))}
                    >
                      {tag} x
                    </button>
                  ))}
                  <input
                    list="review-tag-options"
                    value={form.tagInput}
                    onChange={(event) => onFormChange({ ...form, tagInput: event.target.value })}
                    onKeyDown={(event) => {
                      if (event.key === "Enter") {
                        event.preventDefault();
                        onFormChange(addReviewFormTag(form, form.tagInput));
                      }
                    }}
                    placeholder="Add tag"
                  />
                  <datalist id="review-tag-options">
                    {availableTags.map((tag) => (
                      <option key={tag} value={tag} />
                    ))}
                  </datalist>
                </div>
              </div>
            </div>
            <div className="row-actions review-actions">
              <button onClick={onRestore}>Restore</button>
              <button disabled={suggestionRegenerating} onClick={onRegenerateSuggestion}>
                {suggestionRegenerating ? "Regenerating..." : "Regenerate suggestion"}
              </button>
              <button className="primary-button" onClick={onAccept}>
                Accept changes
              </button>
            </div>
            <SuggestionHistoryTable history={suggestionHistory} onPickField={onPickHistoryField} />
          </>
        )}
      </div>
    </section>
  );
}

function ConfidenceSummary({ confidence }: { confidence?: ConfidenceScore }) {
  if (!confidence) {
    return null;
  }
  const fields = [
    ["Title", confidence.title],
    ["Description", confidence.description],
    ["Schema", confidence.schemaPrompt],
    ["Tags", confidence.tags],
    ["Category", confidence.category],
  ] as const;
  const knownFields = fields.filter(([, score]) => typeof score === "number");
  if (typeof confidence.overall !== "number" && knownFields.length === 0) {
    return null;
  }
  return (
    <div className="confidence-panel">
      <strong>Confidence {formatScore(confidence.overall)}</strong>
      <div className="tag-list">
        {knownFields.map(([label, score]) => (
          <span key={label}>{label}: {formatScore(score)}</span>
        ))}
      </div>
    </div>
  );
}

function SuggestionHistoryTable({
  history,
  onPickField,
}: {
  history: Suggestion[];
  onPickField: (suggestion: Suggestion, field: ReviewFieldName | "tags" | "category") => void;
}) {
  if (history.length === 0) {
    return <div className="empty-state compact">No suggestion history.</div>;
  }
  return (
    <div className="history-table">
      <h3>Suggestion history</h3>
      {history.map((suggestion) => (
        <article key={suggestion.id} className="history-row">
          <div>
            <strong>{suggestion.title ?? "Untitled"}</strong>
            <small>{suggestion.status} · {suggestion.createdAt ? displayDate(suggestion.createdAt) : "-"}</small>
          </div>
          <div className="row-actions">
            <button onClick={() => onPickField(suggestion, "title")}>Title</button>
            <button onClick={() => onPickField(suggestion, "description")}>Description</button>
            <button onClick={() => onPickField(suggestion, "schemaPrompt")}>Schema</button>
            <button onClick={() => onPickField(suggestion, "tags")}>Tags</button>
            <button onClick={() => onPickField(suggestion, "category")}>Category</button>
          </div>
        </article>
      ))}
    </div>
  );
}

function formatScore(score: number | null | undefined): string {
  return typeof score === "number" ? `${score}%` : "unknown";
}

function splitCsv(value: string): string[] {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

function previewSmartAlbumCount(
  assets: GalleryAsset[],
  query: {
    text: string;
    tags: string[];
    providers: string[];
    minRating: number | null;
    reviewPending: boolean;
    category: string;
    status: string;
    createdAtFrom: string;
    createdAtTo: string;
  },
): number {
  const text = query.text.trim().toLocaleLowerCase();
  return assets.filter((asset) => {
    if (text && ![asset.title, asset.category, asset.status, asset.provider, asset.modelLabel, asset.prompt, ...asset.tags].some((value) => value?.toLocaleLowerCase().includes(text))) {
      return false;
    }
    if (query.tags.length > 0 && !query.tags.every((tag) => asset.tags.includes(tag))) {
      return false;
    }
    if (query.providers.length > 0 && (!asset.provider || !query.providers.includes(asset.provider))) {
      return false;
    }
    if (query.minRating !== null && (asset.rating ?? 0) < query.minRating) {
      return false;
    }
    if (query.reviewPending && asset.reviewPendingCount === 0) {
      return false;
    }
    if (query.category && asset.category !== query.category) {
      return false;
    }
    if (query.status.trim() && asset.status !== query.status.trim()) {
      return false;
    }
    if (query.createdAtFrom.trim() && asset.createdAt < query.createdAtFrom.trim()) {
      return false;
    }
    if (query.createdAtTo.trim() && asset.createdAt > query.createdAtTo.trim()) {
      return false;
    }
    return true;
  }).length;
}

function ReviewFieldGenerateButton({
  form,
  field,
  onRegenerateField,
}: {
  form: ReviewFormState;
  field: ReviewFieldName;
  onRegenerateField: (field: ReviewFieldName) => void;
}) {
  const loading = isReviewFieldGenerating(form, field);
  return (
    <button
      type="button"
      className="inline-action"
      disabled={loading}
      onClick={() => onRegenerateField(field)}
    >
      {loading ? "Generating..." : "Regenerate"}
    </button>
  );
}

function ReviewFieldGenerationStatus({
  form,
  field,
}: {
  form: ReviewFormState;
  field: ReviewFieldName;
}) {
  const state = form.generation[field];
  if (state.error) {
    return <small className="field-status error-text">Generation failed. Check the message above or Settings Logs.</small>;
  }
  return null;
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
  logs,
  logsLoading,
  selectedLogPath,
  selectedLogContent,
  logContentLoading,
  onRefreshLogs,
  onSelectLog,
}: {
  library: Library | null;
  libraries: Library[];
  libraryPath: string;
  libraryName: string;
  onLibraryPathChange: (value: string) => void;
  onLibraryNameChange: (value: string) => void;
  onCreate: () => void;
  onOpen: () => void;
  logs: AppLog[];
  logsLoading: boolean;
  selectedLogPath: string | null;
  selectedLogContent: AppLogContent | null;
  logContentLoading: boolean;
  onRefreshLogs: () => void;
  onSelectLog: (path: string) => void;
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
      <div className="settings-logs-panel">
        <div className="panel-header">
          <div>
            <h3>Logs</h3>
            <p>{logsLoading ? "Loading logs..." : `${logs.length} recent log${logs.length === 1 ? "" : "s"}`}</p>
          </div>
          <button onClick={onRefreshLogs} disabled={logsLoading}>
            {logsLoading ? "Refreshing..." : "Refresh"}
          </button>
        </div>
        {logs.length === 0 ? (
          <div className="empty-state compact">No app logs found.</div>
        ) : (
          <div className="logs-browser">
            <div className="logs-list">
              {logs.map((log) => (
                <button
                  key={log.path}
                  className={log.path === selectedLogPath ? "log-list-item selected" : "log-list-item"}
                  onClick={() => onSelectLog(log.path)}
                >
                  <span className="log-list-heading">
                    <strong>{log.kind}</strong>
                    <span>{formatBytes(log.sizeBytes)}</span>
                  </span>
                  <span className="log-list-meta">
                    <span>{displayDate(log.modifiedAt)}</span>
                  </span>
                </button>
              ))}
            </div>
            <div className="log-preview">
              {logContentLoading ? (
                <div className="empty-state compact">Loading log preview...</div>
              ) : selectedLogContent ? (
                <>
                  <div className="log-preview-meta">
                    <span className="mono-line">{selectedLogContent.path}</span>
                    {selectedLogContent.truncated && <strong>Truncated</strong>}
                  </div>
                  <pre>{selectedLogContent.content || "Log is empty."}</pre>
                </>
              ) : (
                <div className="empty-state compact">Select a log to preview.</div>
              )}
            </div>
          </div>
        )}
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
  albums,
  onAddToAlbum,
  onGenerateVariation,
}: {
  asset: GalleryAsset | null;
  detailState: DetailLoadState<AssetDetail>;
  onUpdateRating: (rating: number) => void;
  onUpdateTitle: (title: string) => void;
  onAddTag: (tag: string) => void;
  albums: AlbumListItem[];
  onAddToAlbum: (albumId: string) => void;
  onGenerateVariation: () => void;
}) {
  const detail = detailState.detail;
  const [tagInput, setTagInput] = useState("");
  const [tagEditorOpen, setTagEditorOpen] = useState(false);
  const [titleEditing, setTitleEditing] = useState(false);
  const [titleInput, setTitleInput] = useState("");
  const [albumToAdd, setAlbumToAdd] = useState("");
  useEffect(() => {
    setTagInput("");
    setTagEditorOpen(false);
    setTitleEditing(false);
    setTitleInput(detail?.title ?? asset?.title ?? "");
    setAlbumToAdd("");
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
        <MetaRow label="Category" value={detail.category ?? asset.category ?? "-"} />
        <MetaRow label="Parameters" value={detail.parametersJson ?? "-"} />
      </InspectorSection>
      <InspectorSection title="Description">
        <p>{detail.description ?? "No description."}</p>
      </InspectorSection>
      <InspectorSection title="JSON Schema Prompt">
        <pre className="schema-prompt-preview">{detail.schemaPrompt ?? "No schema prompt."}</pre>
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
        <div className="add-album-row">
          <select className="select-control" value={albumToAdd} onChange={(event) => setAlbumToAdd(event.target.value)}>
            <option value="">Add to album</option>
            {albums
              .filter((album) => album.kind === "manual")
              .map((album) => (
                <option key={album.id} value={album.id}>
                  {album.name}
                </option>
              ))}
          </select>
          <button
            className="mini-button"
            disabled={albumToAdd.length === 0}
            onClick={() => {
              onAddToAlbum(albumToAdd);
              setAlbumToAdd("");
            }}
          >
            Add
          </button>
        </div>
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
    albumId: query.albumId,
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

function nextAnimationFrame() {
  return new Promise<void>((resolve) => {
    window.requestAnimationFrame(() => resolve());
  });
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

function suggestionFromAsset(asset: GalleryAsset): Suggestion {
  return {
    id: `suggestion-${asset.id}-${Date.now()}`,
    assetId: asset.id,
    title: asset.title,
    description: null,
    schemaPrompt: schemaPromptFromAsset(asset, asset.prompt ?? asset.title ?? ""),
    tags: asset.tags,
    category: asset.category,
    status: "pending_review",
  };
}

function reviewFieldContext(
  form: ReviewFormState,
  suggestion: Suggestion,
  asset: GalleryAsset | null | undefined,
) {
  return {
    currentTitle: form.title.trim() || null,
    currentDescription: form.description.trim() || null,
    currentSchemaPrompt: form.schemaPrompt.trim() || null,
    assetTitle: asset?.title ?? suggestion.title,
    assetPrompt: asset?.prompt ?? null,
    tags: reviewFormTags(form),
    category: form.category.trim() || suggestion.category,
    provider: asset?.provider ?? null,
    modelLabel: asset?.modelLabel ?? null,
    width: asset?.width ?? null,
    height: asset?.height ?? null,
  };
}

function previewGeneratedReviewField(
  field: ReviewFieldName,
  asset: GalleryAsset | null | undefined,
  sourceText: string,
): string {
  if (field === "title") {
    return titleFromPrompt(sourceText);
  }
  if (field === "description") {
    return descriptionFromPrompt(sourceText);
  }
  return schemaPromptFromAsset(asset, sourceText);
}

function titleFromPrompt(prompt: string | null | undefined): string {
  const words = (prompt ?? "")
    .split(/[^a-zA-Z0-9]+/)
    .map((word) => word.trim().toLowerCase())
    .filter((word) => word.length > 0 && !["a", "an", "and", "the", "of", "with", "to", "for"].includes(word))
    .slice(0, 6)
    .map((word) => word.slice(0, 1).toUpperCase() + word.slice(1));
  return words.join(" ") || "Untitled Review";
}

function descriptionFromPrompt(prompt: string | null | undefined): string {
  const text = (prompt ?? "").trim();
  if (!text) {
    return "Generated visual asset prepared for metadata review.";
  }
  return `Review draft based on the generation prompt: ${text}`;
}

function schemaPromptFromAsset(asset: GalleryAsset | null | undefined, sourceText: string): string {
  const aspectRatio = asset?.width && asset.height ? formatAspectRatio(asset.width, asset.height) : "unspecified";
  const schema = {
    GLOBAL_SETTINGS: {
      aspect_ratio: aspectRatio,
      style: "derived from current asset and prompt",
      clarity: "sharp foreground, visible subject detail",
      render_flags: ["sharp_foreground", "micro_texture", "editorial_finish"],
    },
    ENVIRONMENT: {
      background: "preserve the generated image environment cues",
      lighting: "preserve visible lighting direction and contrast",
      atmosphere: ["match final image mood", "avoid unsupported scene changes"],
    },
    CORE_ASSETS: {
      primary_subject: sourceText || asset?.title || "reviewed image asset",
      materials: ["infer visible materials from final image"],
      composition: "preserve generated composition and camera framing",
    },
    OUTPUT: {
      mood: "match accepted visual direction",
      avoid: ["cheap e-commerce banner", "plastic CGI", "fake brand logos"],
    },
  };
  return `// VERSION: 0.1\n// AESTHETIC: review draft\n${JSON.stringify(schema, null, 2)}`;
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

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
