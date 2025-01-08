#![deny(missing_docs)]

/// Macro to generate an enum with associated methods.
#[macro_export]
macro_rules! define_ast_enum_type {
    ($name:ident { $($variant:ident => $repr:expr),* $(,)? }) => {
        /// Generated enum
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum $name {
            $(
                /// $variant
                $variant,
            )*
        }

        impl $name {
            /// Converts a variant into its string representation.
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => $repr,
                    )*
                }
            }

            /// Returns a list of all variants.
            pub fn all_variants() -> Vec<Self> {
                vec![
                    $(
                        Self::$variant,
                    )*
                ]
            }
        }

        /// Implement `Display` for the enum.
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }
    };
}
