use super::{
    assets::{load_generation_event, load_version},
    database_error,
};
use crate::{
    version_name, AssetId, AssetVersionId, DomainResult, LineageEntry, PromotedSourceView,
    VersionSummary, VersionTreeIssue, VersionTreeNode,
};
use rusqlite::{params, Connection};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct VersionTreeRow {
    version_id: AssetVersionId,
    asset_id: AssetId,
    parent_version_id: Option<AssetVersionId>,
    version_number: u32,
    version_name: String,
    file_path: PathBuf,
    created_at: String,
    provider: Option<String>,
    model_label: Option<String>,
    generation_status: Option<String>,
}

#[derive(Debug, Default)]
pub(super) struct VersionTreeModel {
    pub(super) roots: Vec<VersionTreeNode>,
    pub(super) names_by_id: HashMap<String, String>,
    pub(super) issues: Vec<VersionTreeIssue>,
}

#[derive(Debug, Default)]
pub(super) struct VersionTreeSummary {
    pub(super) names_by_id: HashMap<String, String>,
    pub(super) branch_count: u32,
}

pub(super) fn load_asset_version_tree_summaries(
    connection: &Connection,
) -> DomainResult<HashMap<String, VersionTreeSummary>> {
    let mut grouped: HashMap<String, Vec<VersionTreeRow>> = HashMap::new();
    for row in load_all_version_tree_rows(connection)? {
        grouped.entry(row.asset_id.0.clone()).or_default().push(row);
    }

    let mut summaries = HashMap::new();
    for (asset_id, rows) in grouped {
        let model = build_version_tree_model(rows);
        summaries.insert(
            asset_id,
            VersionTreeSummary {
                names_by_id: model.names_by_id,
                branch_count: count_branching_nodes(&model.roots),
            },
        );
    }
    Ok(summaries)
}

