use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::id::Id;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Artifact {
  pub archived_at: Option<DateTime<Utc>>,
  #[serde(default, skip)]
  pub body: String,
  pub created_at: DateTime<Utc>,
  pub id: Id,
  #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
  pub kind: Option<String>,
  #[serde(default, skip_serializing_if = "yaml_serde::Mapping::is_empty")]
  pub metadata: yaml_serde::Mapping,
  #[serde(default)]
  pub tags: Vec<String>,
  pub title: String,
  pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ArtifactFilter {
  pub all: bool,
  pub kind: Option<String>,
  pub only_archived: bool,
  pub tag: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ArtifactPatch {
  pub body: Option<String>,
  pub kind: Option<String>,
  pub metadata: Option<yaml_serde::Mapping>,
  pub tags: Option<Vec<String>>,
  pub title: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct NewArtifact {
  pub body: String,
  pub kind: Option<String>,
  pub metadata: yaml_serde::Mapping,
  pub tags: Vec<String>,
  pub title: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  mod artifact {
    use super::*;

    mod roundtrip {
      use pretty_assertions::assert_eq;

      use super::*;

      #[test]
      fn it_roundtrips_frontmatter_through_yaml() {
        let now = Utc::now();
        let artifact = Artifact {
          archived_at: None,
          body: String::new(),
          created_at: now,
          id: "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap(),
          kind: Some("note".to_string()),
          metadata: yaml_serde::Mapping::new(),
          tags: vec!["test".to_string()],
          title: "Test Artifact".to_string(),
          updated_at: now,
        };

        let yaml_str = yaml_serde::to_string(&artifact).unwrap();
        let roundtripped: Artifact = yaml_serde::from_str(&yaml_str).unwrap();

        assert_eq!(artifact.archived_at, roundtripped.archived_at);
        assert_eq!(artifact.created_at, roundtripped.created_at);
        assert_eq!(artifact.id, roundtripped.id);
        assert_eq!(artifact.kind, roundtripped.kind);
        assert_eq!(artifact.metadata, roundtripped.metadata);
        assert_eq!(artifact.tags, roundtripped.tags);
        assert_eq!(artifact.title, roundtripped.title);
        assert_eq!(artifact.updated_at, roundtripped.updated_at);
      }

      #[test]
      fn it_serializes_kind_as_type() {
        let now = Utc::now();
        let artifact = Artifact {
          archived_at: None,
          body: String::new(),
          created_at: now,
          id: "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap(),
          kind: Some("note".to_string()),
          metadata: yaml_serde::Mapping::new(),
          tags: vec![],
          title: "Test".to_string(),
          updated_at: now,
        };

        let yaml_str = yaml_serde::to_string(&artifact).unwrap();
        assert!(
          yaml_str.contains("type:"),
          "Expected 'type:' in YAML output, got: {yaml_str}"
        );
        assert!(
          !yaml_str.contains("kind:"),
          "Should not contain 'kind:' in YAML output, got: {yaml_str}"
        );
      }
    }
  }
}
