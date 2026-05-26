import React, { useMemo, useState } from "react";
import type { AlbumListItem, Library, ScheduledGenerationJob, ScheduledGenerationRun, ScheduleRule } from "../../types";
import { defaultScheduleDraftForProvider, type ScheduleDraft } from "../../workflows/schedules/state";

export function SchedulesWorkspace({
  library,
  albums,
  jobs,
  runs,
  selectedJobId,
  defaultProvider,
  loading,
  onSelectJob,
  onRefresh,
  onCreateJob,
  onUpdateJob,
  onDuplicateJob,
  onDeleteJob,
  onRunNow,
  onToggleJob,
  onOpenTask,
}: {
  library: Library | null;
  albums: AlbumListItem[];
  jobs: ScheduledGenerationJob[];
  runs: ScheduledGenerationRun[];
  selectedJobId: string | null;
  defaultProvider: string;
  loading: boolean;
  onSelectJob: (jobId: string) => void;
  onRefresh: () => void;
  onCreateJob: (draft: ScheduleDraft) => void;
  onUpdateJob: (jobId: string, draft: ScheduleDraft) => void;
  onDuplicateJob: (job: ScheduledGenerationJob) => void;
  onDeleteJob: (jobId: string) => void;
  onRunNow: (jobId: string) => void;
  onToggleJob: (job: ScheduledGenerationJob) => void;
  onOpenTask: (taskId: string) => void;
}) {
  const manualAlbums = albums.filter((album) => album.kind === "manual");
  const [draft, setDraft] = useState<ScheduleDraft>(() =>
    defaultScheduleDraftForProvider(defaultProvider, manualAlbums[0]?.id ?? ""),
  );
  const [editingJobId, setEditingJobId] = useState<string | null>(null);
  const selectedJob = jobs.find((job) => job.id === selectedJobId) ?? jobs[0] ?? null;
  const editingJob = editingJobId ? jobs.find((job) => job.id === editingJobId) ?? null : null;
  const selectedRuns = useMemo(
    () => runs.filter((run) => run.jobId === selectedJob?.id),
    [runs, selectedJob?.id],
  );

  function updateDraft(patch: Partial<ScheduleDraft>) {
    setDraft((current) => ({ ...current, ...patch }));
  }

  function loadJobIntoDraft(job: ScheduledGenerationJob) {
    setEditingJobId(job.id);
    setDraft({
      name: job.name,
      promptMode: job.promptMode,
      fixedPrompt: job.fixedPrompt ?? "",
      basePrompt: job.basePrompt ?? "",
      dynamicPrompt: job.dynamicPrompt ?? "",
      targetAlbumId: job.targetAlbumId,
      tags: job.tags.join(", "),
      imageProvider: job.imageProvider,
      imageModel: job.imageModel,
      promptExpanderProvider: job.promptExpanderProvider ?? "codex-cli",
      promptExpanderModel: job.promptExpanderModel ?? "codex",
      scheduleKind: job.scheduleRule.kind,
      minutes: job.scheduleRule.minutes ?? 30,
      hours: job.scheduleRule.hours ?? 6,
      localTimeHhMm: job.scheduleRule.localTimeHhMm ?? "09:00",
    });
  }

  function resetDraftForNewJob() {
    setEditingJobId(null);
    setDraft(defaultScheduleDraftForProvider(defaultProvider, manualAlbums[0]?.id ?? ""));
  }

  return (
    <section className="workflow-grid schedules-workspace">
      <section className="task-panel">
        <div className="panel-heading">
          <div>
            <h3>Scheduled Jobs</h3>
            <p>{jobs.length} jobs</p>
          </div>
          <button onClick={onRefresh}>{loading ? "Refreshing" : "Refresh"}</button>
        </div>
        {!library ? (
          <div className="empty-state compact">Open a real library before creating schedules.</div>
        ) : jobs.length === 0 ? (
          <div className="empty-state compact">No scheduled jobs yet.</div>
        ) : (
          <div className="task-list">
            {jobs.map((job) => (
              <button
                key={job.id}
                className={job.id === selectedJob?.id ? "task-row schedule-job-row selected" : "task-row schedule-job-row"}
                onClick={() => onSelectJob(job.id)}
              >
                <span className="task-row-main">
                  <strong>{job.name}</strong>
                  <small>{job.imageProvider} · {job.promptMode} · {formatScheduleRule(job.scheduleRule)}</small>
                </span>
                <span className={`status-pill ${job.status}`}>{job.status}</span>
              </button>
            ))}
          </div>
        )}
      </section>

      <section className="task-panel">
        <div className="panel-heading">
          <div>
            <h3>{editingJob ? "Edit Schedule" : "Create Schedule"}</h3>
            <p>{editingJob ? `Editing ${editingJob.name}` : "Fixed or dynamic prompt, with selected job editing"}</p>
          </div>
          {editingJob && <button onClick={resetDraftForNewJob}>New schedule</button>}
        </div>
        <div className="form-grid compact-form">
          <label>
            <span>Name</span>
            <input value={draft.name} onChange={(event) => updateDraft({ name: event.target.value })} />
          </label>
          <label>
            <span>Prompt mode</span>
            <select className="select-control" value={draft.promptMode} onChange={(event) => updateDraft({ promptMode: event.target.value as ScheduleDraft["promptMode"] })}>
              <option value="fixed">Fixed prompt</option>
              <option value="dynamic">Dynamic prompt</option>
            </select>
          </label>
          {draft.promptMode === "fixed" ? (
            <label className="span-2">
              <span>Prompt</span>
              <textarea value={draft.fixedPrompt} onChange={(event) => updateDraft({ fixedPrompt: event.target.value })} />
            </label>
          ) : (
            <>
              <label className="span-2">
                <span>Base prompt</span>
                <textarea value={draft.basePrompt} onChange={(event) => updateDraft({ basePrompt: event.target.value })} />
              </label>
              <label className="span-2">
                <span>Dynamic prompt</span>
                <textarea value={draft.dynamicPrompt} onChange={(event) => updateDraft({ dynamicPrompt: event.target.value })} />
              </label>
            </>
          )}
          <label>
            <span>Image provider</span>
            <select
              className="select-control"
              value={draft.imageProvider}
              onChange={(event) => {
                const nextProvider = event.target.value;
                updateDraft({
                  imageProvider: nextProvider,
                  imageModel: nextProvider === "fake" ? "fake" : draft.imageModel === "fake" ? "codex" : draft.imageModel || "codex",
                });
              }}
            >
              <option value="codex-cli">codex-cli</option>
              <option value="fake">fake</option>
            </select>
          </label>
          <label>
            <span>Image model</span>
            <input value={draft.imageModel} onChange={(event) => updateDraft({ imageModel: event.target.value })} />
          </label>
          {draft.promptMode === "dynamic" && (
            <>
              <label>
                <span>Prompt expander</span>
                <select
                  className="select-control"
                  value={draft.promptExpanderProvider}
                  onChange={(event) => {
                    const nextProvider = event.target.value;
                    updateDraft({
                      promptExpanderProvider: nextProvider,
                      promptExpanderModel: nextProvider === "fake" ? "fake" : draft.promptExpanderModel === "fake" ? "codex" : draft.promptExpanderModel || "codex",
                    });
                  }}
                >
                  <option value="codex-cli">codex-cli</option>
                  <option value="fake">fake</option>
                </select>
              </label>
              <label>
                <span>Expander model</span>
                <input value={draft.promptExpanderModel} onChange={(event) => updateDraft({ promptExpanderModel: event.target.value })} />
              </label>
            </>
          )}
          <label>
            <span>Schedule</span>
            <select className="select-control" value={draft.scheduleKind} onChange={(event) => updateDraft({ scheduleKind: event.target.value as ScheduleRule["kind"] })}>
              <option value="interval_minutes">Every N minutes</option>
              <option value="interval_hours">Every N hours</option>
              <option value="daily_time">Daily time</option>
            </select>
          </label>
          {draft.scheduleKind === "interval_minutes" && (
            <label>
              <span>Minutes</span>
              <input type="number" min={1} value={draft.minutes} onChange={(event) => updateDraft({ minutes: Number(event.target.value) })} />
            </label>
          )}
          {draft.scheduleKind === "interval_hours" && (
            <label>
              <span>Hours</span>
              <input type="number" min={1} value={draft.hours} onChange={(event) => updateDraft({ hours: Number(event.target.value) })} />
            </label>
          )}
          {draft.scheduleKind === "daily_time" && (
            <label>
              <span>Time</span>
              <input value={draft.localTimeHhMm} onChange={(event) => updateDraft({ localTimeHhMm: event.target.value })} />
            </label>
          )}
          <label>
            <span>Album</span>
            <select className="select-control" value={draft.targetAlbumId} onChange={(event) => updateDraft({ targetAlbumId: event.target.value })}>
              <option value="">Select album</option>
              {manualAlbums.map((album) => (
                <option key={album.id} value={album.id}>{album.name}</option>
              ))}
            </select>
          </label>
          <label>
            <span>Tags</span>
            <input value={draft.tags} onChange={(event) => updateDraft({ tags: event.target.value })} placeholder="comma,separated" />
          </label>
          {editingJob ? (
            <>
              <button className="primary-button span-2" disabled={!library || !draft.targetAlbumId} onClick={() => onUpdateJob(editingJob.id, draft)}>
                Update schedule
              </button>
              <button className="span-2" onClick={resetDraftForNewJob}>
                New schedule
              </button>
            </>
          ) : (
            <button className="primary-button span-2" disabled={!library || !draft.targetAlbumId} onClick={() => onCreateJob(draft)}>
              Create schedule
            </button>
          )}
        </div>
      </section>

      <section className="task-panel task-detail-panel">
        {!selectedJob ? (
          <div className="empty-state compact">Select a schedule.</div>
        ) : (
          <>
            <div className="panel-heading">
              <div>
                <h3>{selectedJob.name}</h3>
                <p>{selectedJob.status} · next {formatTimestamp(selectedJob.nextRunAt)}</p>
              </div>
              <div className="button-row">
                <button onClick={() => loadJobIntoDraft(selectedJob)}>Edit</button>
                <button onClick={() => onToggleJob(selectedJob)}>{selectedJob.status === "active" ? "Pause" : "Enable"}</button>
                <button onClick={() => onRunNow(selectedJob.id)}>Run now</button>
                <button onClick={() => onDuplicateJob(selectedJob)}>Duplicate</button>
                <button onClick={() => onDeleteJob(selectedJob.id)}>Delete</button>
              </div>
            </div>
            <div className="meta-grid">
              <span>Prompt mode</span>
              <strong>{selectedJob.promptMode}</strong>
              <span>Image provider</span>
              <strong>{selectedJob.imageProvider}</strong>
              <span>Image model</span>
              <strong>{selectedJob.imageModel}</strong>
              <span>Schedule</span>
              <strong>{formatScheduleRule(selectedJob.scheduleRule)}</strong>
              <span>Album</span>
              <strong>{manualAlbums.find((album) => album.id === selectedJob.targetAlbumId)?.name ?? selectedJob.targetAlbumId}</strong>
              <span>Tags</span>
              <strong>{selectedJob.tags.length ? selectedJob.tags.join(", ") : "-"}</strong>
            </div>
            <h4>Run History</h4>
            {selectedRuns.length === 0 ? (
              <p>No runs yet.</p>
            ) : (
              selectedRuns.map((run) => (
                <div className="task-output-row" key={run.id}>
                  <span>
                    <strong>{run.status}</strong>
                    <small>{formatTimestamp(run.scheduledFor)} · {run.outputAssetCount} output(s)</small>
                  </span>
                  {run.imageTaskId && <button onClick={() => onOpenTask(run.imageTaskId!)}>Open task</button>}
                </div>
              ))
            )}
          </>
        )}
      </section>
    </section>
  );
}

export type { ScheduleDraft };

function formatScheduleRule(rule: ScheduleRule): string {
  if (rule.kind === "interval_minutes") {
    return `every ${rule.minutes ?? "-"} min`;
  }
  if (rule.kind === "interval_hours") {
    return `every ${rule.hours ?? "-"} h`;
  }
  return `daily ${rule.localTimeHhMm ?? "-"}`;
}

function formatTimestamp(value: string): string {
  const millis = Number(value);
  if (!Number.isFinite(millis) || millis <= 0) {
    return value;
  }
  return new Date(millis).toLocaleString();
}
