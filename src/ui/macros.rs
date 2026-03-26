/// Implement `fmt::Display` for a type that has a `write_to` method.
///
/// Two variants:
/// - `impl_display_via_write_to!(Type, theme)` — for `write_to(&self, w, &Theme)`
/// - `impl_display_via_write_to!(Type)` — for `write_to(&self, w)` (no theme)
macro_rules! impl_display_via_write_to {
  ($ty:ty, theme) => {
    impl std::fmt::Display for $ty {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = Vec::new();
        self
          .write_to(&mut buf, &$crate::ui::theme::Theme::default())
          .map_err(|_| std::fmt::Error)?;
        let s = String::from_utf8(buf).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", s.trim_end())
      }
    }
  };
  ($ty:ty) => {
    impl std::fmt::Display for $ty {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = Vec::new();
        self.write_to(&mut buf).map_err(|_| std::fmt::Error)?;
        let s = String::from_utf8(buf).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", s.trim_end())
      }
    }
  };
}

pub(crate) use impl_display_via_write_to;
