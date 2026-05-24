import type { ReviewFieldName } from "./workflows/review/state.js";

export type View = "gallery" | "albums" | "prompts" | "review" | "queue" | "settings";
export type TaskPanel = "compose" | "queue" | "detail";

export const GALLERY_REFRESH_DEBOUNCE_MS = 250;
export const METADATA_POLL_INTERVAL_MS = 800;
export const TASK_QUEUE_POLL_INTERVAL_MS = 1500;
export const TASK_QUEUE_BACKGROUND_POLL_INTERVAL_MS = 4000;

export const initialUpdateState: UpdateState = {
  currentVersion: "0.1.0",
  lastCheckedAt: null,
  checking: false,
  installing: false,
  pendingRestart: false,
  availableUpdate: null,
  error: null,
  status: "idle",
};

export type Library = {
  id: string;
  name: string;
  rootPath: string;
  hidden: boolean;
  schemaVersion: number;
};

export type LibraryStatus = {
  storageSizeBytes: number;
  integrityStatus: string;
  integrityIssueCount: number;
};

export type LibraryBackup = {
  library: Library;
  cloned: boolean;
};

export type ProviderHealth = {
  provider: string;
  displayName: string;
  availability: string;
  credentialState: string;
  supportedOperations: string[];
  recoverableError: string | null;
};

export type GalleryAsset = {
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
  currentVersionNumber?: number | null;
  currentVersionName?: string | null;
  currentVersionTreeName?: string | null;
  imagePath: string | null;
  width: number | null;
  height: number | null;
  versionLabel: string | null;
  versionCount: number;
  versionTreeBranchCount?: number;
  albums: Album[];
  albumContext: Album | null;
  createdAt: string;
  updatedAt: string;
};

export type LightboxImage = {
  path: string;
  label: string;
};

export type AssetView = {
  id: string;
  title: string | null;
  category: string | null;
  rating: number | null;
  status: string;
};

export type Version = {
  id: string;
  assetId: string;
  parentVersionId: string | null;
  generationEventId: string | null;
  versionNumber?: number;
  versionName?: string;
  filePath: string;
  checksumAlgorithm: string;
  checksum: string;
  mimeType: string;
};

