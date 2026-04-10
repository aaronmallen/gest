//! JSON envelope wrapping domain entities with their sidecar data.
//!
//! [`Envelope`] flattens the entity's own fields into the top level and appends
//! relationship, tag, and (optionally) note collections so that a single JSON
//! response carries everything a consumer needs.

use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::Serialize;

use crate::store::{
  Error,
  model::{
    note,
    primitives::{EntityType, Id, RelationshipType},
    relationship,
  },
  repo,
};

/// A JSON-serializable wrapper that flattens an entity and attaches sidecars.
#[derive(Clone, Debug, Serialize)]
pub struct Envelope<'a, T: Serialize> {
  /// The domain entity whose fields are promoted to the top level.
  #[serde(flatten)]
  pub entity: &'a T,
  /// Notes attached to this entity (omitted from output when `None`).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub notes: Option<Vec<NoteView>>,
  /// Directed relationships involving this entity.
  pub relationships: Vec<RelationshipView>,
  /// Tag labels attached to this entity.
  pub tags: Vec<String>,
}

impl<'a, T: Serialize> Envelope<'a, T> {
  /// Build envelopes for many entities using batch queries (one query per sidecar type).
  pub async fn load_many(
    conn: &Connection,
    entity_type: EntityType,
    entities: &'a [(Id, T)],
    include_notes: bool,
  ) -> Result<Vec<Envelope<'a, T>>, Error> {
    let ids: Vec<Id> = entities.iter().map(|(id, _)| id.clone()).collect();
    let mut rels = repo::relationship::for_entities(conn, entity_type, &ids).await?;
    let mut tags = repo::tag::for_entities(conn, entity_type, &ids).await?;
    let mut note_map = if include_notes {
      Some(repo::note::for_entities(conn, entity_type, &ids).await?)
    } else {
      None
    };

    let envelopes = entities
      .iter()
      .map(|(id, entity)| {
        let relationships = rels
          .remove(id)
          .unwrap_or_default()
          .iter()
          .map(RelationshipView::from)
          .collect();

        let tag_labels = tags
          .remove(id)
          .unwrap_or_default()
          .into_iter()
          .map(|t| t.label().to_string())
          .collect();

        let notes = note_map
          .as_mut()
          .map(|m| m.remove(id).unwrap_or_default().iter().map(NoteView::from).collect());

        Envelope {
          entity,
          notes,
          relationships,
          tags: tag_labels,
        }
      })
      .collect();

    Ok(envelopes)
  }

  /// Build an envelope for a single entity, loading sidecars from the database.
  pub async fn load_one(
    conn: &Connection,
    entity_type: EntityType,
    entity_id: &Id,
    entity: &'a T,
    include_notes: bool,
  ) -> Result<Envelope<'a, T>, Error> {
    let ids = [entity_id.clone()];
    let mut rels = repo::relationship::for_entities(conn, entity_type, &ids).await?;
    let mut tags = repo::tag::for_entities(conn, entity_type, &ids).await?;
    let mut note_map = if include_notes {
      Some(repo::note::for_entities(conn, entity_type, &ids).await?)
    } else {
      None
    };

    let relationships = rels
      .remove(entity_id)
      .unwrap_or_default()
      .iter()
      .map(RelationshipView::from)
      .collect();

    let tag_labels = tags
      .remove(entity_id)
      .unwrap_or_default()
      .into_iter()
      .map(|t| t.label().to_string())
      .collect();

    let notes = note_map.as_mut().map(|m| {
      m.remove(entity_id)
        .unwrap_or_default()
        .iter()
        .map(NoteView::from)
        .collect()
    });

    Ok(Envelope {
      entity,
      notes,
      relationships,
      tags: tag_labels,
    })
  }
}

/// Serialization-only view of a note, without entity-linking fields.
#[derive(Clone, Debug, Serialize)]
pub struct NoteView {
  /// Markdown body content.
  pub body: String,
  /// When the note was first created.
  pub created_at: DateTime<Utc>,
  /// Stable identifier.
  pub id: Id,
  /// When the note body was last modified.
  pub updated_at: DateTime<Utc>,
}

impl From<&note::Model> for NoteView {
  fn from(m: &note::Model) -> Self {
    Self {
      body: m.body().clone(),
      created_at: *m.created_at(),
      id: m.id().clone(),
      updated_at: *m.updated_at(),
    }
  }
}

/// Serialization-only view of a relationship, without the `source_type` field.
#[derive(Clone, Debug, Serialize)]
pub struct RelationshipView {
  /// When the relationship was first recorded.
  pub created_at: DateTime<Utc>,
  /// Stable identifier.
  pub id: Id,
  /// Semantic kind of relationship.
  pub rel_type: RelationshipType,
  /// Identifier of the source (from-side) entity.
  pub source_id: Id,
  /// Identifier of the target (to-side) entity.
  pub target_id: Id,
  /// Domain type of the target entity.
  pub target_type: EntityType,
  /// When the relationship was last modified.
  pub updated_at: DateTime<Utc>,
}

