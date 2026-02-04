use std::path::Path;

use crate::Format;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum MediaType {
    // --- Text / structured text ---
    TextPlain,
    TextMarkdown,
    TextHtml,
    TextXml,
    TextCsv,
    TextToml,
    TextYaml,
    TextJson,

    // --- Code (optional but handy for memory services) ---
    CodeRust,
    CodeCSharp,
    CodeTypeScript,
    CodeJavaScript,
    CodePython,
    CodeGo,
    CodeJava,
    CodeKotlin,
    CodeSwift,
    CodeCpp,
    CodeC,
    CodeSql,
    CodeShell,
    CodeDockerfile,

    // --- Documents / binary formats ---
    Pdf,
    Docx,
    Pptx,
    Xlsx,
    Parquet,
    Avro,

    // --- Images ---
    ImagePng,
    ImageJpeg,
    ImageWebp,
    ImageGif,
    ImageSvg,

    // --- Audio / Video ---
    AudioMp3,
    AudioWav,
    AudioM4a,
    VideoMp4,
    VideoWebm,

    // --- Archives ---
    ArchiveZip,
    ArchiveTar,
    ArchiveGzip,

    // --- Fallbacks ---
    /// Known to be text, but not otherwise classified.
    Text,
    /// Known to be binary, but not otherwise classified.
    Binary,
    /// Completely unknown - we don't even know if it's text or binary.
    Unknown,
}