export type GenerationEvent = {
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

export type PromptDocument = {
  id: string;
  name: string;
  kind: string;
  status: string;
  draftBody: string;
  draftNegativePrompt: string | null;
  draftStylePrompt: string | null;
  variablesSchemaJson: string;
  defaultValuesJson: string;
  parameterPresetJson: string;
  notes: string | null;
  latestVersionId: string | null;
  latestVersionNumber: number | null;
  latestVersionName: string | null;
  createdAt: string;
  updatedAt: string;
  archivedAt: string | null;
};

export type PromptVersion = {
  id: string;
  promptId: string;
  versionNumber: number;
  versionName: string;
  body: string;
  negativePrompt: string | null;
  stylePrompt: string | null;
  variablesSchemaJson: string;
  defaultValuesJson: string;
  parameterPresetJson: string;
  notes: string | null;
  createdAt: string;
};

export type RenderPromptRun = {
  promptVersionId: string;
  promptId: string;
  versionNumber: number;
  versionName: string;
  renderedPrompt: string;
  renderedNegativePrompt: string | null;
  valuesJson: string;
  parameterPresetJson: string;
};

export type PromptOutputHistoryItem = {
  generationEventId: string;
  assetId: string | null;
  outputVersionId: string | null;
  taskId: string | null;
  provider: string;
  providerModel: string;
  status: string;
  promptSnapshot: string;
  createdAt: string;
};

export type LineageEntry = {
  version: Version;
  generationEvent: GenerationEvent | null;
};

export type VersionTreeNode = {
  versionId: string;
  parentVersionId: string | null;
  treeName: string;
  versionNumber: number;
  versionName: string;
  filePath: string;
  createdAt: string;
  provider: string | null;
  modelLabel: string | null;
  generationStatus: string | null;
  children: VersionTreeNode[];
};

export type VersionTreeIssue = {
  kind: string;
  versionId: string | null;
  parentVersionId: string | null;
  message: string;
};

export type PromotedSource = {
  sourceAssetId: string;
  sourceAssetTitle: string | null;
  sourceVersionId: string;
  sourceVersionNumber: number;
  sourceVersionName: string;
  sourceVersionTreeName: string | null;
};

export type PromoteAssetVersionResult = {
  asset: AssetView;
  version: Version;
  promotedFrom: PromotedSource;
};

export type ReferenceSource = {
  assetId: string;
  assetTitle: string | null;
  assetStatus: string;
  versionId: string;
  versionNumber: number;
  versionName: string;
  filePath: string;
};

export type Album = {
  id: string;
  name: string;
  kind: "manual" | "smart";
  count?: number;
};

export type AlbumListItem = {
  id: string;
  name: string;
  kind: "manual" | "smart";
  itemCount: number | null;
  sortOrder?: number;
};

export type ConfidenceScore = {
  overall: number | null;
  title: number | null;
  description: number | null;
  schemaPrompt: number | null;
  tags: number | null;
  category: number | null;
};

export type FileContext = {
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

export type AssetDetail = {
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
  currentVersionId: string | null;
  currentVersionNumber: number | null;
  currentVersionName: string | null;
  focusedVersionId?: string | null;
  focusedVersionTreeName?: string | null;
  focusedVersion?: Version | null;
  versions: Version[];
  versionTree?: VersionTreeNode[];
  versionTreeIssues?: VersionTreeIssue[];
  lineage: LineageEntry[];
  sourceReference?: ReferenceSource | null;
  promotedFrom?: PromotedSource | null;
  file: FileContext | null;
};

export type Suggestion = {
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

export type TaskStatus =
  | "queued"
  | "running"
  | "retry_waiting"
  | "failed_retryable"
  | "failed_final"
  | "cancel_requested"
  | "canceled"
  | "completed"
  | "interrupted_retryable"
  | "interrupted_final";

export type TaskDraft = {
  id: string;
  prompt: string;
  provider: string;
  operation: "text_to_image" | "image_to_image";
  negativePrompt: string;
  inputFile: string;
  inputVersionId: string | null;
  parametersJson: string;
  priority: number;
  maxAttempts: number;
};

export type DaemonTask = {
  id: string;
  libraryId: string;
  taskType: string;
  status: TaskStatus;
  queuePosition: number;
  priority: number;
  provider: string | null;
  operation: string | null;
  concurrencyGroup: string | null;
  attemptCount: number;
  maxAttempts: number;
  nextRetryAt: string | null;
  input: Record<string, unknown> | null;
  createdAt: string;
  updatedAt: string;
  lastErrorCode: string | null;
  lastErrorMessage: string | null;
  errorClassification: string | null;
  waitReason: string | null;
};

export type DaemonTaskAttempt = {
  id: string;
  taskId: string;
  attemptNumber: number;
  status: string;
  startedAt: string;
  completedAt: string | null;
  logPath: string | null;
  exitCode: number | null;
  errorCode: string | null;
  errorMessage: string | null;
  errorClassification: string | null;
};

export type DaemonTaskEvent = {
  id: string;
  taskId: string;
  eventType: string;
  message: string | null;
  payload: Record<string, unknown> | null;
  createdAt: string;
};

export type DaemonTaskOutput = {
  id: string;
  taskId: string;
  outputType: string;
  targetId: string;
  payload: Record<string, unknown> | null;
  createdAt: string;
};

export type DaemonTaskDetail = {
  task: DaemonTask;
  attempts: DaemonTaskAttempt[];
  events: DaemonTaskEvent[];
  outputs: DaemonTaskOutput[];
  logTail: string;
  logTailTruncated: boolean;
};

export type CommandError = {
  code?: string;
  message?: string;
  recoverable?: boolean;
};

export type GeneratedReviewField = {
  field: ReviewFieldName;
  value: string;
  logPath: string;
};

export type AppLog = {
  path: string;
  kind: string;
  modifiedAt: string;
  sizeBytes: number;
  preview: string;
};

export type AppLogContent = {
  path: string;
  content: string;
  truncated: boolean;
};

export type UpdateInfo = {
  version: string;
  date: string | null;
  body: string | null;
};

export type UpdateCheck = {
  currentVersion: string;
  available: boolean;
  update: UpdateInfo | null;
};

export type UpdateState = {
  currentVersion: string;
  lastCheckedAt: string | null;
  checking: boolean;
  installing: boolean;
  pendingRestart: boolean;
  availableUpdate: UpdateInfo | null;
  error: string | null;
  status: "idle" | "checking" | "upToDate" | "available" | "installing" | "pendingRestart" | "error";
};
