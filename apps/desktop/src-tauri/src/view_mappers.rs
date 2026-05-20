use crate::*;

pub(crate) fn library_view(summary: imglab_core::LibrarySummary) -> LibraryView {
    let root_path = expand_home_path(summary.root_path.clone()).unwrap_or(summary.root_path);
    LibraryView {
        id: summary.id.0,
        name: summary.name,
        root_path,
        hidden: summary.hidden,
        schema_version: summary.schema_version,
    }
}

pub(crate) fn asset_view(summary: imglab_core::AssetSummary) -> AssetView {
    AssetView {
        id: summary.id.0,
        title: summary.title,
        category: summary.category,
        rating: summary.rating,
        status: summary.status,
    }
}

pub(crate) fn gallery_query_from_input(
    input: QueryGalleryInput,
) -> Result<GalleryQuery, CommandError> {
    Ok(GalleryQuery {
        text: input.text,
        providers: input.providers.unwrap_or_default(),
        min_rating: input.min_rating,
        review_status: review_status_from_input(input.review_status.as_deref())?,
        tags: input.tags.unwrap_or_default(),
        album_id: input.album_id.map(imglab_core::AlbumId),
        sort: gallery_sort_from_input(input.sort.as_deref())?,
    })
}

pub(crate) fn review_status_from_input(
    value: Option<&str>,
) -> Result<ReviewStatusFilter, CommandError> {
    match value.unwrap_or("any") {
        "any" => Ok(ReviewStatusFilter::Any),
        "pending" | "pending_review" => Ok(ReviewStatusFilter::Pending),
        other => Err(CommandError {
            code: "InvalidGalleryQuery".to_string(),
            message: format!("unsupported review status filter: {other}"),
            recoverable: true,
        }),
    }
}

pub(crate) fn gallery_sort_from_input(value: Option<&str>) -> Result<GallerySort, CommandError> {
    match value.unwrap_or("newest") {
        "newest" => Ok(GallerySort::Newest),
        "oldest" => Ok(GallerySort::Oldest),
        "rating_desc" | "ratingDesc" => Ok(GallerySort::RatingDesc),
        "title_asc" | "titleAsc" => Ok(GallerySort::TitleAsc),
        "provider_asc" | "providerAsc" => Ok(GallerySort::ProviderAsc),
        "album_order" | "albumOrder" => Ok(GallerySort::AlbumOrder),
        other => Err(CommandError {
            code: "InvalidGalleryQuery".to_string(),
            message: format!("unsupported gallery sort: {other}"),
            recoverable: true,
        }),
    }
}

pub(crate) fn gallery_asset_view(
    library_path: &Path,
    summary: imglab_core::GalleryAssetView,
) -> GalleryAssetView {
    GalleryAssetView {
        id: summary.id.0,
        title: summary.title,
        category: summary.category,
        rating: summary.rating,
        status: summary.status,
        provider: summary.provider,
        model_label: summary.model_label,
        prompt: summary.prompt,
        tags: summary.tags,
        review_pending_count: summary.review_pending_count,
        current_version_id: summary.current_version_id.map(|id| id.0),
        current_version_number: summary.current_version_number,
        current_version_name: summary.current_version_name,
        image_path: summary
            .image_path
            .map(|path| absolutize_library_path(library_path, path)),
        width: summary.width,
        height: summary.height,
        version_label: summary.version_label,
        version_count: summary.version_count,
        task_origin: summary.task_origin.map(task_origin_view),
        albums: summary
            .albums
            .into_iter()
            .map(album_membership_view)
            .collect(),
        album_context: summary.album_context.map(album_membership_view),
        created_at: summary.created_at,
        updated_at: summary.updated_at,
    }
}

pub(crate) fn task_origin_view(origin: imglab_core::TaskOriginView) -> TaskOriginView {
    TaskOriginView {
        task_id: origin.task_id.0,
        task_type: task_type_value(origin.task_type),
        status: task_status_value(origin.status),
        provider: origin.provider,
        operation: origin.operation.map(operation_value),
    }
}

pub(crate) fn task_type_value(task_type: imglab_core::TaskType) -> String {
    task_type.as_str().to_string()
}

pub(crate) fn task_status_value(status: imglab_core::TaskStatus) -> String {
    status.as_str().to_string()
}

pub(crate) fn operation_value(operation: GenerationOperation) -> String {
    match operation {
        GenerationOperation::TextToImage => "text_to_image",
        GenerationOperation::ImageToImage => "image_to_image",
    }
    .to_string()
}

