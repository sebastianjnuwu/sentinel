pub fn manager_icon(manager: &str) -> &'static str {
    match manager {
        "npm" | "pnpm" | "yarn" | "bun" => "🟩",
        "go" => "🐹",
        "cargo" => "🦀",
        "pip" | "uv" => "🐍",
        "composer" => "🐘",
        "dotnet" => "🟣",
        "maven" | "gradle" => "☕",
        "bundler" => "💎",
        "pub" => "🐦",
        "hex" => "⚡",
        "swiftpm" => "🍎",
        "terraform" => "🧱",
        "docker" => "🐳",
        "actions" => "🐙",
        _ => "📦",
    }
}

pub fn severity_style(kind: &str) -> (&'static str, &'static str) {
    let c = crate::console::c;
    match kind {
        "MAJOR" => ("🔴", c("\x1b[1;31m")),
        "MINOR" => ("🟡", c("\x1b[1;33m")),
        "PATCH" => ("🟢", c("\x1b[1;32m")),
        _ => ("⚪", c("\x1b[37m")),
    }
}

pub fn update_cmd(manager: &str, has_major: bool) -> &'static str {
    match (manager, has_major) {
        ("npm", true) => "npm install <pkg>@latest",
        ("npm", false) => "npm update",
        ("pnpm", true) => "pnpm update --latest",
        ("pnpm", false) => "pnpm update",
        ("yarn", true) => "yarn upgrade --latest",
        ("yarn", false) => "yarn upgrade",
        ("bun", true) => "bun update --latest",
        ("bun", false) => "bun update",
        ("go", _) => "go get -u ./... && go mod tidy",
        ("cargo", true) => "cargo add <crate>@latest",
        ("cargo", false) => "cargo update",
        ("pip", _) => "pip install --upgrade <package>",
        ("composer", _) => "composer update",
        ("dotnet", _) => "dotnet add package <package>",
        ("maven", _) => "mvn versions:use-latest-releases",
        ("gradle", _) => "./gradlew dependencyUpdates",
        ("bundler", _) => "bundle update",
        ("pub", _) => "dart pub upgrade --major-versions",
        ("hex", _) => "mix deps.update --all",
        ("swiftpm", _) => "swift package update",
        ("terraform", _) => "terraform init -upgrade",
        ("docker", _) => "update image tag & docker compose pull",
        ("actions", _) => "update uses: tag in .github/workflows/*.yml",
        _ => "",
    }
}
