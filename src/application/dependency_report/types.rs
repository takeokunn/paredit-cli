use crate::domain::sexpr::ByteSpan;

#[derive(Debug, Clone)]
pub struct DependencyReport {
    pub dependencies: Vec<DependencyReportItem>,
}

#[derive(Debug, Clone)]
pub struct DependencyReportItem {
    pub kind: DependencyKind,
    pub target: String,
    pub path: String,
    pub span: ByteSpan,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DependencyKind {
    AsdfDependsOn,
    AsdfComponent,
    Require,
    Provide,
    Load,
    LoadFile,
    LoadLibrary,
    UsePackage,
    Import,
    DefpackageUse,
    DefpackageImportFrom,
    QualifiedSymbol,
}

impl DependencyKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::AsdfDependsOn => "asdf-depends-on",
            Self::AsdfComponent => "asdf-component",
            Self::Require => "require",
            Self::Provide => "provide",
            Self::Load => "load",
            Self::LoadFile => "load-file",
            Self::LoadLibrary => "load-library",
            Self::UsePackage => "use-package",
            Self::Import => "import",
            Self::DefpackageUse => "defpackage-use",
            Self::DefpackageImportFrom => "defpackage-import-from",
            Self::QualifiedSymbol => "qualified-symbol",
        }
    }
}