pub(crate) fn version_view(summary: imglab_core::VersionSummary) -> VersionView {
    VersionView {
        id: summary.id.0,
        asset_id: summary.asset_id.0,
        parent_version_id: summary.parent_version_id.map(|id| id.0),
        generation_event_id: summary.generation_event_id.map(|id| id.0),
        version_number: summary.version_number,
        version_name: summary.version_name,
        file_path: summary.file_path,
        checksum_algorithm: summary.checksum_algorithm,
        checksum: summary.checksum,
        mime_type: summary.mime_type,
    }
}

pub(crate) fn generation_event_view(
    summary: imglab_core::GenerationEventSummary,
) -> GenerationEventView {
    GenerationEventView {
        id: summary.id.0,
        asset_id: summary.asset_id.map(|id| id.0),
        output_version_id: summary.output_version_id.map(|id| id.0),
        provider: summary.provider,
        provider_model: summary.provider_model,
        operation_type: operation_value(summary.operation_type),
        prompt: summary.prompt,
        parameters_json: summary.parameters_json,
        status: summary.status,
    }
}

pub(crate) fn studio_overview_view(summary: imglab_core::StudioOverviewView) -> StudioOverviewView {
    StudioOverviewView {
        library: library_view(summary.library),
        status: library_status_view(summary.status),
        registered_library_count: summary.registered_library_count,
        missing_library_count: summary.missing_library_count,
        review_pending_count: summary.review_pending_count,
        task_summary: StudioTaskSummaryView {
            active_count: summary.task_summary.active_count,
            queued_count: summary.task_summary.queued_count,
            running_count: summary.task_summary.running_count,
            retry_waiting_count: summary.task_summary.retry_waiting_count,
            failed_count: summary.task_summary.failed_count,
        },
        provider_health: summary
            .provider_health
            .into_iter()
            .map(provider_health_summary_view)
            .collect(),
    }
}

pub(crate) fn provider_health_summary_view(
    summary: imglab_core::ProviderHealthSummaryView,
) -> ProviderHealthSummaryView {
    ProviderHealthSummaryView {
        provider: summary.provider,
        display_name: summary.display_name,
        availability: summary.availability,
        credential_state: summary.credential_state,
        supported_operations: summary
            .supported_operations
            .into_iter()
            .map(operation_value)
            .collect(),
        recoverable_error: summary.recoverable_error,
    }
}

pub(crate) fn diagnostics_overview_view(
    summary: imglab_core::DiagnosticsOverviewView,
) -> DiagnosticsOverviewView {
    DiagnosticsOverviewView {
        provider_health: summary
            .provider_health
            .into_iter()
            .map(provider_health_summary_view)
            .collect(),
        daemon_status: DaemonStatusView {
            state: summary.daemon_status.state,
            recoverable_error: summary.daemon_status.recoverable_error,
        },
        library_status: library_status_view(summary.library_status),
        library_count: summary.library_count,
        missing_library_count: summary.missing_library_count,
    }
}

pub(crate) fn album_membership_view(album: imglab_core::AlbumMembershipView) -> AlbumView {
    AlbumView {
        id: album.id.0,
        name: album.name,
        kind: match album.kind {
            imglab_core::AlbumKind::Manual => "manual",
            imglab_core::AlbumKind::Smart => "smart",
        }
        .to_string(),
    }
}

pub(crate) fn asset_detail_view(
    summary: imglab_core::AssetDetailView,
    library_path: &Path,
) -> AssetDetailView {
    AssetDetailView {
        id: summary.id.0,
        title: summary.title,
        description: summary.description,
        schema_prompt: summary.schema_prompt,
        category: summary.category,
        rating: summary.rating,
        status: summary.status,
        created_at: summary.created_at,
        updated_at: summary.updated_at,
        prompt: summary.prompt,
        negative_prompt: summary.negative_prompt,
        provider: summary.provider,
        model_label: summary.model_label,
        parameters_json: summary.parameters_json,
        tags: summary.tags,
        albums: summary
            .albums
            .into_iter()
            .map(|album| AlbumView {
                id: album.id.0,
                name: album.name,
                kind: match album.kind {
                    imglab_core::AlbumKind::Manual => "manual",
                    imglab_core::AlbumKind::Smart => "smart",
                }
                .to_string(),
            })
            .collect(),
        review_pending_count: summary.review_pending_count,
        current_version_id: summary.current_version_id.map(|id| id.0),
        current_version_number: summary.current_version_number,
        current_version_name: summary.current_version_name,
        versions: summary
            .versions
            .into_iter()
            .map(|version| version_view_with_library_path(library_path, version))
            .collect(),
        lineage: summary
            .lineage
            .into_iter()
            .map(|entry| LineageEntryView {
                version: version_view_with_library_path(library_path, entry.version),
                generation_event: entry.generation_event.map(generation_event_view),
            })
            .collect(),
        source_reference: summary
            .source_reference
            .map(|source| reference_source_view(source, library_path)),
        file: summary.file.map(|file| FileContextView {
            filename: file.filename,
            relative_location: file.relative_location,
            mime_type: file.mime_type,
            size_bytes: file.size_bytes,
            width: file.width,
            height: file.height,
            checksum_algorithm: file.checksum_algorithm,
            checksum: file.checksum,
            integrity_status: file.integrity_status,
        }),
    }
}