pub(super) fn build_asset_version_tree(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<VersionTreeModel> {
    let rows = load_version_tree_rows_for_asset(connection, asset_id)?;
    let cross_asset_parent_ids = load_cross_asset_parent_ids(connection, asset_id, &rows)?;
    Ok(build_version_tree_model_with_cross_asset_parents(
        rows,
        cross_asset_parent_ids,
    ))
}

fn load_all_version_tree_rows(connection: &Connection) -> DomainResult<Vec<VersionTreeRow>> {
    load_version_tree_rows(connection, None)
}

fn load_version_tree_rows_for_asset(
    connection: &Connection,
    asset_id: &AssetId,
) -> DomainResult<Vec<VersionTreeRow>> {
    load_version_tree_rows(connection, Some(asset_id))
}

fn load_version_tree_rows(
    connection: &Connection,
    asset_id: Option<&AssetId>,
) -> DomainResult<Vec<VersionTreeRow>> {
    let where_clause = if asset_id.is_some() {
        "WHERE av.asset_id = ?1"
    } else {
        ""
    };
    let sql = format!(
        "
        SELECT av.id, av.asset_id, av.parent_version_id, av.version_number,
               av.file_path, av.created_at, ge.provider, ge.provider_model, ge.status
        FROM asset_versions av
        LEFT JOIN generation_events ge ON ge.id = av.generation_event_id
        {where_clause}
        ORDER BY av.asset_id ASC, av.created_at ASC, av.id ASC
        "
    );
    let mut statement = connection.prepare(&sql).map_err(database_error)?;
    let map_row = |row: &rusqlite::Row<'_>| {
        let version_number: u32 = row.get(3)?;
        Ok(VersionTreeRow {
            version_id: AssetVersionId(row.get(0)?),
            asset_id: AssetId(row.get(1)?),
            parent_version_id: row.get::<_, Option<String>>(2)?.map(AssetVersionId),
            version_number,
            version_name: version_name(version_number),
            file_path: PathBuf::from(row.get::<_, String>(4)?),
            created_at: row.get(5)?,
            provider: row.get(6)?,
            model_label: row.get(7)?,
            generation_status: row.get(8)?,
        })
    };

    let rows = if let Some(asset_id) = asset_id {
        statement
            .query_map(params![asset_id.0], map_row)
            .map_err(database_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(database_error)?
    } else {
        statement
            .query_map([], map_row)
            .map_err(database_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(database_error)?
    };
    Ok(rows)
}

fn build_version_tree_model(rows: Vec<VersionTreeRow>) -> VersionTreeModel {
    build_version_tree_model_with_cross_asset_parents(rows, HashSet::new())
}

fn build_version_tree_model_with_cross_asset_parents(
    rows: Vec<VersionTreeRow>,
    cross_asset_parent_ids: HashSet<String>,
) -> VersionTreeModel {
    let mut model = VersionTreeModel::default();
    if rows.is_empty() {
        return model;
    }

    let by_id: HashMap<String, VersionTreeRow> = rows
        .iter()
        .cloned()
        .map(|row| (row.version_id.0.clone(), row))
        .collect();
    let mut children_by_parent: HashMap<String, Vec<VersionTreeRow>> = HashMap::new();
    let mut roots = Vec::new();

    for row in rows {
        match &row.parent_version_id {
            Some(parent_id) if by_id.contains_key(&parent_id.0) => {
                children_by_parent
                    .entry(parent_id.0.clone())
                    .or_default()
                    .push(row);
            }
            Some(parent_id) => {
                let cross_asset_parent = cross_asset_parent_ids.contains(&parent_id.0);
                model.issues.push(VersionTreeIssue {
                    kind: if cross_asset_parent {
                        "cross_asset_parent"
                    } else {
                        "missing_parent"
                    }
                    .to_string(),
                    version_id: Some(row.version_id.clone()),
                    parent_version_id: Some(parent_id.clone()),
                    message: if cross_asset_parent {
                        "version parent belongs to a different asset"
                    } else {
                        "version parent is missing"
                    }
                    .to_string(),
                });
                roots.push(row);
            }
            None => roots.push(row),
        }
    }

    for children in children_by_parent.values_mut() {
        sort_version_tree_rows(children);
    }
    sort_version_tree_rows(&mut roots);

    let mut visited = HashSet::new();
    let mut path = HashSet::new();
    for (index, root) in roots.iter().enumerate() {
        let tree_name = format!("v{}", index + 1);
        let node = build_version_tree_node(
            root,
            tree_name,
            &children_by_parent,
            &mut visited,
            &mut path,
            &mut model,
        );
        model.roots.push(node);
    }

    let mut remaining: Vec<VersionTreeRow> = by_id
        .into_iter()
        .filter_map(|(id, row)| (!visited.contains(&id)).then_some(row))
        .collect();
    sort_version_tree_rows(&mut remaining);
    for row in remaining {
        model.issues.push(VersionTreeIssue {
            kind: "cycle".to_string(),
            version_id: Some(row.version_id.clone()),
            parent_version_id: row.parent_version_id.clone(),
            message: "version tree contains a cycle or unreachable parent chain".to_string(),
        });
        let tree_name = row.version_name.clone();
        let node = version_tree_leaf(&row, tree_name.clone());
        model
            .names_by_id
            .insert(row.version_id.0.clone(), tree_name);
        model.roots.push(node);
    }

    model
}

fn load_cross_asset_parent_ids(
    connection: &Connection,
    asset_id: &AssetId,
    rows: &[VersionTreeRow],
) -> DomainResult<HashSet<String>> {
    let local_ids: HashSet<&str> = rows.iter().map(|row| row.version_id.0.as_str()).collect();
    let parent_ids: Vec<&str> = rows
        .iter()
        .filter_map(|row| row.parent_version_id.as_ref())
        .map(|parent_id| parent_id.0.as_str())
        .filter(|parent_id| !local_ids.contains(parent_id))
        .collect();
    let mut cross_asset_parent_ids = HashSet::new();
    for parent_id in parent_ids {
        let parent_asset_id = connection
            .query_row(
                "SELECT asset_id FROM asset_versions WHERE id = ?1",
                params![parent_id],
                |row| row.get::<_, String>(0),
            )
            .map(Some)
            .or_else(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(database_error(other)),
            })?;
        if parent_asset_id
            .as_deref()
            .is_some_and(|id| id != asset_id.0)
        {
            cross_asset_parent_ids.insert(parent_id.to_string());
        }
    }
    Ok(cross_asset_parent_ids)
}

fn build_version_tree_node(
    row: &VersionTreeRow,
    tree_name: String,
    children_by_parent: &HashMap<String, Vec<VersionTreeRow>>,
    visited: &mut HashSet<String>,
    path: &mut HashSet<String>,
    model: &mut VersionTreeModel,
) -> VersionTreeNode {
    if path.contains(&row.version_id.0) {
        model.issues.push(VersionTreeIssue {
            kind: "cycle".to_string(),
            version_id: Some(row.version_id.clone()),
            parent_version_id: row.parent_version_id.clone(),
            message: "version tree cycle detected".to_string(),
        });
        return version_tree_leaf(row, tree_name);
    }

    visited.insert(row.version_id.0.clone());
    path.insert(row.version_id.0.clone());
    model
        .names_by_id
        .insert(row.version_id.0.clone(), tree_name.clone());

    let mut node = version_tree_leaf(row, tree_name.clone());
    if let Some(children) = children_by_parent.get(&row.version_id.0) {
        for (index, child) in children.iter().enumerate() {
            let child_name = format!("{tree_name}.{}", index + 1);
            node.children.push(build_version_tree_node(
                child,
                child_name,
                children_by_parent,
                visited,
                path,
                model,
            ));
        }
    }
    path.remove(&row.version_id.0);
    node
}

fn version_tree_leaf(row: &VersionTreeRow, tree_name: String) -> VersionTreeNode {
    VersionTreeNode {
        version_id: row.version_id.clone(),
        parent_version_id: row.parent_version_id.clone(),
        tree_name,
        version_number: row.version_number,
        version_name: row.version_name.clone(),
        file_path: row.file_path.clone(),
        created_at: row.created_at.clone(),
        provider: row.provider.clone(),
        model_label: row.model_label.clone(),
        generation_status: row.generation_status.clone(),
        children: Vec::new(),
    }
}

fn sort_version_tree_rows(rows: &mut [VersionTreeRow]) {
    rows.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.version_id.0.cmp(&right.version_id.0))
    });
}

