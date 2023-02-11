use crate::FileGroup;

#[derive(Debug, Clone)]
pub enum RunnerKind {
    JsFunction(u64),
}

#[derive(Debug, Clone)]
pub enum DependencyKind {
    FileGroup(FileGroup),
    Transform(Transform),
}

#[salsa::input]
#[derive(Debug, Clone)]
pub struct Transform {
    name: String,
    runner: RunnerKind,
    input: Vec<DependencyKind>,
    dependencies: Vec<DependencyKind>,
}
