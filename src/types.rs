use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FileAst {
    pub path: String,
    pub root_kind: String,
    pub nodes: Vec<JsonNode>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct JsonNode {
    pub kind: String,
    pub start_byte: u32,
    pub end_byte: u32,
    pub start_line: u32,
    pub end_line: u32,
    pub child_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    pub crate_path: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub name: String,
    pub text: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub line_text: String,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct CrateCaptures {
    pub crate_path: String,
    pub captures: Vec<Capture>,
}
