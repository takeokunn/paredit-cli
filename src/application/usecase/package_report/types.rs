use crate::domain::sexpr::ByteSpan;

#[derive(Debug, Clone)]
pub struct PackageReport {
    pub defpackages: Vec<PackageDefinitionReport>,
    pub in_packages: Vec<InPackageReport>,
}

#[derive(Debug, Clone)]
pub struct PackageDefinitionReport {
    pub path: String,
    pub span: ByteSpan,
    pub name: String,
    pub nicknames: Vec<String>,
    pub uses: Vec<String>,
    pub exports: Vec<String>,
    pub imports: Vec<PackageImportReport>,
    pub option_count: usize,
}

#[derive(Debug, Clone)]
pub struct PackageImportReport {
    pub package: String,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct InPackageReport {
    pub path: String,
    pub span: ByteSpan,
    pub name: String,
}
