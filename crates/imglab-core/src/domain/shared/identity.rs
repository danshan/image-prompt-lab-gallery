#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetVersionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationEventId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataSuggestionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptVersionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskAttemptId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskEventId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOutputId(pub String);