pub(crate) fn reference_source_view(
    summary: imglab_core::ReferenceSourceView,
    library_path: &Path,
) -> ReferenceSourceView {
    ReferenceSourceView {
        asset_id: summary.asset_id.0,
        asset_title: summary.asset_title,
        asset_status: summary.asset_status,
        version_id: summary.version_id.0,
        version_number: summary.version_number,
        version_name: summary.version_name,
        file_path: absolutize_library_path(library_path, summary.file_path),
    }
}

pub(crate) fn asset_inspector_detail_view(
    summary: imglab_core::AssetInspectorDetailView,
    library_path: &Path,
) -> AssetInspectorDetailView {
    AssetInspectorDetailView {
        asset: asset_detail_view(summary.asset, library_path),
        canonical_metadata: CanonicalMetadataView {
            title: summary.canonical_metadata.title,
            description: summary.canonical_metadata.description,
            schema_prompt: summary.canonical_metadata.schema_prompt,
            category: summary.canonical_metadata.category,
            rating: summary.canonical_metadata.rating,
            tags: summary.canonical_metadata.tags,
            status: summary.canonical_metadata.status,
        },
        pending_suggestions: summary
            .pending_suggestions
            .into_iter()
            .map(|suggestion| PendingSuggestionSummaryView {
                id: suggestion.id.0,
                asset_id: suggestion.asset_id.0,
                title: suggestion.title,
                category: suggestion.category,
                tag_count: suggestion.tag_count,
                created_at: suggestion.created_at,
            })
            .collect(),
        generated_task_origin: summary.generated_task_origin.map(task_origin_view),
    }
}

pub(crate) fn library_status_view(summary: imglab_core::LibraryStatusView) -> LibraryStatusView {
    LibraryStatusView {
        storage_size_bytes: summary.storage_size_bytes,
        integrity_status: summary.integrity_status,
        integrity_issue_count: summary.integrity_issue_count,
    }
}

pub(crate) fn repair_summary_view(summary: imglab_core::RepairSummary) -> RepairSummaryView {
    RepairSummaryView {
        dry_run: summary.dry_run,
        scanned_versions: summary.scanned_versions,
        files_moved: summary.files_moved,
        paths_updated: summary.paths_updated,
        checksums_updated: summary.checksums_updated,
        dimensions_updated: summary.dimensions_updated,
        issues: summary
            .issues
            .into_iter()
            .map(|issue| RepairIssueView {
                version_id: issue.version_id.0,
                path: issue.path,
                message: issue.message,
            })
            .collect(),
    }
}

pub(crate) fn absolutize_library_path(library_path: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        library_path.join(path)
    }
}

pub(crate) fn version_view_with_library_path(
    library_path: &Path,
    summary: imglab_core::VersionSummary,
) -> VersionView {
    let mut view = version_view(summary);
    if view.file_path.is_relative() {
        view.file_path = library_path.join(&view.file_path);
    }
    view
}

pub(crate) fn album_view(summary: imglab_core::AlbumSummary) -> AlbumView {
    AlbumView {
        id: summary.id.0,
        name: summary.name,
        kind: match summary.kind {
            imglab_core::AlbumKind::Manual => "manual",
            imglab_core::AlbumKind::Smart => "smart",
        }
        .to_string(),
    }
}

pub(crate) fn album_list_item_view(item: imglab_core::AlbumListItem) -> AlbumListItemView {
    AlbumListItemView {
        id: item.id.0,
        name: item.name,
        kind: match item.kind {
            imglab_core::AlbumKind::Manual => "manual",
            imglab_core::AlbumKind::Smart => "smart",
        }
        .to_string(),
        item_count: item.item_count,
        sort_order: item.sort_order,
    }
}

pub(crate) fn suggestion_view(summary: imglab_core::MetadataSuggestion) -> SuggestionView {
    let confidence = service().normalize_confidence(&summary.confidence_json);
    SuggestionView {
        id: summary.id.0,
        asset_id: summary.asset_id.0,
        title: summary.suggested_title,
        description: summary.suggested_description,
        schema_prompt: summary.suggested_schema_prompt,
        tags: summary.suggested_tags,
        category: summary.suggested_category,
        status: summary.status,
        confidence_json: summary.confidence_json,
        created_at: summary.created_at,
        reviewed_at: summary.reviewed_at,
        confidence: confidence_score_view(confidence),
    }
}

