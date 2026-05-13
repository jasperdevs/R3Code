use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSurfaceMetadata {
    pub name: &'static str,
    pub upstream_name: &'static str,
    pub version: &'static str,
    pub private: bool,
    pub module_type: &'static str,
    pub main: Option<&'static str>,
    pub product_name: Option<&'static str>,
    pub files: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopBuildEntry {
    pub entry: &'static str,
    pub format: &'static str,
    pub out_dir: &'static str,
    pub sourcemap: bool,
    pub clean: bool,
    pub js_extension: &'static str,
    pub no_external_prefixes: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopDevElectronPlan {
    pub required_files: Vec<&'static str>,
    pub watched_directories: Vec<(&'static str, Vec<&'static str>)>,
    pub forced_shutdown_timeout_ms: u64,
    pub restart_debounce_ms: u64,
    pub child_tree_grace_period_ms: u64,
    pub requires_explicit_dev_server_port: bool,
    pub clears_electron_run_as_node: bool,
    pub shutdown_signals: Vec<(&'static str, i32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopElectronLauncherPlan {
    pub launcher_version: u32,
    pub dev_display_name: &'static str,
    pub prod_display_name: &'static str,
    pub dev_bundle_id: &'static str,
    pub prod_bundle_id: &'static str,
    pub macos_runtime_dir: &'static str,
    pub macos_metadata_file: &'static str,
    pub macos_dev_icon_source: &'static str,
    pub macos_icon_sizes: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopWaitForResourcesPlan {
    pub default_tcp_hosts: Vec<&'static str>,
    pub interval_ms: u64,
    pub timeout_ms: u64,
    pub connect_timeout_ms: u64,
    pub requires_positive_tcp_port: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopSmokeTestPlan {
    pub electron_bin: &'static str,
    pub main_js: &'static str,
    pub timeout_ms: u64,
    pub forced_env: Vec<(&'static str, &'static str)>,
    pub fatal_patterns: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketingReleaseSurface {
    pub upstream_repo: &'static str,
    pub release_url: &'static str,
    pub api_url: &'static str,
    pub cache_key: &'static str,
    pub caches_only_when_assets_exist: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractsPackageSurface {
    pub metadata: PackageSurfaceMetadata,
    pub exports: BTreeMap<&'static str, BTreeMap<&'static str, &'static str>>,
    pub scripts: BTreeMap<&'static str, &'static str>,
    pub dependencies: Vec<&'static str>,
    pub dev_dependencies: Vec<&'static str>,
    pub tsconfig_extends: &'static str,
    pub tsconfig_include: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedPackageSurface {
    pub metadata: PackageSurfaceMetadata,
    pub exports: BTreeMap<&'static str, BTreeMap<&'static str, &'static str>>,
    pub scripts: BTreeMap<&'static str, &'static str>,
    pub dependencies: Vec<&'static str>,
    pub dev_dependencies: Vec<&'static str>,
    pub tsconfig_extends: &'static str,
    pub tsconfig_include: Vec<&'static str>,
}

pub fn desktop_package_metadata() -> PackageSurfaceMetadata {
    PackageSurfaceMetadata {
        name: "@r3tools/desktop",
        upstream_name: "@t3tools/desktop",
        version: "0.0.23",
        private: true,
        module_type: "module",
        main: Some("dist-electron/main.cjs"),
        product_name: Some("R3 Code (Alpha)"),
        files: Vec::new(),
    }
}

pub fn desktop_package_scripts() -> BTreeMap<&'static str, &'static str> {
    BTreeMap::from([
        ("dev", "bun run --parallel dev:bundle dev:electron"),
        ("dev:bundle", "tsdown --watch"),
        ("dev:electron", "node scripts/dev-electron.mjs"),
        ("build", "tsdown"),
        ("start", "node scripts/start-electron.mjs"),
        ("typecheck", "tsc --noEmit"),
        ("test", "vitest run --passWithNoTests"),
        ("smoke-test", "node scripts/smoke-test.mjs"),
    ])
}

pub fn desktop_package_dependencies() -> Vec<&'static str> {
    vec![
        "@effect/platform-node",
        "effect",
        "electron",
        "electron-updater",
    ]
}

pub fn desktop_workspace_dev_dependencies() -> Vec<&'static str> {
    vec![
        "@t3tools/client-runtime",
        "@t3tools/contracts",
        "@t3tools/shared",
        "@t3tools/ssh",
        "@t3tools/tailscale",
        "effect-acp",
    ]
}

pub fn desktop_tsdown_entries() -> Vec<DesktopBuildEntry> {
    vec![
        DesktopBuildEntry {
            entry: "src/main.ts",
            format: "cjs",
            out_dir: "dist-electron",
            sourcemap: true,
            clean: true,
            js_extension: ".cjs",
            no_external_prefixes: vec!["@t3tools/", "effect-acp"],
        },
        DesktopBuildEntry {
            entry: "src/preload.ts",
            format: "cjs",
            out_dir: "dist-electron",
            sourcemap: true,
            clean: false,
            js_extension: ".cjs",
            no_external_prefixes: Vec::new(),
        },
    ]
}

pub fn desktop_dev_electron_plan() -> DesktopDevElectronPlan {
    DesktopDevElectronPlan {
        required_files: vec![
            "dist-electron/main.cjs",
            "dist-electron/preload.cjs",
            "../server/dist/bin.mjs",
        ],
        watched_directories: vec![
            ("dist-electron", vec!["main.cjs", "preload.cjs"]),
            ("../server/dist", vec!["bin.mjs"]),
        ],
        forced_shutdown_timeout_ms: 1_500,
        restart_debounce_ms: 120,
        child_tree_grace_period_ms: 1_200,
        requires_explicit_dev_server_port: true,
        clears_electron_run_as_node: true,
        shutdown_signals: vec![("SIGINT", 130), ("SIGTERM", 143), ("SIGHUP", 129)],
    }
}

pub fn desktop_electron_launcher_plan() -> DesktopElectronLauncherPlan {
    DesktopElectronLauncherPlan {
        launcher_version: 2,
        dev_display_name: "R3 Code (Dev)",
        prod_display_name: "R3 Code (Alpha)",
        dev_bundle_id: "com.r3tools.r3code.dev",
        prod_bundle_id: "com.r3tools.r3code",
        macos_runtime_dir: ".electron-runtime",
        macos_metadata_file: "metadata.json",
        macos_dev_icon_source: "assets/dev/blueprint-macos-1024.png",
        macos_icon_sizes: vec![16, 32, 128, 256, 512],
    }
}

pub fn desktop_wait_for_resources_plan() -> DesktopWaitForResourcesPlan {
    DesktopWaitForResourcesPlan {
        default_tcp_hosts: vec!["127.0.0.1", "localhost", "::1"],
        interval_ms: 100,
        timeout_ms: 120_000,
        connect_timeout_ms: 500,
        requires_positive_tcp_port: true,
    }
}

pub fn desktop_smoke_test_plan() -> DesktopSmokeTestPlan {
    DesktopSmokeTestPlan {
        electron_bin: "node_modules/.bin/electron",
        main_js: "dist-electron/main.cjs",
        timeout_ms: 8_000,
        forced_env: vec![
            ("VITE_DEV_SERVER_URL", ""),
            ("ELECTRON_ENABLE_LOGGING", "1"),
        ],
        fatal_patterns: vec![
            "Cannot find module",
            "MODULE_NOT_FOUND",
            "Refused to execute",
            "Uncaught Error",
            "Uncaught TypeError",
            "Uncaught ReferenceError",
        ],
    }
}

pub fn marketing_package_metadata() -> PackageSurfaceMetadata {
    PackageSurfaceMetadata {
        name: "@r3tools/marketing",
        upstream_name: "@t3tools/marketing",
        version: "0.0.0",
        private: true,
        module_type: "module",
        main: None,
        product_name: None,
        files: Vec::new(),
    }
}

pub fn marketing_package_scripts() -> BTreeMap<&'static str, &'static str> {
    BTreeMap::from([
        ("dev", "astro dev"),
        ("build", "astro build"),
        ("preview", "astro preview"),
        ("typecheck", "astro check"),
    ])
}

pub fn marketing_release_surface() -> MarketingReleaseSurface {
    MarketingReleaseSurface {
        upstream_repo: "pingdotgg/t3code",
        release_url: "https://github.com/pingdotgg/t3code/releases",
        api_url: "https://api.github.com/repos/pingdotgg/t3code/releases/latest",
        cache_key: "r3code-latest-release",
        caches_only_when_assets_exist: true,
    }
}

pub fn contracts_package_surface() -> ContractsPackageSurface {
    ContractsPackageSurface {
        metadata: PackageSurfaceMetadata {
            name: "@r3tools/contracts",
            upstream_name: "@t3tools/contracts",
            version: "0.0.23",
            private: true,
            module_type: "module",
            main: Some("./dist/index.cjs"),
            product_name: None,
            files: vec!["dist"],
        },
        exports: BTreeMap::from([
            (
                ".",
                BTreeMap::from([
                    ("types", "./src/index.ts"),
                    ("import", "./src/index.ts"),
                    ("require", "./dist/index.cjs"),
                ]),
            ),
            (
                "./settings",
                BTreeMap::from([
                    ("types", "./src/settings.ts"),
                    ("import", "./src/settings.ts"),
                    ("require", "./src/settings.ts"),
                ]),
            ),
        ]),
        scripts: BTreeMap::from([
            (
                "dev",
                "tsdown src/index.ts --format esm,cjs --dts --watch --clean",
            ),
            (
                "build",
                "tsdown src/index.ts --format esm,cjs --dts --clean",
            ),
            ("typecheck", "tsc --noEmit"),
            ("test", "vitest run"),
        ]),
        dependencies: vec!["effect"],
        dev_dependencies: vec![
            "@effect/language-service",
            "@effect/vitest",
            "tsdown",
            "typescript",
            "vitest",
        ],
        tsconfig_extends: "../../tsconfig.base.json",
        tsconfig_include: vec!["src"],
    }
}

pub fn shared_package_surface() -> SharedPackageSurface {
    SharedPackageSurface {
        metadata: PackageSurfaceMetadata {
            name: "@r3tools/shared",
            upstream_name: "@t3tools/shared",
            version: "0.0.0-alpha.1",
            private: true,
            module_type: "module",
            main: None,
            product_name: None,
            files: Vec::new(),
        },
        exports: BTreeMap::from([
            shared_export("./model", "./src/model.ts"),
            shared_export("./git", "./src/git.ts"),
            shared_export("./sourceControl", "./src/sourceControl.ts"),
            shared_export("./logging", "./src/logging.ts"),
            shared_export("./observability", "./src/observability.ts"),
            shared_export("./shell", "./src/shell.ts"),
            shared_export("./semver", "./src/semver.ts"),
            shared_export("./Net", "./src/Net.ts"),
            shared_export("./DrainableWorker", "./src/DrainableWorker.ts"),
            shared_export("./KeyedCoalescingWorker", "./src/KeyedCoalescingWorker.ts"),
            shared_export("./schemaJson", "./src/schemaJson.ts"),
            shared_export("./toolActivity", "./src/toolActivity.ts"),
            shared_export("./Struct", "./src/Struct.ts"),
            shared_export("./serverSettings", "./src/serverSettings.ts"),
            shared_export("./String", "./src/String.ts"),
            shared_export("./projectScripts", "./src/projectScripts.ts"),
            shared_export("./searchRanking", "./src/searchRanking.ts"),
            shared_export("./qrCode", "./src/qrCode.ts"),
            shared_export("./cliArgs", "./src/cliArgs.ts"),
            shared_export("./path", "./src/path.ts"),
            shared_export("./keybindings", "./src/keybindings.ts"),
        ]),
        scripts: BTreeMap::from([("typecheck", "tsc --noEmit"), ("test", "vitest run")]),
        dependencies: vec!["@t3tools/contracts", "effect"],
        dev_dependencies: vec![
            "@effect/language-service",
            "@effect/platform-node",
            "@effect/vitest",
            "@types/node",
            "typescript",
            "vitest",
        ],
        tsconfig_extends: "../../tsconfig.base.json",
        tsconfig_include: vec!["src"],
    }
}

fn shared_export(
    key: &'static str,
    source: &'static str,
) -> (&'static str, BTreeMap<&'static str, &'static str>) {
    (key, BTreeMap::from([("types", source), ("import", source)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ports_desktop_package_scripts_and_launcher_contracts() {
        let package = desktop_package_metadata();
        assert_eq!(package.name, "@r3tools/desktop");
        assert_eq!(package.upstream_name, "@t3tools/desktop");
        assert_eq!(package.main, Some("dist-electron/main.cjs"));
        assert_eq!(package.product_name, Some("R3 Code (Alpha)"));
        assert_eq!(
            desktop_package_scripts()["dev"],
            "bun run --parallel dev:bundle dev:electron"
        );
        assert_eq!(
            desktop_package_scripts()["smoke-test"],
            "node scripts/smoke-test.mjs"
        );
        assert!(desktop_package_dependencies().contains(&"electron"));
        assert!(desktop_workspace_dev_dependencies().contains(&"effect-acp"));

        let entries = desktop_tsdown_entries();
        assert_eq!(entries[0].entry, "src/main.ts");
        assert_eq!(
            entries[0].no_external_prefixes,
            vec!["@t3tools/", "effect-acp"]
        );
        assert_eq!(entries[1].entry, "src/preload.ts");
        assert_eq!(entries[1].js_extension, ".cjs");

        let dev = desktop_dev_electron_plan();
        assert_eq!(dev.required_files[2], "../server/dist/bin.mjs");
        assert_eq!(dev.restart_debounce_ms, 120);
        assert_eq!(
            dev.shutdown_signals,
            vec![("SIGINT", 130), ("SIGTERM", 143), ("SIGHUP", 129)]
        );

        let launcher = desktop_electron_launcher_plan();
        assert_eq!(launcher.launcher_version, 2);
        assert_eq!(launcher.dev_display_name, "R3 Code (Dev)");
        assert_eq!(launcher.prod_bundle_id, "com.r3tools.r3code");
        assert_eq!(launcher.macos_icon_sizes, vec![16, 32, 128, 256, 512]);

        let wait = desktop_wait_for_resources_plan();
        assert_eq!(
            wait.default_tcp_hosts,
            vec!["127.0.0.1", "localhost", "::1"]
        );
        assert_eq!(wait.timeout_ms, 120_000);

        let smoke = desktop_smoke_test_plan();
        assert_eq!(smoke.timeout_ms, 8_000);
        assert!(smoke.fatal_patterns.contains(&"MODULE_NOT_FOUND"));
    }

    #[test]
    fn ports_marketing_release_and_contracts_package_surfaces() {
        let marketing = marketing_package_metadata();
        assert_eq!(marketing.name, "@r3tools/marketing");
        assert_eq!(marketing.upstream_name, "@t3tools/marketing");
        assert_eq!(marketing_package_scripts()["typecheck"], "astro check");
        let release = marketing_release_surface();
        assert_eq!(release.upstream_repo, "pingdotgg/t3code");
        assert_eq!(
            release.release_url,
            "https://github.com/pingdotgg/t3code/releases"
        );
        assert_eq!(release.cache_key, "r3code-latest-release");
        assert!(release.caches_only_when_assets_exist);

        let contracts = contracts_package_surface();
        assert_eq!(contracts.metadata.name, "@r3tools/contracts");
        assert_eq!(contracts.metadata.main, Some("./dist/index.cjs"));
        assert_eq!(contracts.exports["."]["require"], "./dist/index.cjs");
        assert_eq!(
            contracts.exports["./settings"]["import"],
            "./src/settings.ts"
        );
        assert_eq!(
            contracts.scripts["build"],
            "tsdown src/index.ts --format esm,cjs --dts --clean"
        );
        assert_eq!(contracts.tsconfig_extends, "../../tsconfig.base.json");
        assert_eq!(contracts.tsconfig_include, vec!["src"]);
    }

    #[test]
    fn ports_shared_package_surface() {
        let shared = shared_package_surface();
        assert_eq!(shared.metadata.name, "@r3tools/shared");
        assert_eq!(shared.metadata.upstream_name, "@t3tools/shared");
        assert_eq!(shared.metadata.version, "0.0.0-alpha.1");
        assert!(shared.metadata.private);
        assert_eq!(shared.metadata.module_type, "module");
        assert_eq!(shared.exports.len(), 21);
        assert_eq!(shared.exports["./qrCode"]["types"], "./src/qrCode.ts");
        assert_eq!(
            shared.exports["./keybindings"]["import"],
            "./src/keybindings.ts"
        );
        assert_eq!(shared.scripts["typecheck"], "tsc --noEmit");
        assert_eq!(shared.scripts["test"], "vitest run");
        assert_eq!(shared.dependencies, vec!["@t3tools/contracts", "effect"]);
        assert!(shared.dev_dependencies.contains(&"@effect/platform-node"));
        assert!(shared.dev_dependencies.contains(&"@types/node"));
        assert_eq!(shared.tsconfig_extends, "../../tsconfig.base.json");
        assert_eq!(shared.tsconfig_include, vec!["src"]);
    }
}