fn count_branching_nodes(nodes: &[VersionTreeNode]) -> u32 {
    nodes
        .iter()
        .map(|node| u32::from(!node.children.is_empty()) + count_branching_nodes(&node.children))
        .sum()
}

pub(super) fn load_promoted_source(
    connection: &Connection,
    target_version: &VersionSummary,
    target_names_by_id: &HashMap<String, String>,
) -> DomainResult<Option<PromotedSourceView>> {
    let promoted_source = connection
        .query_row(
            "
            SELECT src.source_asset_id, a.title, src.source_version_id, av.version_number
            FROM asset_version_sources src
            INNER JOIN assets a ON a.id = src.source_asset_id
            INNER JOIN asset_versions av ON av.id = src.source_version_id
            WHERE src.target_version_id = ?1 AND src.source_kind = 'promoted_from'
            ",
            params![target_version.id.0],
            |row| {
                let source_version_id = AssetVersionId(row.get(2)?);
                let source_version_number: u32 = row.get(3)?;
                Ok(PromotedSourceView {
                    source_asset_id: AssetId(row.get(0)?),
                    source_asset_title: row.get(1)?,
                    source_version_id: source_version_id.clone(),
                    source_version_number,
                    source_version_name: version_name(source_version_number),
                    source_version_tree_name: None,
                })
            },
        )
        .map(Some)
        .or_else(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(database_error(other)),
        })?;

    let Some(mut promoted_source) = promoted_source else {
        return Ok(None);
    };
    promoted_source.source_version_tree_name = target_names_by_id
        .get(&promoted_source.source_version_id.0)
        .cloned()
        .or_else(|| {
            build_asset_version_tree(connection, &promoted_source.source_asset_id)
                .ok()
                .and_then(|model| {
                    model
                        .names_by_id
                        .get(&promoted_source.source_version_id.0)
                        .cloned()
                })
        });
    Ok(Some(promoted_source))
}

pub(super) fn load_asset_scoped_lineage(
    connection: &Connection,
    focused_version: &VersionSummary,
) -> DomainResult<Vec<LineageEntry>> {
    let mut lineage = Vec::new();
    let mut current = Some(focused_version.clone());
    let mut visited = HashSet::new();

    while let Some(version) = current {
        if !visited.insert(version.id.0.clone()) {
            break;
        }
        let event = version
            .generation_event_id
            .as_ref()
            .map(|event_id| load_generation_event(connection, event_id))
            .transpose()?;
        let next = version
            .parent_version_id
            .as_ref()
            .and_then(|parent_id| load_version(connection, parent_id).ok())
            .filter(|parent| parent.asset_id == focused_version.asset_id);
        lineage.push(LineageEntry {
            version,
            generation_event: event,
        });
        current = next;
    }

    Ok(lineage)
}
