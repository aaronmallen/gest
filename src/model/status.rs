/// Generates `as_str`, `is_terminal`, `Display`, and `FromStr` impls for a `Status` enum.
///
/// # Usage
///
/// ```ignore
/// impl_status! {
///   Status {
///     Active => "active",
///     Cancelled => "cancelled" [terminal],
///     Completed => "completed" [terminal],
///   }
/// }
/// ```
macro_rules! impl_status {
  // Entry: strip outer braces, start collecting
  ( $type:ident { $($body:tt)+ } ) => {
    impl_status!(@collect $type, [], [], $($body)+);
  };

  // Collect: terminal variant (with trailing comma + more)
  (@collect $type:ident, [ $($term:tt)* ], [ $($nonterm:tt)* ],
    $variant:ident => $str:literal [terminal], $($rest:tt)+
  ) => {
    impl_status!(@collect $type, [ $($term)* $variant => $str, ], [ $($nonterm)* ], $($rest)+);
  };

  // Collect: terminal variant (last, with or without trailing comma)
  (@collect $type:ident, [ $($term:tt)* ], [ $($nonterm:tt)* ],
    $variant:ident => $str:literal [terminal] $(,)?
  ) => {
    impl_status!(@emit $type, [ $($term)* $variant => $str, ], [ $($nonterm)* ]);
  };

  // Collect: non-terminal variant (with trailing comma + more)
  (@collect $type:ident, [ $($term:tt)* ], [ $($nonterm:tt)* ],
    $variant:ident => $str:literal, $($rest:tt)+
  ) => {
    impl_status!(@collect $type, [ $($term)* ], [ $($nonterm)* $variant => $str, ], $($rest)+);
  };

  // Collect: non-terminal variant (last, with or without trailing comma)
  (@collect $type:ident, [ $($term:tt)* ], [ $($nonterm:tt)* ],
    $variant:ident => $str:literal $(,)?
  ) => {
    impl_status!(@emit $type, [ $($term)* ], [ $($nonterm)* $variant => $str, ]);
  };

  // Emit: generate all impls
  (@emit $type:ident, [ $( $tv:ident => $ts:literal, )* ], [ $( $nv:ident => $ns:literal, )* ]) => {
    impl $type {
      pub fn as_str(&self) -> &'static str {
        match self {
          $( Self::$tv => $ts, )*
          $( Self::$nv => $ns, )*
        }
      }

      pub fn is_terminal(&self) -> bool {
        match self {
          $( Self::$tv => true, )*
          $( Self::$nv => false, )*
        }
      }
    }

    impl std::fmt::Display for $type {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
      }
    }

    impl std::str::FromStr for $type {
      type Err = String;

      fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
          $( $ts => Ok(Self::$tv), )*
          $( $ns => Ok(Self::$nv), )*
          other => Err(format!("unknown status: {other}")),
        }
      }
    }
  };
}

pub(crate) use impl_status;
