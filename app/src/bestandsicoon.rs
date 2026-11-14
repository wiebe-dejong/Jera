//! De iconen in de bestandsboom: GitLab voor bestanden en Catppuccin voor mappen, of die van Zed.
//! Welke, staat in de instellingen.
//! De svg's en hun licenties staan in `iconen/gitlab` en `iconen/catppuccin`.

use std::{collections::HashMap, path::Path, sync::{Arc, LazyLock}};

use gpui_kit::*;

use crate::instellingen;
use crate::thema::{self as t, Thema};

macro_rules! gitlab {
    ($($naam:literal),* $(,)?) => {
        &[$(($naam, include_bytes!(concat!("../iconen/gitlab/", $naam, ".svg")))),*]
    };
}

const GITLAB: &[(&str, &[u8])] = gitlab!(
    "rust", "python", "typescript", "typescript-def", "react", "javascript", "javascript-map", "json",
    "markdown", "mdx", "settings", "yaml", "html", "css", "sass", "less", "stylus", "console",
    "powershell", "lock", "image", "video", "audio", "pdf", "word", "powerpoint", "table", "zip",
    "database", "file", "log", "xml", "go", "java", "kotlin", "swift", "c", "h", "cpp", "hpp",
    "csharp", "ruby", "vue", "astro", "graphql", "terraform", "nix", "lua", "dart", "elixir",
    "erlang", "haskell", "r", "julia", "tex", "font", "exe", "key", "certificate", "email",
    "readme", "changelog", "contributing", "docker", "git", "gitlab", "npm", "yarn", "editorconfig",
    "prettier", "eslint", "webpack", "rollup", "babel", "jenkins", "test-ts", "test-js", "test-jsx",
    "todo", "favicon", "cmake",
);

const MAP: [&[u8]; 4] = [
    include_bytes!("../iconen/catppuccin/mocha_folder.svg"),
    include_bytes!("../iconen/catppuccin/mocha_folder_open.svg"),
    include_bytes!("../iconen/catppuccin/latte_folder.svg"),
    include_bytes!("../iconen/catppuccin/latte_folder_open.svg"),
];

fn beeld(bytes: &[u8]) -> Arc<Image> {
    Arc::new(Image::from_bytes(ImageFormat::Svg, bytes.to_vec()))
}

static BESTANDEN: LazyLock<HashMap<&'static str, Arc<Image>>> =
    LazyLock::new(|| GITLAB.iter().map(|(naam, bytes)| (*naam, beeld(bytes))).collect());
static MAPPEN: LazyLock<[Arc<Image>; 4]> = LazyLock::new(|| MAP.map(beeld));

macro_rules! zed {
    ($($naam:literal),* $(,)?) => {
        &[$(($naam, include_bytes!(concat!("../iconen/zed/", $naam, ".svg")))),*]
    };
}

const ZED_SVG: &[(&str, &[u8])] = zed!(
    "folder", "folder_open", "rust", "python", "typescript", "react", "javascript", "code", "book",
    "settings", "toml", "yaml", "html", "css", "sass", "terminal", "lock", "image", "video", "audio",
    "file", "archive", "database", "go", "java", "kotlin", "swift", "c", "cpp", "ruby", "vue", "astro",
    "graphql", "terraform", "nix", "lua", "dart", "elixir", "erlang", "haskell", "r", "julia", "font",
    "docker", "git", "gitlab", "package", "prettier", "eslint",
);

/// Het Zed-icoon bij een GitLab-naam.
fn zed_naam(gitlab: &str, ext: &str) -> &'static str {
    match gitlab {
        "typescript" | "typescript-def" | "test-ts" => "typescript",
        "react" | "test-jsx" => "react",
        "javascript" | "javascript-map" | "test-js" => "javascript",
        "json" | "csharp" => "code",
        "markdown" | "mdx" | "readme" | "changelog" | "contributing" | "todo" => "book",
        "settings" if ext == "toml" => "toml",
        "settings" | "editorconfig" => "settings",
        "sass" | "less" | "stylus" => "sass",
        "console" | "powershell" => "terminal",
        "lock" | "yarn" => "lock",
        "image" | "favicon" => "image",
        "zip" => "archive",
        "h" => "c",
        "hpp" => "cpp",
        "npm" => "package",
        n => ZED_SVG.iter().find(|(z, _)| *z == n).map_or("file", |(z, _)| z),
    }
}

fn zed_svg(naam: &str, th: &Thema) -> AnyElement {
    let bytes = ZED_SVG.iter().find(|(z, _)| *z == naam).map_or(&[][..], |(_, b)| b);
    svg().data(bytes).flex_none().size(px(t::ICOON)).text_color(rgb(th.mute)).into_any_element()
}

/// Het icoon van een map. Catppuccin heeft een donkere (mocha) en een lichte (latte) versie.
pub fn map(open: bool, th: &Thema) -> AnyElement {
    if instellingen::zed() {
        return zed_svg(if open { "folder_open" } else { "folder" }, th);
    }
    img(MAPPEN[open as usize + 2 * th.licht as usize].clone()).flex_none().size(px(t::ICOON)).into_any_element()
}

