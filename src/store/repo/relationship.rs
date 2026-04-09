use std::collections::{HashMap, HashSet};

use chrono::Utc;
use libsql::{Connection, Value};

use crate::store::{
  Error,
  model::{
    primitives::{EntityType, Id, RelationshipType},
    relationship::Model,
  },
};

const SELECT_COLUMNS: &str = "id, rel_type, source_id, source_type, target_id, target_type, created_at, updated_at";

/// Create a new relationship between two entities.
pub async fn create(
  conn: &Connection,
  rel_type: RelationshipType,
  source_type: EntityType,
  source_id: &Id,
  target_type: EntityType,
  target_id: &Id,
) -> Result<Model, Error> {
  log::debug!("repo::relationship::create");
  let id = Id::new();
  let now = Utc::now().to_rfc3339();
  conn
    .execute(
      &format!("INSERT INTO relationships ({SELECT_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"),
      [
        id.to_string(),
        rel_type.to_string(),
        source_id.to_string(),
        source_type.to_string(),
        target_id.to_string(),
        target_type.to_string(),
        now.clone(),
        now,
      ],
    )
    .await?;

  find_by_id(conn, id)
    .await?
    .ok_or_else(|| Error::InvalidValue("relationship not found after insert".into()))
}

/// Delete a relationship by its ID. Returns true if deleted.
pub async fn delete(conn: &Connection, id: &Id) -> Result<bool, Error> {
  log::debug!("repo::relationship::delete");
  let affected = conn
    .execute("DELETE FROM relationships WHERE id = ?1", [id.to_string()])
    .await?;
  Ok(affected > 0)
}

/// Find a relationship by its [`Id`].
pub async fn find_by_id(conn: &Connection, id: impl Into<Id>) -> Result<Option<Model>, Error> {
  log::debug!("repo::relationship::find_by_id");
  let id = id.into();
  let mut rows = conn
    .query(
      &format!("SELECT {SELECT_COLUMNS} FROM relationships WHERE id = ?1"),
      [id.to_string()],
    )
    .await?;

  match rows.next().await? {
    Some(row) => Ok(Some(Model::try_from(row)?)),
    None => Ok(None),
  }
}

/// Return all relationships attached to each of the given entities in a
/// single query.
///
/// Each returned [`Model`] is associated with every queried entity it touches:
/// a relationship where both endpoints are in `entity_ids` appears under both
/// keys. Entities with no relationships are absent from the returned map.
/// Passing an empty slice returns an empty map without issuing a query.
pub async fn for_entities(
  conn: &Connection,
  entity_type: EntityType,
  entity_ids: &[Id],
) -> Result<HashMap<Id, Vec<Model>>, Error> {
  log::debug!("repo::relationship::for_entities");
  let mut map: HashMap<Id, Vec<Model>> = HashMap::new();
  if entity_ids.is_empty() {
    return Ok(map);
  }

  let placeholders = (2..entity_ids.len() + 2)
    .map(|i| format!("?{i}"))
    .collect::<Vec<_>>()
    .join(", ");
  let sql = format!(
    "SELECT {SELECT_COLUMNS} FROM relationships \
      WHERE (source_type = ?1 AND source_id IN ({placeholders})) \
      OR (target_type = ?1 AND target_id IN ({placeholders}))"
  );

  let mut params: Vec<Value> = Vec::with_capacity(entity_ids.len() * 2 + 1);
  params.push(Value::from(entity_type.to_string()));
  for id in entity_ids {
    params.push(Value::from(id.to_string()));
  }
  for id in entity_ids {
    params.push(Value::from(id.to_string()));
  }

  let mut rows = conn.query(&sql, libsql::params_from_iter(params)).await?;
  let id_set: HashSet<&Id> = entity_ids.iter().collect();
  while let Some(row) = rows.next().await? {
    let rel = Model::try_from(row)?;
    let matches_source = rel.source_type() == entity_type && id_set.contains(rel.source_id());
    let matches_target = rel.target_type() == entity_type && id_set.contains(rel.target_id());
    match (matches_source, matches_target) {
      (true, true) => {
        map.entry(rel.source_id().clone()).or_default().push(rel.clone());
        map.entry(rel.target_id().clone()).or_default().push(rel);
      }
      (true, false) => {
        map.entry(rel.source_id().clone()).or_default().push(rel);
      }
      (false, true) => {
        map.entry(rel.target_id().clone()).or_default().push(rel);
      }
      (false, false) => {}
    }
  }

  Ok(map)
}

