import React, { useEffect, useMemo, useState } from "react";
import ReactDOM from "react-dom/client";
import { convertFileSrc, invoke as tauriInvoke } from "@tauri-apps/api/core";
import {
  acceptSuggestionState,
  rejectSuggestionState,
  updateQueueJobStatus,
  type JobStatus,
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

type Asset = {
  id: string;
  title: string | null;
  category: string | null;
  rating: number | null;
  status: string;
  provider: string;
  prompt: string;
  tags: string[];
  albums: string[];
  versions: Version[];
  swatch: string;
  imagePath?: string | null;
};

type GalleryItem = {
  id: string;
  title: string | null;
  category: string | null;
  rating: number | null;
  status: string;
  imagePath: string | null;
  versionId: string | null;
  mimeType: string | null;
};

type Version = {
  id: string;
  assetId: string;
  parentVersionId: string | null;
  generationEventId: string | null;
  filePath: string;
  sha256: string;
  mimeType: string;
};

type Album = {
  id: string;
  name: string;
  kind: "manual" | "smart";
  count: number;
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

const mockLibrary: Library = {
  id: "library-local",
  name: "Local Lab",
  rootPath: "/Users/demo/Pictures/ImagePromptLab",
  hidden: false,
  schemaVersion: 1,
};

const mockVersions: Version[] = [
  {
    id: "version-aurora-1",
    assetId: "asset-aurora",
    parentVersionId: null,
    generationEventId: "event-aurora",
    filePath: "originals/imported/version-aurora-1.png",
    sha256: "43044b9f977ef333aa328b242d0e9ff0f9fed13e1c77abdd5ff12dd8edac5dd5",
    mimeType: "image/png",
  },
  {
    id: "version-city-1",
    assetId: "asset-city",
    parentVersionId: null,
    generationEventId: "event-city",
    filePath: "originals/imported/version-city-1.png",
    sha256: "8f5b6c5fcf05c9be9c981db9215a5e8c71e8dc829ab2c8f51b9a5dfcc766f9a4",
    mimeType: "image/png",
  },
];

const initialAssets: Asset[] = [
  {
    id: "asset-aurora",
    title: "Aurora Archive",
    category: "concept",
    rating: 4,
    status: "imported",
    provider: "codex-cli",
    prompt: "A quiet polar research room lit by green aurora reflections.",
    tags: ["environment", "cinematic", "cold"],
    albums: ["Look Dev"],
    versions: [mockVersions[0]],
    swatch: "#2f6f73",
  },
  {
    id: "asset-city",
    title: "Flesh City",
    category: "study",
    rating: 3,
    status: "generated",
    provider: "codex-cli",
    prompt: "A surreal city street with faceless figures and organic architecture.",
    tags: ["surreal", "urban"],
    albums: ["Experiments"],
    versions: [mockVersions[1]],
    swatch: "#8f4e55",
  },
];

const initialSuggestions: Suggestion[] = [
  {
    id: "suggestion-1",
    assetId: "asset-city",
    title: "Organic Avenue",
    description: "A controlled surreal study for city-scale body horror textures.",
    tags: ["surreal", "architecture", "body-horror"],
    category: "study",
    status: "pending_review",
  },
];

const initialAlbums: Album[] = [
  { id: "album-lookdev", name: "Look Dev", kind: "manual", count: 1 },
  { id: "album-rated", name: "Rated 4+", kind: "smart", count: 1 },
  { id: "album-experiments", name: "Experiments", kind: "manual", count: 1 },
];

const initialQueue: QueueJob[] = [
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
  const [library, setLibrary] = useState<Library | null>(runningInTauri ? null : mockLibrary);
  const [assets, setAssets] = useState<Asset[]>(runningInTauri ? [] : initialAssets);
  const [selectedAssetId, setSelectedAssetId] = useState(runningInTauri ? "" : initialAssets[0]?.id ?? "");
  const [albums, setAlbums] = useState<Album[]>(initialAlbums);
  const [suggestions, setSuggestions] = useState<Suggestion[]>(initialSuggestions);
  const [queue, setQueue] = useState<QueueJob[]>(initialQueue);
  const [prompt, setPrompt] = useState("");
  const [provider, setProvider] = useState("codex-cli");
  const [status, setStatus] = useState(
    runningInTauri ? "Create or open a library before generation" : "Preview mode",
  );
  const [libraryPathInput, setLibraryPathInput] = useState("");
  const [libraryNameInput, setLibraryNameInput] = useState("Image Prompt Lab");

  const selectedAsset = useMemo(
    () => assets.find((asset) => asset.id === selectedAssetId) ?? assets[0] ?? null,
    [assets, selectedAssetId],
  );
  const pendingSuggestions = suggestions.filter((suggestion) => suggestion.status === "pending_review");

  useEffect(() => {
    if (runningInTauri) {
      void refreshLibraries();
    }
  }, [runningInTauri]);

  async function refreshLibraries() {
    try {
      const libraries = await invokeCommand<Library[]>("list_libraries", { includeHidden: false });
      const nextLibrary = libraries[0] ?? null;
      setLibrary(nextLibrary);
      setLibraryPathInput(nextLibrary?.rootPath ?? libraryPathInput);
      if (nextLibrary) {
        await refreshAssets(nextLibrary);
      } else {
        setAssets([]);
        setSelectedAssetId("");
        setStatus("No library registered");
      }
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  async function refreshAssets(nextLibrary = library) {
    if (!nextLibrary) {
      return;
    }

    try {
      const results = await invokeCommand<GalleryItem[]>("gallery_items", {
        input: {
          libraryPath: nextLibrary.rootPath,
        },
      });
      const hydrated = results.map((item, index) => ({
        id: item.id,
        title: item.title,
        category: item.category,
        rating: item.rating,
        status: item.status,
        provider: "unknown",
        prompt: "Prompt history will appear after selecting a generated version.",
        tags: [],
        albums: [],
        versions: item.versionId
          ? [
              {
                id: item.versionId,
                assetId: item.id,
                parentVersionId: null,
                generationEventId: null,
                filePath: item.imagePath ?? "",
                sha256: "",
                mimeType: item.mimeType ?? "image/png",
              },
            ]
          : [],
        swatch: swatchFor(index),
        imagePath: item.imagePath,
      }));
      setAssets(hydrated);
      setSelectedAssetId(hydrated[0]?.id ?? "");
      setStatus(`Gallery refreshed: ${hydrated.length} item${hydrated.length === 1 ? "" : "s"}`);
    } catch (error) {
      setStatus(errorMessage(error));
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
      setLibrary(created);
      setAssets([]);
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
      setLibrary(opened);
      setStatus("Library opened");
      await refreshAssets(opened);
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  async function generateImage() {
    if (!library || prompt.trim().length === 0) {
      setStatus("Create or open a real library before generation");
      return;
    }

    const job: QueueJob = {
      id: crypto.randomUUID(),
      provider,
      prompt,
      status: "running",
    };
    setQueue((current) => [job, ...current]);
    setStatus("Starting generation");

    try {
      const started = await invokeCommand<GenerationJob>("start_generation", {
        input: {
          libraryPath: library.rootPath,
          provider,
          prompt,
          negativePrompt: null,
          inputFile: null,
          inputVersionId: null,
          parametersJson: "{}",
        },
      });
      setQueue((current) =>
        current.map((item) =>
          item.id === job.id
            ? {
                ...item,
                id: started.id,
                status: started.status,
                logPath: started.logPath,
              }
            : item,
        ),
      );
      setQueue((current) =>
        updateQueueJobStatus(current, started.id, "running"),
      );
      setPrompt("");
      setStatus(`Generation running. Log: ${started.logPath}`);
      void pollGenerationJob(started.id);
    } catch (error) {
      setQueue((current) =>
        updateQueueJobStatus(current, job.id, "failed"),
      );
      setStatus(errorMessage(error));
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
              }
            : item,
        ),
      );

      if (job.status === "completed") {
        setStatus(`${job.versions.length} version generated`);
        if (job.versions.length > 0) {
          const generatedAssets = job.versions.map((version, index) => ({
            id: version.assetId,
            title: null,
            category: null,
            rating: null,
            status: "generated",
            provider: job.provider,
            prompt: job.prompt,
            tags: [],
            albums: [],
            versions: [version],
            swatch: swatchFor(index),
            imagePath: resolveLibraryPath(library, version.filePath),
          }));
          setAssets((current) => mergeAssets(generatedAssets, current));
          setSelectedAssetId(generatedAssets[0]?.id ?? selectedAssetId);
        }
        await refreshAssets();
        return;
      }

      if (job.status === "failed") {
        setStatus(`${job.error ?? "Generation failed"}; log: ${job.logPath}`);
        return;
      }

      window.setTimeout(() => {
        void pollGenerationJob(jobId);
      }, 1200);
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  async function acceptSuggestion(suggestion: Suggestion) {
    if (!library) {
      return;
    }

    try {
      const asset = await invokeCommand<Asset>("accept_suggestion", {
        input: {
          libraryPath: library.rootPath,
          suggestionId: suggestion.id,
          title: suggestion.title,
          description: suggestion.description,
          tags: suggestion.tags,
          category: suggestion.category,
        },
      });
      setAssets((current) => {
        const state = acceptSuggestionState(
          current,
          suggestions,
          {
            id: suggestion.id,
            assetId: asset.id,
            title: asset.title,
            description: suggestion.description,
            category: asset.category,
            tags: suggestion.tags,
            status: suggestion.status,
          },
        );
        return state.assets;
      });
      setSuggestions((current) =>
        rejectSuggestionState(current, suggestion.id),
      );
      setStatus("Suggestion accepted");
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  async function rejectSuggestion(suggestion: Suggestion) {
    if (!library) {
      return;
    }

    try {
      await invokeCommand("reject_suggestion", {
        libraryPath: library.rootPath,
        suggestionId: suggestion.id,
      });
      setSuggestions((current) =>
        rejectSuggestionState(current, suggestion.id),
      );
      setStatus("Suggestion rejected");
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  async function updateRating(rating: number) {
    if (!library || !selectedAsset) {
      return;
    }

    try {
      const asset = await invokeCommand<Asset>("update_asset_metadata", {
        input: {
          libraryPath: library.rootPath,
          assetId: selectedAsset.id,
          rating,
          category: selectedAsset.category,
          status: selectedAsset.status,
        },
      });
      setAssets((current) =>
        current.map((item) => (item.id === asset.id ? { ...item, rating: asset.rating } : item)),
      );
      setStatus("Rating updated");
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  return (
    <main className="workbench">
      <aside className="sidebar">
        <div className="brand">
          <h1>Image Prompt Lab</h1>
          <span>{library ? library.name : "No library"}</span>
        </div>
        <nav className="nav">
          <NavButton active={activeView === "gallery"} onClick={() => setActiveView("gallery")} label="Gallery" />
          <NavButton active={activeView === "albums"} onClick={() => setActiveView("albums")} label="Albums" />
          <NavButton active={activeView === "review"} onClick={() => setActiveView("review")} label="Review Inbox" count={pendingSuggestions.length} />
          <NavButton active={activeView === "queue"} onClick={() => setActiveView("queue")} label="Generation Queue" count={queue.length} />
          <NavButton active={activeView === "settings"} onClick={() => setActiveView("settings")} label="Settings" />
        </nav>
        <div className="library-panel">
          <button onClick={refreshLibraries}>Refresh</button>
          <button onClick={() => refreshAssets()}>Sync Gallery</button>
        </div>
      </aside>

      <section className="workspace">
        <header className="workspace-header">
          <div>
            <h2>{viewTitle(activeView)}</h2>
            <p>{status}</p>
          </div>
          <div className="toolbar">
            <button onClick={() => setActiveView("gallery")}>Grid</button>
            <button onClick={() => setActiveView("review")}>Review</button>
          </div>
        </header>

        <GenerationComposer
          prompt={prompt}
          provider={provider}
          onPromptChange={setPrompt}
          onProviderChange={setProvider}
          onGenerate={generateImage}
        />

        {activeView === "gallery" && (
          <Gallery assets={assets} selectedAssetId={selectedAsset?.id ?? ""} onSelect={setSelectedAssetId} />
        )}
        {activeView === "albums" && <Albums albums={albums} setAlbums={setAlbums} />}
        {activeView === "review" && (
          <ReviewInbox suggestions={pendingSuggestions} onAccept={acceptSuggestion} onReject={rejectSuggestion} />
        )}
        {activeView === "queue" && <GenerationQueue queue={queue} />}
        {activeView === "settings" && <Settings library={library} />}
        {activeView === "settings" && (
          <LibrarySettings
            libraryPath={libraryPathInput}
            libraryName={libraryNameInput}
            onLibraryPathChange={setLibraryPathInput}
            onLibraryNameChange={setLibraryNameInput}
            onCreate={createLibrary}
            onOpen={openLibrary}
          />
        )}
      </section>

      <Inspector asset={selectedAsset} onUpdateRating={updateRating} />
    </main>
  );
}

function NavButton({
  active,
  onClick,
  label,
  count,
}: {
  active: boolean;
  onClick: () => void;
  label: string;
  count?: number;
}) {
  return (
    <button className={active ? "nav-button active" : "nav-button"} onClick={onClick}>
      <span>{label}</span>
      {typeof count === "number" && <strong>{count}</strong>}
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
      <select value={provider} onChange={(event) => onProviderChange(event.target.value)}>
        <option value="codex-cli">codex-cli</option>
        <option value="fake">fake</option>
      </select>
      <input
        value={prompt}
        onChange={(event) => onPromptChange(event.target.value)}
        placeholder="Prompt"
      />
      <button disabled={prompt.trim().length === 0} onClick={onGenerate}>
        Generate
      </button>
    </section>
  );
}

function Gallery({
  assets,
  selectedAssetId,
  onSelect,
}: {
  assets: Asset[];
  selectedAssetId: string;
  onSelect: (id: string) => void;
}) {
  return (
    <section className="gallery-grid">
      {assets.map((asset) => (
        <button
          key={asset.id}
          className={asset.id === selectedAssetId ? "asset-tile selected" : "asset-tile"}
          onClick={() => onSelect(asset.id)}
        >
          <span className="thumb" style={{ backgroundColor: asset.swatch }} />
          {asset.imagePath && (
            <img
              alt={asset.title ?? "Generated image"}
              className="thumb-image"
              src={convertImagePath(asset.imagePath)}
            />
          )}
          <span className="asset-title">{asset.title ?? "Untitled"}</span>
          <span className="asset-meta">
            {asset.provider} · {asset.rating ?? "-"} / 5
          </span>
        </button>
      ))}
    </section>
  );
}

function Albums({ albums, setAlbums }: { albums: Album[]; setAlbums: (albums: Album[]) => void }) {
  return (
    <section className="list-view">
      <button
        className="row-action"
        onClick={() =>
          setAlbums([
            ...albums,
            {
              id: crypto.randomUUID(),
              name: `Manual Album ${albums.length + 1}`,
              kind: "manual",
              count: 0,
            },
          ])
        }
      >
        New Manual Album
      </button>
      {albums.map((album) => (
        <article className="list-row" key={album.id}>
          <div>
            <h3>{album.name}</h3>
            <p>{album.kind}</p>
          </div>
          <strong>{album.count}</strong>
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
            {job.logPath && <p className="log-path">{job.logPath}</p>}
            {job.error && <p className="error-text">{job.error}</p>}
          </div>
          <span className={`status ${job.status}`}>{job.status}</span>
        </article>
      ))}
    </section>
  );
}

function Settings({ library }: { library: Library | null }) {
  return (
    <section className="settings-grid">
      <div>
        <h3>Library</h3>
        <p>{library?.rootPath ?? "Not opened"}</p>
      </div>
      <div>
        <h3>Provider</h3>
        <p>codex-cli imagegen skill</p>
      </div>
      <div>
        <h3>Schema</h3>
        <p>{library?.schemaVersion ?? "-"}</p>
      </div>
    </section>
  );
}

function LibrarySettings({
  libraryPath,
  libraryName,
  onLibraryPathChange,
  onLibraryNameChange,
  onCreate,
  onOpen,
}: {
  libraryPath: string;
  libraryName: string;
  onLibraryPathChange: (value: string) => void;
  onLibraryNameChange: (value: string) => void;
  onCreate: () => void;
  onOpen: () => void;
}) {
  return (
    <section className="library-form">
      <label>
        <span>Library path</span>
        <input
          value={libraryPath}
          onChange={(event) => onLibraryPathChange(event.target.value)}
          placeholder="/Users/you/Pictures/image-prompt-lab"
        />
      </label>
      <label>
        <span>Library name</span>
        <input
          value={libraryName}
          onChange={(event) => onLibraryNameChange(event.target.value)}
          placeholder="Image Prompt Lab"
        />
      </label>
      <div className="row-actions">
        <button onClick={onCreate}>Create Library</button>
        <button onClick={onOpen}>Open Library</button>
      </div>
    </section>
  );
}

function Inspector({
  asset,
  onUpdateRating,
}: {
  asset: Asset | null;
  onUpdateRating: (rating: number) => void;
}) {
  if (!asset) {
    return (
      <aside className="inspector">
        <h2>Inspector</h2>
        <div className="empty-state compact">No asset selected.</div>
      </aside>
    );
  }

  return (
    <aside className="inspector">
      <h2>Inspector</h2>
      <div className="preview" style={{ backgroundColor: asset.swatch }}>
        {asset.imagePath && (
          <img
            alt={asset.title ?? "Generated image preview"}
            className="preview-image"
            src={convertImagePath(asset.imagePath)}
          />
        )}
      </div>
      <section className="inspector-section">
        <h3>{asset.title ?? "Untitled"}</h3>
        <p>{asset.prompt}</p>
      </section>
      <section className="inspector-section">
        <h3>Rating</h3>
        <div className="rating-row">
          {[1, 2, 3, 4, 5].map((rating) => (
            <button
              key={rating}
              className={asset.rating === rating ? "rating active" : "rating"}
              onClick={() => onUpdateRating(rating)}
            >
              {rating}
            </button>
          ))}
        </div>
      </section>
      <section className="inspector-section">
        <h3>Tags</h3>
        <div className="tag-list">
          {asset.tags.map((tag) => (
            <span key={tag}>{tag}</span>
          ))}
        </div>
      </section>
      <section className="inspector-section">
        <h3>Versions</h3>
        {asset.versions.map((version) => (
          <p className="version-line" key={version.id}>
            {version.id} · {version.mimeType}
          </p>
        ))}
      </section>
    </aside>
  );
}

function viewTitle(view: View) {
  switch (view) {
    case "gallery":
      return "Gallery";
    case "albums":
      return "Albums";
    case "review":
      return "Review Inbox";
    case "queue":
      return "Generation Queue";
    case "settings":
      return "Settings";
  }
}

function errorMessage(error: unknown) {
  if (typeof error === "object" && error && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return String(error);
}

function convertImagePath(path: string) {
  if (!hasTauriRuntime()) {
    return path;
  }
  return convertFileSrc(path);
}

function resolveLibraryPath(library: Library | null, filePath: string) {
  if (filePath.length === 0 || filePath.startsWith("/")) {
    return filePath;
  }
  if (!library) {
    return filePath;
  }
  return `${library.rootPath.replace(/\/+$/, "")}/${filePath.replace(/^\/+/, "")}`;
}

function mergeAssets(primary: Asset[], secondary: Asset[]) {
  const seen = new Set<string>();
  const merged: Asset[] = [];
  for (const asset of [...primary, ...secondary]) {
    if (seen.has(asset.id)) {
      continue;
    }
    seen.add(asset.id);
    merged.push(asset);
  }
  return merged;
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

function swatchFor(index: number) {
  const colors = ["#2f6f73", "#8f4e55", "#6b705c", "#345995", "#806443", "#5f4b8b"];
  return colors[index % colors.length];
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