impl From<&relationship::Model> for RelationshipView {
  fn from(m: &relationship::Model) -> Self {
    Self {
      created_at: *m.created_at(),
      id: m.id().clone(),
      rel_type: m.rel_type(),
      source_id: m.source_id().clone(),
      target_id: m.target_id().clone(),
      target_type: m.target_type(),
      updated_at: *m.updated_at(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod envelope_serialization {
    use pretty_assertions::assert_eq;
    use serde_json::{Value, json};

    use super::*;

    #[derive(Clone, Debug, Serialize)]
    struct Stub {
      name: String,
      status: String,
    }

    fn stub_entity() -> Stub {
      Stub {
        name: "test".to_string(),
        status: "open".to_string(),
      }
    }

    #[test]
    fn it_flattens_entity_fields_to_top_level() {
      let entity = stub_entity();
      let env = Envelope {
        entity: &entity,
        notes: None,
        relationships: vec![],
        tags: vec![],
      };

      let json: Value = serde_json::to_value(&env).unwrap();

      assert_eq!(json["name"], "test");
      assert_eq!(json["status"], "open");
    }

    #[test]
    fn it_omits_notes_key_when_none() {
      let entity = stub_entity();
      let env = Envelope {
        entity: &entity,
        notes: None,
        relationships: vec![],
        tags: vec![],
      };

      let json: Value = serde_json::to_value(&env).unwrap();

      assert!(json.get("notes").is_none());
    }

    #[test]
    fn it_serializes_empty_relationships_as_empty_array() {
      let entity = stub_entity();
      let env = Envelope {
        entity: &entity,
        notes: None,
        relationships: vec![],
        tags: vec![],
      };

      let json: Value = serde_json::to_value(&env).unwrap();

      assert_eq!(json["relationships"], json!([]));
    }

    #[test]
    fn it_serializes_empty_tags_as_empty_array() {
      let entity = stub_entity();
      let env = Envelope {
        entity: &entity,
        notes: None,
        relationships: vec![],
        tags: vec![],
      };

      let json: Value = serde_json::to_value(&env).unwrap();

      assert_eq!(json["tags"], json!([]));
    }

    #[test]
    fn it_serializes_notes_when_present() {
      let entity = stub_entity();
      let now = Utc::now();
      let env = Envelope {
        entity: &entity,
        notes: Some(vec![NoteView {
          body: "hello".to_string(),
          created_at: now,
          id: Id::new(),
          updated_at: now,
        }]),
        relationships: vec![],
        tags: vec![],
      };

      let json: Value = serde_json::to_value(&env).unwrap();

      let notes = json["notes"].as_array().unwrap();
      assert_eq!(notes.len(), 1);
      assert_eq!(notes[0]["body"], "hello");
    }
  }

  mod note_view {
    use chrono::Utc;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_serializes_all_fields() {
      let now = Utc::now();
      let id = Id::new();
      let view = NoteView {
        body: "test body".to_string(),
        created_at: now,
        id: id.clone(),
        updated_at: now,
      };

      let json: serde_json::Value = serde_json::to_value(&view).unwrap();

      assert_eq!(json["body"], "test body");
      assert_eq!(json["id"], id.to_string());
      assert!(json.get("created_at").is_some());
      assert!(json.get("updated_at").is_some());
    }
  }

  mod relationship_view {
    use chrono::Utc;
    use pretty_assertions::assert_eq;
    use serde_json::Value;

    use super::*;

    #[test]
    fn it_excludes_source_type_from_serialization() {
      let now = Utc::now();
      let view = RelationshipView {
        created_at: now,
        id: Id::new(),
        rel_type: RelationshipType::Blocks,
        source_id: Id::new(),
        target_id: Id::new(),
        target_type: EntityType::Task,
        updated_at: now,
      };

      let json: Value = serde_json::to_value(&view).unwrap();

      assert!(json.get("source_type").is_none());
    }

    #[test]
    fn it_includes_all_expected_fields() {
      let now = Utc::now();
      let view = RelationshipView {
        created_at: now,
        id: Id::new(),
        rel_type: RelationshipType::BlockedBy,
        source_id: Id::new(),
        target_id: Id::new(),
        target_type: EntityType::Artifact,
        updated_at: now,
      };

      let json: Value = serde_json::to_value(&view).unwrap();

      assert!(json.get("created_at").is_some());
      assert!(json.get("id").is_some());
      assert!(json.get("rel_type").is_some());
      assert!(json.get("source_id").is_some());
      assert!(json.get("target_id").is_some());
      assert!(json.get("target_type").is_some());
      assert!(json.get("updated_at").is_some());
    }
  }
}
