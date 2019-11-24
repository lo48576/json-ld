//! JSON-LD error.

use std::fmt;

use thiserror;

/// JSON-LD processing result.
pub type Result<T> = std::result::Result<T, Error>;

/// Error code for JSON-LD processing.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#jsonlderrorcode>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// Colliding keywords.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-colliding-keywords>.
    CollidingKeywords,
    /// Conflicting indexes.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-conflicting-indexes>.
    ConflictingIndexes,
    /// Context overflow.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-context-overflow>.
    ContextOverflow,
    /// Cyclic IRI mapping.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-cyclic-iri-mapping>.
    CyclicIriMapping,
    /// Invalid base direction.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dfn-invalid-base-direction>.
    InvalidBaseDirection,
    /// Invalid base IRI.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-base-iri>.
    InvalidBaseIri,
    /// Invalid container mapping.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-container-mapping>.
    InvalidContainerMapping,
    /// Invalid context entry.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-context-entry>.
    InvalidContextEntry,
    /// Invalid context nullification.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-context-nullification>.
    InvalidContextNullification,
    /// Invalid default language.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-default-language>.
    InvalidDefaultLanguage,
    /// Invalid `@id` value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-@id-value>.
    InvalidIdValue,
    /// Invalid `@import` value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-@import-value>.
    InvalidImportValue,
    /// Invalid `@included` value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-@included-value>.
    InvalidIncludedValue,
    /// Invalid `@index` value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-@index-value>.
    InvalidIndexValue,
    /// Invalid IRI mapping.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-iri-mapping>.
    InvalidIriMapping,
    /// Invalid JSON literal.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-json-literal>.
    InvalidJsonLiteral,
    /// Invalid keyword alias.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-keyword-alias>.
    InvalidKeywordAlias,
    /// Invalid language map value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-language-map-value>.
    InvalidLanguageMapValue,
    /// Invalid language mapping.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-language-mapping>.
    InvalidLanguageMapping,
    /// Invalid language-tagged string.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-language-tagged-string>.
    InvalidLanguageTaggedString,
    /// Invalid language-tagged value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-language-tagged-value>.
    InvalidLanguageTaggedValue,
    /// Invalid local context.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-local-context>.
    InvalidLocalContext,
    /// Invalid `@nest` value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-@nest-value>.
    InvalidNestValue,
    /// Invalid `@prefix` value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-@prefix-value>.
    InvalidPrefixValue,
    /// Invalid `@propagate` value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-@propagate-value>.
    InvalidPropagateValue,
    /// Invalid `@protected` value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191112/#jsonlderrorcode>.
    InvalidProtectedValue,
    /// Invalid remote context.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-remote-context>.
    InvalidRemoteContext,
    /// Invalid reverse property.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-reverse-property>.
    InvalidReverseProperty,
    /// Invalid reverse property map.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-reverse-property-map>.
    InvalidReversePropertyMap,
    /// Invalid reverse property value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-reverse-property-value>.
    InvalidReversePropertyValue,
    /// Invalid `@reverse` value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-@reverse-value>.
    InvalidReverseValue,
    /// Invalid scoped context.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-scoped-context>.
    InvalidScopedContext,
    /// Invalid script element.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-script-element>.
    InvalidScriptElement,
    /// Invalid set or list object.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-set-or-list-object>.
    InvalidSetOrListObject,
    /// Invalid term definition.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-term-definition>.
    InvalidTermDefinition,
    /// Invalid type mapping.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-type-mapping>.
    InvalidTypeMapping,
    /// Invalid type value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-type-value>.
    InvalidTypeValue,
    /// Invalid typed value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-typed-value>.
    InvalidTypedValue,
    /// Invalid value object.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-value-object>.
    InvalidValueObject,
    /// Invalid value object value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-value-object-value>.
    InvalidValueObjectValue,
    /// Invalid `@version` value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-@version-value>.
    InvalidVersionValue,
    /// Invalid vocab mapping.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-invalid-vocab-mapping>.
    InvalidVocabMapping,
    /// IRI confused with prefix.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-iri-confused-with-prefix>.
    IriConfusedWithPrefix,
    /// Keyword redefinition.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-keyword-redefinition>.
    KeywordRedefinition,
    /// Loading document failed.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-loading-document-failed>.
    LoadingDocumentFailed,
    /// Loading remote context failed.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-loading-remote-context-failed>.
    LoadingRemoteContextFailed,
    /// Multiple context link headers.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-multiple-context-link-headers>.
    MultipleContextLinkHeaders,
    /// Processing mode conflict.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-processing-mode-conflict>.
    ProcessingModeConflict,
    /// Protected term redefinition.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#dom-jsonlderrorcode-protected-term-redefinition>.
    ProtectedTermRedefinition,
    /// Uncategorized errors (not specified in the spec).
    ///
    /// This may include spec ambiguity and internal processor error.
    Uncategorized,
}

