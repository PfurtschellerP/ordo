/// Something is wrong with the design tokens.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A file could not be read.
    #[error("could not read {file}: {source}")]
    Io {
        /// The file.
        file: String,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// A file is not valid JSON or TOML, or has the wrong structure.
    #[error("{file}: {message}")]
    Parse {
        /// The file.
        file: String,
        /// What is wrong.
        message: String,
    },
    /// The design manifest (`design.toml`) is invalid.
    #[error("design.toml: {0}")]
    Manifest(String),
    /// A token is invalid.
    #[error("token {path}: {message}")]
    Token {
        /// Dotted path of the token.
        path: String,
        /// What is wrong.
        message: String,
    },
    /// A token is defined twice.
    #[error("token {path} is defined in both {first} and {second}")]
    Duplicate {
        /// Dotted path of the token.
        path: String,
        /// File of the first definition.
        first: String,
        /// File of the second definition.
        second: String,
    },
    /// A theme is invalid.
    #[error("theme {theme}: {message}")]
    Theme {
        /// Name of the theme.
        theme: String,
        /// What is wrong.
        message: String,
    },
    /// An organisation theme tries to change a protected token.
    #[error("organisation themes may not change {path}: it belongs to the protected group {group}")]
    Protected {
        /// Dotted path of the token.
        path: String,
        /// The protected group it belongs to.
        group: String,
    },
    /// An organisation theme defines a token the base theme doesn't have.
    #[error(
        "{path} is not a token of theme {theme}; organisation themes can only change existing semantic tokens"
    )]
    UnknownOverride {
        /// Dotted path of the token.
        path: String,
        /// The base theme.
        theme: String,
    },
}

impl Error {
    pub(crate) fn token(path: &str, message: impl Into<String>) -> Self {
        Self::Token {
            path: path.to_owned(),
            message: message.into(),
        }
    }

    pub(crate) fn theme(theme: &str, message: impl Into<String>) -> Self {
        Self::Theme {
            theme: theme.to_owned(),
            message: message.into(),
        }
    }
}
