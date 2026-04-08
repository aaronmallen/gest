//! Shared `--limit` flag for list commands.

use clap::Args;

/// Common `--limit` flag that can be flattened into any list command struct.
#[derive(Args, Clone, Debug, Default)]
pub struct LimitArgs {
  /// Cap the number of items returned.
  #[arg(long, value_name = "N")]
  pub limit: Option<usize>,
}

impl LimitArgs {
  /// Truncate `items` in place to at most `limit` entries when set; no-op otherwise.
  pub fn apply<T>(&self, items: &mut Vec<T>) {
    if let Some(n) = self.limit
      && items.len() > n
    {
      items.truncate(n);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod apply {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_does_nothing_when_limit_is_none() {
      let args = LimitArgs {
        limit: None,
      };
      let mut items = vec![1, 2, 3, 4, 5];

      args.apply(&mut items);

      assert_eq!(items, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn it_does_nothing_when_under_limit() {
      let args = LimitArgs {
        limit: Some(10),
      };
      let mut items = vec![1, 2, 3];

      args.apply(&mut items);

      assert_eq!(items, vec![1, 2, 3]);
    }

    #[test]
    fn it_truncates_to_zero() {
      let args = LimitArgs {
        limit: Some(0),
      };
      let mut items = vec![1, 2, 3];

      args.apply(&mut items);

      assert!(items.is_empty());
    }

    #[test]
    fn it_truncates_when_over_limit() {
      let args = LimitArgs {
        limit: Some(2),
      };
      let mut items = vec![1, 2, 3, 4, 5];

      args.apply(&mut items);

      assert_eq!(items, vec![1, 2]);
    }
  }
}