/// Het icoon van git, in dezelfde stijl als de bestanden.
pub fn git(th: &Thema) -> AnyElement {
    bestand(Path::new(".gitignore"), th)
}

/// Het icoon van een bestand, naar zijn naam.
pub fn bestand(pad: &Path, th: &Thema) -> AnyElement {
    let naam = pad.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
    let gitlab = soort(&naam);
    if instellingen::zed() {
        return zed_svg(zed_naam(gitlab, naam.rsplit_once('.').map_or("", |(_, e)| e)), th);
    }
    img(BESTANDEN[gitlab].clone()).flex_none().size(px(t::ICOON)).into_any_element()
}

/// De naam van het GitLab-icoon voor een bestandsnaam in kleine letters.
fn soort(naam: &str) -> &'static str {
    let begint = |s: &str| naam.starts_with(s);
    let eindigt = |s: &str| naam.ends_with(s);
    match naam {
        _ if begint("readme") => "readme",
        _ if begint("changelog") => "changelog",
        _ if begint("contributing") => "contributing",
        _ if begint("license") || begint("licence") || naam == "copying" => "certificate",
        _ if begint("todo") => "todo",
        _ if begint("dockerfile") || begint("docker-compose") || begint("compose.y") => "docker",
        ".gitlab-ci.yml" => "gitlab",
        ".gitignore" | ".gitattributes" | ".gitmodules" | ".gitkeep" => "git",
        "package.json" | "package-lock.json" | ".npmrc" => "npm",
        "yarn.lock" => "yarn",
        ".editorconfig" => "editorconfig",
        "jenkinsfile" => "jenkins",
        "cmakelists.txt" => "cmake",
        "favicon.ico" => "favicon",
        _ if begint(".prettierrc") || begint("prettier.config") => "prettier",
        _ if begint(".eslintrc") || begint("eslint.config") => "eslint",
        _ if begint("webpack.config") => "webpack",
        _ if begint("rollup.config") => "rollup",
        _ if begint(".babelrc") || begint("babel.config") => "babel",
        _ if begint(".env") => "key",
        _ if eindigt(".d.ts") => "typescript-def",
        _ if eindigt(".test.ts") || eindigt(".spec.ts") => "test-ts",
        _ if eindigt(".test.js") || eindigt(".spec.js") => "test-js",
        _ if eindigt(".test.tsx") || eindigt(".spec.tsx") || eindigt(".test.jsx") || eindigt(".spec.jsx") => "test-jsx",
        _ if eindigt(".js.map") => "javascript-map",
        _ => match naam.rsplit_once('.').map_or("", |(_, ext)| ext) {
            "rs" => "rust",
            "py" | "pyi" | "pyw" | "ipynb" => "python",
            "ts" | "mts" | "cts" => "typescript",
            "tsx" | "jsx" => "react",
            "js" | "mjs" | "cjs" => "javascript",
            "json" | "jsonc" | "json5" => "json",
            "md" | "markdown" => "markdown",
            "mdx" => "mdx",
            "toml" | "ini" | "cfg" | "conf" | "properties" => "settings",
            "yml" | "yaml" => "yaml",
            "html" | "htm" => "html",
            "css" => "css",
            "scss" | "sass" => "sass",
            "less" => "less",
            "styl" => "stylus",
            "sh" | "bash" | "zsh" | "fish" => "console",
            "ps1" => "powershell",
            "lock" => "lock",
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "tif" | "tiff" | "heic" | "svg" => "image",
            "mp4" | "mov" | "webm" | "mkv" | "avi" => "video",
            "mp3" | "wav" | "flac" | "m4a" | "ogg" => "audio",
            "pdf" => "pdf",
            "doc" | "docx" | "odt" | "rtf" => "word",
            "ppt" | "pptx" => "powerpoint",
            "xls" | "xlsx" | "csv" | "tsv" | "numbers" => "table",
            "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "7z" | "rar" => "zip",
            "sql" | "sqlite" | "db" => "database",
            "log" => "log",
            "xml" | "plist" => "xml",
            "go" => "go",
            "java" => "java",
            "kt" | "kts" => "kotlin",
            "swift" => "swift",
            "c" => "c",
            "h" => "h",
            "cpp" | "cc" | "cxx" => "cpp",
            "hpp" | "hh" => "hpp",
            "cs" => "csharp",
            "rb" => "ruby",
            "vue" => "vue",
            "astro" => "astro",
            "graphql" | "gql" => "graphql",
            "tf" => "terraform",
            "nix" => "nix",
            "lua" => "lua",
            "dart" => "dart",
            "ex" | "exs" => "elixir",
            "erl" => "erlang",
            "hs" => "haskell",
            "r" => "r",
            "jl" => "julia",
            "tex" => "tex",
            "ttf" | "otf" | "woff" | "woff2" => "font",
            "exe" => "exe",
            "pem" | "key" => "key",
            "crt" | "cer" => "certificate",
            "eml" => "email",
            _ => "file",
        },
    }
}