impl ErrorCode {
    /// Returns error message string.
    pub fn message(self) -> &'static str {
        match self {
            Self::CollidingKeywords => "colliding keywords",
            Self::ConflictingIndexes => "conflicting indexes",
            Self::ContextOverflow => "context overflow",
            Self::CyclicIriMapping => "cyclic IRI mapping",
            Self::InvalidBaseDirection => "invalid base direction",
            Self::InvalidBaseIri => "invalid base IRI",
            Self::InvalidContainerMapping => "invalid container mapping",
            Self::InvalidContextEntry => "invalid context entry",
            Self::InvalidContextNullification => "invalid context nullification",
            Self::InvalidDefaultLanguage => "invalid default language",
            Self::InvalidIdValue => "invalid @id value",
            Self::InvalidImportValue => "invalid @import value",
            Self::InvalidIncludedValue => "invalid @included value",
            Self::InvalidIndexValue => "invalid @index value",
            Self::InvalidIriMapping => "invalid IRI mapping",
            Self::InvalidJsonLiteral => "invalid JSON literal",
            Self::InvalidKeywordAlias => "invalid keyword alias",
            Self::InvalidLanguageMapValue => "invalid language map value",
            Self::InvalidLanguageMapping => "invalid language mapping",
            Self::InvalidLanguageTaggedString => "invalid language-tagged string",
            Self::InvalidLanguageTaggedValue => "invalid language-tagged value",
            Self::InvalidLocalContext => "invalid local context",
            Self::InvalidNestValue => "invalid @nest value",
            Self::InvalidPrefixValue => "invalid @prefix value",
            Self::InvalidPropagateValue => "invalid @propagate value",
            Self::InvalidProtectedValue => "invalid @protected value",
            Self::InvalidRemoteContext => "invalid remote context",
            Self::InvalidReverseProperty => "invalid reverse property",
            Self::InvalidReversePropertyMap => "invalid reverse property map",
            Self::InvalidReversePropertyValue => "invalid reverse property value",
            Self::InvalidReverseValue => "invalid @reverse value",
            Self::InvalidScopedContext => "invalid scoped context",
            Self::InvalidScriptElement => "invalid script element",
            Self::InvalidSetOrListObject => "invalid set or list object",
            Self::InvalidTermDefinition => "invalid term definition",
            Self::InvalidTypeMapping => "invalid type mapping",
            Self::InvalidTypeValue => "invalid type value",
            Self::InvalidTypedValue => "invalid typed value",
            Self::InvalidValueObject => "invalid value object",
            Self::InvalidValueObjectValue => "invalid value object value",
            Self::InvalidVersionValue => "invalid @version value",
            Self::InvalidVocabMapping => "invalid vocab mapping",
            Self::IriConfusedWithPrefix => "IRI confused with prefix",
            Self::KeywordRedefinition => "keyword redefinition",
            Self::LoadingDocumentFailed => "loading document failed",
            Self::LoadingRemoteContextFailed => "loading remote context failed",
            Self::MultipleContextLinkHeaders => "multiple context link header",
            Self::ProcessingModeConflict => "processing mode conflict",
            Self::ProtectedTermRedefinition => "protected term redefinition",
            Self::Uncategorized => "uncategorized error",
        }
    }

    /// Creates an `Error` from the error code and the given source error.
    pub(crate) fn and_source<E>(self, source: E) -> Error
    where
        E: Into<anyhow::Error>,
    {
        Error {
            code: self,
            source: Some(source.into()),
        }
    }

    /*
    /// Creates an `Error` from the error code and the given source error generator.
    pub(crate) fn with_source<E, F>(self, f: F) -> Error
    where
        E: Into<anyhow::Error>,
        F: FnOnce() -> E,
    {
        Error {
            code: self,
            source: Some(f().into()),
        }
    }
    */
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for ErrorCode {}

/// JSON-LD processing error.
#[derive(Debug, thiserror::Error)]
pub struct Error {
    /// Error code.
    code: ErrorCode,
    /// Details of the error (if available).
    #[source]
    source: Option<anyhow::Error>,
}

impl Error {
    /// Returns the error code.
    pub fn code(&self) -> ErrorCode {
        self.code
    }

    /// Wraps the error with the additional context.
    pub(crate) fn context<C>(self, context: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        let source = match self.source {
            Some(source) => source.context(context),
            None => anyhow::anyhow!("{}", context),
        };

        Self {
            code: self.code,
            source: Some(source),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code.message())?;
        if let Some(source) = self.source.as_ref() {
            write!(f, ": {}", source)?;
        }
        Ok(())
    }
}

impl From<ErrorCode> for Error {
    fn from(code: ErrorCode) -> Self {
        Self { code, source: None }
    }
}

/// Extension trait for JSON-LD processing result.
pub(crate) trait ResultExt<T> {
    /// Wraps the error value with the additional context.
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static;
    /// Wraps the error value with the additional context generated by the given function.
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T> ResultExt<T> for Result<T> {
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| err.context(context))
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|err| err.context(f()))
    }
}