pub(crate) fn review_draft_detail_view(
    summary: imglab_core::ReviewDraftDetailView,
    library_path: &Path,
) -> ReviewDraftDetailView {
    ReviewDraftDetailView {
        suggestion: suggestion_view(summary.suggestion),
        draft_seed: ReviewDraftSeedView {
            title: summary.draft_seed.title,
            description: summary.draft_seed.description,
            schema_prompt: summary.draft_seed.schema_prompt,
            tags: summary.draft_seed.tags,
            category: summary.draft_seed.category,
        },
        confidence: confidence_score_view(summary.confidence),
        history: summary.history.into_iter().map(suggestion_view).collect(),
        generated_field_results: summary
            .generated_field_results
            .into_iter()
            .map(|result| GeneratedReviewFieldResultView {
                task_id: result.task_id.0,
                field: result.field,
                value: result.value,
                base_revision: result.base_revision,
                created_at: result.created_at,
            })
            .collect(),
        related_tasks: summary
            .related_tasks
            .into_iter()
            .map(|task| RelatedTaskSummaryView {
                id: task.id.0,
                task_type: task_type_value(task.task_type),
                status: task_status_value(task.status),
                provider: task.provider,
                operation: task.operation.map(operation_value),
            })
            .collect(),
        asset: asset_detail_view(summary.asset, library_path),
    }
}

pub(crate) fn confidence_score_view(
    summary: imglab_core::ConfidenceScoreView,
) -> ConfidenceScoreView {
    ConfidenceScoreView {
        overall: summary.overall,
        title: summary.title,
        description: summary.description,
        schema_prompt: summary.schema_prompt,
        tags: summary.tags,
        category: summary.category,
    }
}

pub(crate) fn daemon_task_view(task: DaemonTask) -> DaemonTaskView {
    DaemonTaskView {
        id: task.id,
        library_id: task.library_id,
        task_type: task.task_type,
        status: task.status,
        queue_position: task.queue_position,
        priority: task.priority,
        provider: task.provider,
        operation: task.operation,
        concurrency_group: task.concurrency_group,
        attempt_count: task.attempt_count,
        max_attempts: task.max_attempts,
        next_retry_at: task.next_retry_at,
        input: task.input,
        created_at: task.created_at,
        updated_at: task.updated_at,
        last_error_code: task.last_error_code,
        last_error_message: task.last_error_message,
        error_classification: task.error_classification,
        wait_reason: task.wait_reason,
    }
}

pub(crate) fn daemon_task_attempt_view(attempt: DaemonTaskAttempt) -> DaemonTaskAttemptView {
    DaemonTaskAttemptView {
        id: attempt.id,
        task_id: attempt.task_id,
        attempt_number: attempt.attempt_number,
        status: attempt.status,
        started_at: attempt.started_at,
        completed_at: attempt.completed_at,
        log_path: attempt.log_path,
        exit_code: attempt.exit_code,
        error_code: attempt.error_code,
        error_message: attempt.error_message,
        error_classification: attempt.error_classification,
    }
}

pub(crate) fn daemon_task_event_view(event: DaemonTaskEvent) -> DaemonTaskEventView {
    DaemonTaskEventView {
        id: event.id,
        task_id: event.task_id,
        event_type: event.event_type,
        message: event.message,
        payload: event.payload,
        created_at: event.created_at,
    }
}

pub(crate) fn daemon_task_output_view(output: DaemonTaskOutput) -> DaemonTaskOutputView {
    DaemonTaskOutputView {
        id: output.id,
        task_id: output.task_id,
        output_type: output.output_type,
        target_id: output.target_id,
        payload: output.payload,
        created_at: output.created_at,
    }
}

pub(crate) fn daemon_task_detail_view(
    detail: DaemonTaskDetail,
    log_tail: String,
    log_tail_truncated: bool,
) -> DaemonTaskDetailView {
    DaemonTaskDetailView {
        task: daemon_task_view(detail.task),
        attempts: detail
            .attempts
            .into_iter()
            .map(daemon_task_attempt_view)
            .collect(),
        events: detail
            .events
            .into_iter()
            .map(daemon_task_event_view)
            .collect(),
        outputs: detail
            .outputs
            .into_iter()
            .map(daemon_task_output_view)
            .collect(),
        log_tail,
        log_tail_truncated,
    }
}