/// Return all relationships where the entity is either source or target.
pub async fn for_entity(conn: &Connection, entity_type: EntityType, entity_id: &Id) -> Result<Vec<Model>, Error> {
  log::debug!("repo::relationship::for_entity");
  let mut rows = conn
    .query(
      &format!(
        "SELECT {SELECT_COLUMNS} FROM relationships \
          WHERE (source_type = ?1 AND source_id = ?2) \
          OR (target_type = ?1 AND target_id = ?2)"
      ),
      [entity_type.to_string(), entity_id.to_string()],
    )
    .await?;

  let mut relationships = Vec::new();
  while let Some(row) = rows.next().await? {
    relationships.push(Model::try_from(row)?);
  }
  Ok(relationships)
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use tempfile::TempDir;

  use super::*;
  use crate::store::{self, Db, model::Project};

  async fn setup() -> (Arc<Db>, Connection, TempDir, Id, Id) {
    let (store, tmp) = store::open_temp().await.unwrap();
    let conn = store.connect().await.unwrap();
    let project = Project::new("/tmp/rel-test".into());
    conn
      .execute(
        "INSERT INTO projects (id, root, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        [
          project.id().to_string(),
          project.root().to_string_lossy().into_owned(),
          project.created_at().to_rfc3339(),
          project.updated_at().to_rfc3339(),
        ],
      )
      .await
      .unwrap();
    let pid = project.id().clone();

    let t1 = Id::new();
    let t2 = Id::new();
    conn
      .execute(
        "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
        [t1.to_string(), pid.to_string(), "Task 1".to_string()],
      )
      .await
      .unwrap();
    conn
      .execute(
        "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
        [t2.to_string(), pid.to_string(), "Task 2".to_string()],
      )
      .await
      .unwrap();
    (store, conn, tmp, t1, t2)
  }

  mod create_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_creates_a_relationship() {
      let (_store, conn, _tmp, t1, t2) = setup().await;

      let rel = create(
        &conn,
        RelationshipType::Blocks,
        EntityType::Task,
        &t1,
        EntityType::Task,
        &t2,
      )
      .await
      .unwrap();

      assert_eq!(rel.rel_type(), RelationshipType::Blocks);
      assert_eq!(rel.source_id(), &t1);
      assert_eq!(rel.target_id(), &t2);
    }
  }

  mod delete_fn {
    use super::*;

    #[tokio::test]
    async fn it_deletes_a_relationship() {
      let (_store, conn, _tmp, t1, t2) = setup().await;

      let rel = create(
        &conn,
        RelationshipType::Blocks,
        EntityType::Task,
        &t1,
        EntityType::Task,
        &t2,
      )
      .await
      .unwrap();

      let deleted = delete(&conn, rel.id()).await.unwrap();

      assert!(deleted);
    }
  }

  mod for_entities_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    async fn insert_task(conn: &Connection, project_id: &Id, title: &str) -> Id {
      let id = Id::new();
      conn
        .execute(
          "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
          [id.to_string(), project_id.to_string(), title.to_string()],
        )
        .await
        .unwrap();
      id
    }

    #[tokio::test]
    async fn it_groups_relationships_by_entity_in_one_query() {
      let (_store, conn, _tmp, t1, t2) = setup().await;

      let rel = create(
        &conn,
        RelationshipType::Blocks,
        EntityType::Task,
        &t1,
        EntityType::Task,
        &t2,
      )
      .await
      .unwrap();

      let map = for_entities(&conn, EntityType::Task, &[t1.clone(), t2.clone()])
        .await
        .unwrap();

      assert_eq!(map.len(), 2);
      let t1_rels = map.get(&t1).unwrap();
      assert_eq!(t1_rels.len(), 1);
      assert_eq!(t1_rels[0].id(), rel.id());
      let t2_rels = map.get(&t2).unwrap();
      assert_eq!(t2_rels.len(), 1);
      assert_eq!(t2_rels[0].id(), rel.id());
    }

    #[tokio::test]
    async fn it_returns_empty_map_for_empty_input() {
      let (_store, conn, _tmp, _t1, _t2) = setup().await;

      let map = for_entities(&conn, EntityType::Task, &[]).await.unwrap();

      assert!(map.is_empty());
    }

    #[tokio::test]
    async fn it_omits_entities_with_no_relationships() {
      let (_store, conn, _tmp, t1, t2) = setup().await;

      // Create a lone task with no relationships; ensure it's absent from the map.
      let project = crate::store::model::Project::new("/tmp/rel-test-2".into());
      conn
        .execute(
          "INSERT INTO projects (id, root, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
          [
            project.id().to_string(),
            project.root().to_string_lossy().into_owned(),
            project.created_at().to_rfc3339(),
            project.updated_at().to_rfc3339(),
          ],
        )
        .await
        .unwrap();
      let lonely = insert_task(&conn, project.id(), "Lonely").await;

      create(
        &conn,
        RelationshipType::Blocks,
        EntityType::Task,
        &t1,
        EntityType::Task,
        &t2,
      )
      .await
      .unwrap();

      let map = for_entities(&conn, EntityType::Task, &[t1.clone(), lonely.clone()])
        .await
        .unwrap();

      assert!(map.contains_key(&t1));
      assert!(!map.contains_key(&lonely));
    }
  }

  mod for_entity_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_returns_relationships_where_entity_is_source_or_target() {
      let (_store, conn, _tmp, t1, t2) = setup().await;

      create(
        &conn,
        RelationshipType::Blocks,
        EntityType::Task,
        &t1,
        EntityType::Task,
        &t2,
      )
      .await
      .unwrap();

      let rels_t1 = for_entity(&conn, EntityType::Task, &t1).await.unwrap();

      assert_eq!(rels_t1.len(), 1);

      let rels_t2 = for_entity(&conn, EntityType::Task, &t2).await.unwrap();

      assert_eq!(rels_t2.len(), 1);
    }
  }
}