impl Default for MediaType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl MediaType {
    pub fn as_mime_str(self) -> &'static str {
        match self {
            Self::TextPlain => "text/plain",
            Self::TextMarkdown => "text/markdown",
            Self::TextHtml => "text/html",
            Self::TextXml => "text/xml",
            Self::TextCsv => "text/csv",
            Self::TextToml => "application/toml",
            Self::TextYaml => "application/yaml",
            Self::TextJson => "application/json",

            Self::CodeRust => "text/x-rust",
            Self::CodeCSharp => "text/x-csharp",
            Self::CodeTypeScript => "text/x-typescript",
            Self::CodeJavaScript => "text/javascript",
            Self::CodePython => "text/x-python",
            Self::CodeGo => "text/x-go",
            Self::CodeJava => "text/x-java",
            Self::CodeKotlin => "text/x-kotlin",
            Self::CodeSwift => "text/x-swift",
            Self::CodeCpp => "text/x-c++",
            Self::CodeC => "text/x-c",
            Self::CodeSql => "application/sql",
            Self::CodeShell => "text/x-shellscript",
            Self::CodeDockerfile => "text/x-dockerfile-config",

            Self::Pdf => "application/pdf",
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Pptx => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
            Self::Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Self::Parquet => "application/x-parquet",
            Self::Avro => "application/avro",

            Self::ImagePng => "image/png",
            Self::ImageJpeg => "image/jpeg",
            Self::ImageWebp => "image/webp",
            Self::ImageGif => "image/gif",
            Self::ImageSvg => "image/svg+xml",

            Self::AudioMp3 => "audio/mpeg",
            Self::AudioWav => "audio/wav",
            Self::AudioM4a => "audio/mp4",
            Self::VideoMp4 => "video/mp4",
            Self::VideoWebm => "video/webm",

            Self::ArchiveZip => "application/zip",
            Self::ArchiveTar => "application/x-tar",
            Self::ArchiveGzip => "application/gzip",

            Self::Text => "text/plain",
            Self::Binary => "application/octet-stream",
            Self::Unknown => "application/octet-stream",
        }
    }

    pub fn is_textlike(self) -> bool {
        matches!(
            self,
            Self::TextPlain
                | Self::TextMarkdown
                | Self::TextHtml
                | Self::TextXml
                | Self::TextCsv
                | Self::TextToml
                | Self::TextYaml
                | Self::TextJson
                | Self::CodeRust
                | Self::CodeCSharp
                | Self::CodeTypeScript
                | Self::CodeJavaScript
                | Self::CodePython
                | Self::CodeGo
                | Self::CodeJava
                | Self::CodeKotlin
                | Self::CodeSwift
                | Self::CodeCpp
                | Self::CodeC
                | Self::CodeSql
                | Self::CodeShell
                | Self::CodeDockerfile
                | Self::Text
        )
    }

    pub fn format(self) -> Format {
        match self {
            Self::TextJson => Format::Json,
            Self::TextYaml => Format::Yaml,
            Self::TextToml => Format::Toml,
            Self::TextXml => Format::Xml,
            Self::TextCsv => Format::Csv,
            Self::TextMarkdown => Format::Markdown,
            Self::TextHtml => Format::Html,
            Self::TextPlain
            | Self::Text
            | Self::CodeRust
            | Self::CodeCSharp
            | Self::CodeTypeScript
            | Self::CodeJavaScript
            | Self::CodePython
            | Self::CodeGo
            | Self::CodeJava
            | Self::CodeKotlin
            | Self::CodeSwift
            | Self::CodeCpp
            | Self::CodeC
            | Self::CodeSql
            | Self::CodeShell
            | Self::CodeDockerfile => Format::Text,
            _ => Format::Binary,
        }
    }

    /// Best-effort inference from a file path extension.
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase());

        match ext.as_deref() {
            Some("txt") => Self::TextPlain,
            Some("md") | Some("markdown") => Self::TextMarkdown,
            Some("html") | Some("htm") => Self::TextHtml,
            Some("xml") => Self::TextXml,
            Some("csv") => Self::TextCsv,
            Some("toml") => Self::TextToml,
            Some("yaml") | Some("yml") => Self::TextYaml,
            Some("json") => Self::TextJson,

            Some("rs") => Self::CodeRust,
            Some("cs") => Self::CodeCSharp,
            Some("ts") => Self::CodeTypeScript,
            Some("js") | Some("mjs") | Some("cjs") => Self::CodeJavaScript,
            Some("py") => Self::CodePython,
            Some("go") => Self::CodeGo,
            Some("java") => Self::CodeJava,
            Some("kt") | Some("kts") => Self::CodeKotlin,
            Some("swift") => Self::CodeSwift,
            Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") | Some("hh") | Some("hxx") => {
                Self::CodeCpp
            }
            Some("c") | Some("h") => Self::CodeC,
            Some("sql") => Self::CodeSql,
            Some("sh") | Some("bash") | Some("zsh") | Some("fish") => Self::CodeShell,

            Some("pdf") => Self::Pdf,
            Some("docx") => Self::Docx,
            Some("pptx") => Self::Pptx,
            Some("xlsx") => Self::Xlsx,
            Some("parquet") => Self::Parquet,
            Some("avro") => Self::Avro,

            Some("png") => Self::ImagePng,
            Some("jpg") | Some("jpeg") => Self::ImageJpeg,
            Some("webp") => Self::ImageWebp,
            Some("gif") => Self::ImageGif,
            Some("svg") => Self::ImageSvg,

            Some("mp3") => Self::AudioMp3,
            Some("wav") => Self::AudioWav,
            Some("m4a") => Self::AudioM4a,
            Some("mp4") => Self::VideoMp4,
            Some("webm") => Self::VideoWebm,

            Some("zip") => Self::ArchiveZip,
            Some("tar") => Self::ArchiveTar,
            Some("gz") | Some("gzip") => Self::ArchiveGzip,

            _ => Self::Unknown,
        }
    }

    pub fn from_mime_str(mime: &str) -> Self {
        let m = mime.trim().to_ascii_lowercase();
        match m.as_str() {
            "text/plain" => Self::TextPlain,
            "text/markdown" => Self::TextMarkdown,
            "text/html" => Self::TextHtml,
            "text/xml" | "application/xml" => Self::TextXml,
            "text/csv" => Self::TextCsv,
            "application/toml" => Self::TextToml,
            "application/yaml" | "text/yaml" => Self::TextYaml,
            "application/json" | "text/json" => Self::TextJson,

            "application/pdf" => Self::Pdf,
            "application/octet-stream" => Self::Binary,
            "image/png" => Self::ImagePng,
            "image/jpeg" => Self::ImageJpeg,
            "image/webp" => Self::ImageWebp,
            "image/gif" => Self::ImageGif,
            "image/svg+xml" => Self::ImageSvg,

            "audio/mpeg" => Self::AudioMp3,
            "audio/wav" => Self::AudioWav,
            "audio/mp4" => Self::AudioM4a,
            "video/mp4" => Self::VideoMp4,
            "video/webm" => Self::VideoWebm,

            "application/zip" => Self::ArchiveZip,
            "application/x-tar" => Self::ArchiveTar,
            "application/gzip" => Self::ArchiveGzip,

            _ => {
                if m.starts_with("text/") {
                    Self::Text
                } else {
                    Self::Unknown
                }
            }
        }
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_mime_str())
    }
}
