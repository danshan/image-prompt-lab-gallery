import type { AlbumListItem, AssetDetail, DaemonTask, GalleryAsset, Library, LibraryStatus, ProviderHealth, Suggestion, TaskDraft } from "./types";

export const mockLibrary: Library = {
  id: "library-local",
  name: "MyImageLab.library",
  rootPath: "/Users/demo/ImagePromptLab",
  hidden: false,
  schemaVersion: 1,
};

export const mockLibraries: Library[] = [
  mockLibrary,
  {
    id: "library-reference",
    name: "Reference Sets",
    rootPath: "/Users/demo/ReferenceSets",
    hidden: false,
    schemaVersion: 1,
  },
];

export const mockLibraryStatus: LibraryStatus = {
  storageSizeBytes: 153223987,
  integrityStatus: "healthy",
  integrityIssueCount: 0,
};

export const mockProviderHealth: ProviderHealth[] = [
  {
    provider: "codex-cli",
    displayName: "Codex CLI",
    availability: "not_checked",
    credentialState: "external",
    supportedOperations: ["text_to_image", "image_to_image"],
    recoverableError: null,
  },
  {
    provider: "fake",
    displayName: "Fake",
    availability: "available",
    credentialState: "not_required",
    supportedOperations: ["text_to_image"],
    recoverableError: null,
  },
  {
    provider: "grok",
    displayName: "Grok",
    availability: "not_configured",
    credentialState: "missing",
    supportedOperations: ["text_to_image"],
    recoverableError: "native provider client is deferred",
  },
];

export const mockSchemaPrompt = `// VERSION: 0.1
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

export function mockImageDataUrl(startColor: string, endColor: string) {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0%" stop-color="${startColor}"/><stop offset="100%" stop-color="${endColor}"/></linearGradient></defs><rect width="1024" height="1024" fill="url(#g)"/></svg>`;
  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
}

export const mockGallery: GalleryAsset[] = [
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
    imagePath: mockImageDataUrl("#05151b", "#d69a2d"),
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
    imagePath: mockImageDataUrl("#485368", "#ff9d78"),
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
    imagePath: mockImageDataUrl("#f7f5ee", "#4c6b63"),
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
    imagePath: mockImageDataUrl("#cc522c", "#4c1639"),
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
    imagePath: mockImageDataUrl("#111827", "#c3a978"),
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
    imagePath: mockImageDataUrl("#04283f", "#d98646"),
    width: 1024,
    height: 1536,
    versionLabel: "v3",
    versionCount: 3,
    createdAt: "Monday, 1:03 PM",
    updatedAt: "Monday, 1:03 PM",
  },
];

export const mockDetail: AssetDetail = {
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
  currentVersionId: "version-botanical-3",
  currentVersionNumber: 3,
  currentVersionName: "v3",
  versions: [
    {
      id: "version-botanical-3",
      assetId: "asset-botanical",
      parentVersionId: "version-botanical-2",
      generationEventId: "event-botanical-3",
      versionNumber: 3,
      versionName: "v3",
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
      versionNumber: 2,
      versionName: "v2",
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
        versionNumber: 3,
        versionName: "v3",
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

export const mockAlbumList: AlbumListItem[] = [
  { id: "album-nature", name: "Nature Studies", kind: "manual", itemCount: 18 },
  { id: "album-neon", name: "Neon & Glow", kind: "manual", itemCount: 11 },
  { id: "album-product", name: "Product Shots", kind: "manual", itemCount: 6 },
];

export const mockSuggestions: Suggestion[] = [
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

export function createTaskDraft(patch: Partial<TaskDraft> = {}): TaskDraft {
  return {
    id: crypto.randomUUID(),
    prompt: "",
    provider: "codex-cli",
    operation: "text_to_image",
    negativePrompt: "",
    inputFile: "",
    inputVersionId: null,
    parametersJson: "{}",
    priority: 0,
    maxAttempts: 3,
    ...patch,
  };
}

export const mockTasks: DaemonTask[] = [
  {
    id: "task-1",
    libraryId: mockLibrary.id,
    taskType: "image_generation",
    provider: "codex-cli",
    operation: "text_to_image",
    input: { prompt: "Retro UI poster with a glass scanner bed and annotated prompt tokens." },
    status: "queued",
    queuePosition: 1,
    priority: 0,
    concurrencyGroup: null,
    attemptCount: 0,
    maxAttempts: 3,
    nextRetryAt: null,
    createdAt: "2026-05-18T00:00:00Z",
    updatedAt: "2026-05-18T00:00:00Z",
    lastErrorCode: null,
    lastErrorMessage: null,
    errorClassification: null,
    waitReason: null,
  },
];
