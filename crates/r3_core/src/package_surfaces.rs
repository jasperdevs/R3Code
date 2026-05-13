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
    pub dependencies: BTreeMap<&'static str, &'static str>,
    pub dev_dependencies: BTreeMap<&'static str, &'static str>,
    pub tsconfig_extends: &'static str,
    pub tsconfig_include: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedPackageSurface {
    pub metadata: PackageSurfaceMetadata,
    pub exports: BTreeMap<&'static str, BTreeMap<&'static str, &'static str>>,
    pub scripts: BTreeMap<&'static str, &'static str>,
    pub dependencies: BTreeMap<&'static str, &'static str>,
    pub dev_dependencies: BTreeMap<&'static str, &'static str>,
    pub tsconfig_extends: &'static str,
    pub tsconfig_include: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtilityPackageSurface {
    pub metadata: PackageSurfaceMetadata,
    pub exports: BTreeMap<&'static str, BTreeMap<&'static str, &'static str>>,
    pub scripts: BTreeMap<&'static str, &'static str>,
    pub dependencies: BTreeMap<&'static str, &'static str>,
    pub dev_dependencies: BTreeMap<&'static str, &'static str>,
    pub tsconfig_extends: &'static str,
    pub tsconfig_include: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebPackageSurface {
    pub metadata: PackageSurfaceMetadata,
    pub scripts: BTreeMap<&'static str, &'static str>,
    pub dependencies: BTreeMap<&'static str, &'static str>,
    pub dev_dependencies: BTreeMap<&'static str, &'static str>,
    pub tsconfig: WebTsConfigSurface,
    pub components: WebComponentsRegistrySurface,
    pub index_html: WebIndexHtmlSurface,
    pub vite: WebViteConfigSurface,
    pub vitest_browser: WebVitestBrowserConfigSurface,
    pub vercel: WebVercelConfigSurface,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebTsConfigSurface {
    pub extends: &'static str,
    pub composite: bool,
    pub module: &'static str,
    pub module_resolution: &'static str,
    pub erasable_syntax_only: bool,
    pub verbatim_module_syntax: bool,
    pub jsx: &'static str,
    pub lib: Vec<&'static str>,
    pub types: Vec<&'static str>,
    pub paths: BTreeMap<&'static str, Vec<&'static str>>,
    pub effect_plugin_name: &'static str,
    pub effect_namespace_import_packages: Vec<&'static str>,
    pub effect_diagnostic_severity: BTreeMap<&'static str, &'static str>,
    pub include: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebComponentsRegistrySurface {
    pub schema: &'static str,
    pub style: &'static str,
    pub rsc: bool,
    pub tsx: bool,
    pub tailwind_css: &'static str,
    pub base_color: &'static str,
    pub css_variables: bool,
    pub icon_library: &'static str,
    pub rtl: bool,
    pub menu_color: &'static str,
    pub menu_accent: &'static str,
    pub aliases: BTreeMap<&'static str, &'static str>,
    pub registries: BTreeMap<&'static str, &'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebIndexHtmlSurface {
    pub lang: &'static str,
    pub charset: &'static str,
    pub viewport: &'static str,
    pub theme_colors: Vec<(&'static str, Option<&'static str>)>,
    pub icon_href: &'static str,
    pub apple_touch_icon_href: &'static str,
    pub light_background: &'static str,
    pub dark_background: &'static str,
    pub theme_storage_key: &'static str,
    pub upstream_theme_storage_key: &'static str,
    pub default_theme: &'static str,
    pub font_stylesheet_href: &'static str,
    pub title: &'static str,
    pub upstream_title: &'static str,
    pub root_id: &'static str,
    pub boot_shell_id: &'static str,
    pub boot_shell_card_id: &'static str,
    pub boot_shell_logo_id: &'static str,
    pub boot_shell_card_size_px: u32,
    pub boot_shell_logo_size_px: u32,
    pub splash_aria_label: &'static str,
    pub upstream_splash_aria_label: &'static str,
    pub logo_src: &'static str,
    pub logo_alt: &'static str,
    pub upstream_logo_alt: &'static str,
    pub main_script: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebBuildSourcemap {
    Enabled,
    Disabled,
    Hidden,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebViteConfigSurface {
    pub default_port: u16,
    pub default_host: &'static str,
    pub ws_url_env: &'static str,
    pub hosted_app_url_env: &'static str,
    pub hosted_app_channel_env: &'static str,
    pub app_version_env: &'static str,
    pub upstream_sourcemap_env: &'static str,
    pub sourcemap_env: &'static str,
    pub plugins: Vec<&'static str>,
    pub babel_parser_plugins: Vec<&'static str>,
    pub babel_preset: &'static str,
    pub optimize_deps_include: Vec<&'static str>,
    pub define_keys: Vec<&'static str>,
    pub tsconfig_paths: bool,
    pub server_strict_port: bool,
    pub proxy_paths: Vec<&'static str>,
    pub proxy_change_origin: bool,
    pub hmr_protocol: &'static str,
    pub build_out_dir: &'static str,
    pub build_empty_out_dir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebVitestBrowserConfigSurface {
    pub merged_from_vite_config: bool,
    pub src_alias: &'static str,
    pub src_alias_target: &'static str,
    pub server_strict_port: bool,
    pub include: Vec<&'static str>,
    pub browser_enabled: bool,
    pub provider: &'static str,
    pub instances: Vec<&'static str>,
    pub headless: bool,
    pub api_strict_port: bool,
    pub test_timeout_ms: u64,
    pub hook_timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebVercelConfigSurface {
    pub build_command: &'static str,
    pub install_command: &'static str,
    pub deployment_enabled: bool,
    pub router_host: &'static str,
    pub upstream_router_host: &'static str,
    pub hosted_web_channel_cookie: &'static str,
    pub upstream_hosted_web_channel_cookie: &'static str,
    pub latest_origin: &'static str,
    pub upstream_latest_origin: &'static str,
    pub nightly_origin: &'static str,
    pub upstream_nightly_origin: &'static str,
    pub channel_route: &'static str,
    pub channel_query_key: &'static str,
    pub channels: Vec<&'static str>,
    pub clean_channel_query_transform: (&'static str, &'static str, &'static str),
    pub channel_cookie_parts: Vec<&'static str>,
    pub app_rewrite_source: &'static str,
    pub app_rewrite_destination: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebIndexCssSurface {
    pub imports: Vec<&'static str>,
    pub custom_variants: Vec<(&'static str, &'static str)>,
    pub theme_inline_vars: Vec<&'static str>,
    pub root_light_tokens: BTreeMap<&'static str, &'static str>,
    pub root_dark_tokens: BTreeMap<&'static str, &'static str>,
    pub safe_area_utilities: Vec<(&'static str, &'static str)>,
    pub body_font_stack: Vec<&'static str>,
    pub root_layout_rules: BTreeMap<&'static str, &'static str>,
    pub noise_overlay_opacity: &'static str,
    pub scrollbar_width_px: u32,
    pub terminal_scrollbar_width_px: u32,
    pub class_hooks: Vec<&'static str>,
    pub keyframes: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebViteEnvSurface {
    pub reference_types: &'static str,
    pub env_keys: Vec<&'static str>,
    pub window_globals: BTreeMap<&'static str, &'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebAuthHttpHandlersSurface {
    pub test_session_expires_at: &'static str,
    pub environment_id: &'static str,
    pub label: &'static str,
    pub platform_os: &'static str,
    pub platform_arch: &'static str,
    pub server_version: &'static str,
    pub repository_identity_capability: bool,
    pub routes: Vec<(&'static str, &'static str)>,
    pub session_method: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebWsRpcHarnessSurface {
    pub parser: &'static str,
    pub server_factory: &'static str,
    pub stream_methods: Vec<&'static str>,
    pub records_requests: bool,
    pub responds_to_ping_with_pong: bool,
    pub resets_scope_before_reconnect: bool,
    pub stream_pubsub_kind: &'static str,
    pub missing_stream_error_prefix: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebMockServiceWorkerSurface {
    pub package_version: &'static str,
    pub integrity_checksum: &'static str,
    pub install_skip_waiting: bool,
    pub activate_claims_clients: bool,
    pub message_types: Vec<&'static str>,
    pub bypasses_navigation_requests: bool,
    pub bypasses_only_if_cached_cross_origin: bool,
    pub unregisters_when_last_client_closes: bool,
    pub response_event_type: &'static str,
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

pub fn web_package_surface() -> WebPackageSurface {
    WebPackageSurface {
        metadata: PackageSurfaceMetadata {
            name: "@r3tools/web",
            upstream_name: "@t3tools/web",
            version: "0.0.23",
            private: true,
            module_type: "module",
            main: None,
            product_name: None,
            files: Vec::new(),
        },
        scripts: BTreeMap::from([
            ("dev", "vite"),
            ("build", "vite build"),
            ("preview", "vite preview"),
            ("typecheck", "tsc --noEmit"),
            ("test", "vitest run --passWithNoTests"),
            (
                "test:browser",
                "vitest run --config vitest.browser.config.ts",
            ),
            (
                "test:browser:install",
                "playwright install --with-deps chromium",
            ),
        ]),
        dependencies: BTreeMap::from([
            ("@base-ui/react", "^1.4.1"),
            ("@dnd-kit/core", "^6.3.1"),
            ("@dnd-kit/modifiers", "^9.0.0"),
            ("@dnd-kit/sortable", "^10.0.0"),
            ("@dnd-kit/utilities", "^3.2.2"),
            ("@effect/atom-react", "catalog:"),
            ("@formkit/auto-animate", "^0.9.0"),
            ("@legendapp/list", "3.0.0-beta.44"),
            ("@lexical/react", "^0.41.0"),
            ("@pierre/diffs", "catalog:"),
            ("@t3tools/client-runtime", "workspace:*"),
            ("@t3tools/contracts", "workspace:*"),
            ("@t3tools/shared", "workspace:*"),
            ("@tanstack/react-pacer", "^0.19.4"),
            ("@tanstack/react-query", "^5.90.0"),
            ("@tanstack/react-router", "^1.160.2"),
            ("@xterm/addon-fit", "^0.11.0"),
            ("@xterm/xterm", "^6.0.0"),
            ("class-variance-authority", "^0.7.1"),
            ("effect", "catalog:"),
            ("lexical", "^0.41.0"),
            ("lucide-react", "^0.564.0"),
            ("react", "19.2.6"),
            ("react-dom", "19.2.6"),
            ("react-markdown", "^10.1.0"),
            ("remark-gfm", "^4.0.1"),
            ("tailwind-merge", "^3.4.0"),
            ("zustand", "^5.0.11"),
        ]),
        dev_dependencies: BTreeMap::from([
            ("@effect/language-service", "catalog:"),
            ("@rolldown/plugin-babel", "^0.2.0"),
            ("@tailwindcss/vite", "^4.0.0"),
            ("@tanstack/router-plugin", "^1.161.0"),
            ("@types/babel__core", "^7.20.5"),
            ("@types/react", "~19.2.14"),
            ("@types/react-dom", "~19.2.3"),
            ("@vercel/config", "^0.3.0"),
            ("@vitejs/plugin-react", "^6.0.0"),
            ("@vitest/browser-playwright", "^4.0.18"),
            ("babel-plugin-react-compiler", "1.0.0"),
            ("msw", "2.12.11"),
            ("playwright", "^1.58.2"),
            ("tailwindcss", "^4.0.0"),
            ("typescript", "catalog:"),
            ("vite", "^8.0.0"),
            ("vitest", "catalog:"),
            ("vitest-browser-react", "^2.0.5"),
        ]),
        tsconfig: web_tsconfig_surface(),
        components: web_components_registry_surface(),
        index_html: web_index_html_surface(),
        vite: web_vite_config_surface(),
        vitest_browser: web_vitest_browser_config_surface(),
        vercel: web_vercel_config_surface(),
    }
}

pub fn web_tsconfig_surface() -> WebTsConfigSurface {
    WebTsConfigSurface {
        extends: "../../tsconfig.base.json",
        composite: true,
        module: "Preserve",
        module_resolution: "Bundler",
        erasable_syntax_only: false,
        verbatim_module_syntax: false,
        jsx: "react-jsx",
        lib: vec!["ES2023", "DOM", "DOM.Iterable"],
        types: vec!["vite/client"],
        paths: BTreeMap::from([("~/*", vec!["./src/*"])]),
        effect_plugin_name: "@effect/language-service",
        effect_namespace_import_packages: vec!["@effect/platform-node", "effect"],
        effect_diagnostic_severity: BTreeMap::from([
            ("importFromBarrel", "error"),
            ("anyUnknownInErrorContext", "error"),
            ("unsafeEffectTypeAssertion", "error"),
            ("instanceOfSchema", "error"),
            ("deterministicKeys", "error"),
            ("strictEffectProvide", "off"),
            ("missingEffectServiceDependency", "error"),
            ("leakingRequirements", "error"),
            ("globalErrorInEffectCatch", "error"),
            ("globalErrorInEffectFailure", "error"),
            ("unknownInEffectCatch", "error"),
            ("strictBooleanExpressions", "off"),
            ("preferSchemaOverJson", "error"),
            ("schemaSyncInEffect", "error"),
        ]),
        include: vec!["src", "vite.config.ts", "vercel.ts", "test"],
    }
}

pub fn web_components_registry_surface() -> WebComponentsRegistrySurface {
    WebComponentsRegistrySurface {
        schema: "https://ui.shadcn.com/schema.json",
        style: "base-mira",
        rsc: false,
        tsx: true,
        tailwind_css: "src/index.css",
        base_color: "zinc",
        css_variables: true,
        icon_library: "lucide",
        rtl: false,
        menu_color: "default",
        menu_accent: "bold",
        aliases: BTreeMap::from([
            ("components", "~/components"),
            ("utils", "~/lib/utils"),
            ("ui", "~/components/ui"),
            ("lib", "~/lib"),
            ("hooks", "~/hooks"),
        ]),
        registries: BTreeMap::from([("@coss", "https://coss.com/ui/r/{name}.json")]),
    }
}

pub fn web_index_html_surface() -> WebIndexHtmlSurface {
    WebIndexHtmlSurface {
        lang: "en",
        charset: "UTF-8",
        viewport: "width=device-width, initial-scale=1.0, viewport-fit=cover, interactive-widget=resizes-content",
        theme_colors: vec![
            ("#ffffff", Some("(prefers-color-scheme: light)")),
            ("#161616", Some("(prefers-color-scheme: dark)")),
            ("#161616", None),
        ],
        icon_href: "/favicon.ico",
        apple_touch_icon_href: "/apple-touch-icon.png",
        light_background: "#ffffff",
        dark_background: "#161616",
        theme_storage_key: "r3code:theme",
        upstream_theme_storage_key: "t3code:theme",
        default_theme: "system",
        font_stylesheet_href: "https://fonts.googleapis.com/css2?family=DM+Sans:ital,opsz,wght@0,9..40,300..800;1,9..40,300..800&display=swap",
        title: "R3 Code (Alpha)",
        upstream_title: "T3 Code (Alpha)",
        root_id: "root",
        boot_shell_id: "boot-shell",
        boot_shell_card_id: "boot-shell-card",
        boot_shell_logo_id: "boot-shell-logo",
        boot_shell_card_size_px: 96,
        boot_shell_logo_size_px: 64,
        splash_aria_label: "R3 Code splash screen",
        upstream_splash_aria_label: "T3 Code splash screen",
        logo_src: "/apple-touch-icon.png",
        logo_alt: "R3 Code",
        upstream_logo_alt: "T3 Code",
        main_script: "/src/main.tsx",
    }
}

pub fn web_vite_config_surface() -> WebViteConfigSurface {
    WebViteConfigSurface {
        default_port: 5733,
        default_host: "localhost",
        ws_url_env: "VITE_WS_URL",
        hosted_app_url_env: "VITE_HOSTED_APP_URL",
        hosted_app_channel_env: "VITE_HOSTED_APP_CHANNEL",
        app_version_env: "APP_VERSION",
        upstream_sourcemap_env: "T3CODE_WEB_SOURCEMAP",
        sourcemap_env: "R3CODE_WEB_SOURCEMAP",
        plugins: vec![
            "tanstackRouter",
            "@vitejs/plugin-react",
            "@rolldown/plugin-babel",
            "@tailwindcss/vite",
        ],
        babel_parser_plugins: vec!["typescript", "jsx"],
        babel_preset: "reactCompilerPreset",
        optimize_deps_include: vec![
            "@pierre/diffs",
            "@pierre/diffs/react",
            "@pierre/diffs/worker/worker.js",
            "effect/Array",
            "effect/Order",
        ],
        define_keys: vec![
            "import.meta.env.VITE_WS_URL",
            "import.meta.env.VITE_HOSTED_APP_URL",
            "import.meta.env.VITE_HOSTED_APP_CHANNEL",
            "import.meta.env.APP_VERSION",
        ],
        tsconfig_paths: true,
        server_strict_port: true,
        proxy_paths: vec!["/.well-known", "/api", "/attachments"],
        proxy_change_origin: true,
        hmr_protocol: "ws",
        build_out_dir: "dist",
        build_empty_out_dir: true,
    }
}

pub fn web_vitest_browser_config_surface() -> WebVitestBrowserConfigSurface {
    WebVitestBrowserConfigSurface {
        merged_from_vite_config: true,
        src_alias: "~",
        src_alias_target: "./src",
        server_strict_port: false,
        include: vec!["src/components/**/*.browser.tsx"],
        browser_enabled: true,
        provider: "playwright",
        instances: vec!["chromium"],
        headless: true,
        api_strict_port: false,
        test_timeout_ms: 30_000,
        hook_timeout_ms: 30_000,
    }
}

pub fn web_vercel_config_surface() -> WebVercelConfigSurface {
    WebVercelConfigSurface {
        build_command: "turbo build --filter @r3tools/web && bun ../../scripts/apply-web-brand-assets.ts --channel \"${VITE_HOSTED_APP_CHANNEL:-latest}\"",
        install_command: "bun add -g turbo && bun install --filter '@t3tools/contracts' --filter '@t3tools/client-runtime' --filter '@t3tools/scripts' --filter '@r3tools/web'",
        deployment_enabled: false,
        router_host: "app.r3.codes",
        upstream_router_host: "app.t3.codes",
        hosted_web_channel_cookie: "r3code_web_channel",
        upstream_hosted_web_channel_cookie: "t3code_web_channel",
        latest_origin: "https://latest.app.r3.codes",
        upstream_latest_origin: "https://latest.app.t3.codes",
        nightly_origin: "https://nightly.app.r3.codes",
        upstream_nightly_origin: "https://nightly.app.t3.codes",
        channel_route: "/__r3code/channel",
        channel_query_key: "channel",
        channels: vec!["latest", "nightly"],
        clean_channel_query_transform: ("request.query", "delete", "channel"),
        channel_cookie_parts: vec![
            "Path=/",
            "Max-Age=31536000",
            "HttpOnly",
            "Secure",
            "SameSite=Lax",
        ],
        app_rewrite_source: "/(.*)",
        app_rewrite_destination: "/index.html",
    }
}

pub fn web_index_css_surface() -> WebIndexCssSurface {
    WebIndexCssSurface {
        imports: vec!["tailwindcss"],
        custom_variants: vec![
            ("dark", "&:is(.dark, .dark *)"),
            ("wco", "&:is(.wco, .wco *)"),
        ],
        theme_inline_vars: vec![
            "--animate-skeleton",
            "--color-warning-foreground",
            "--color-warning",
            "--color-success-foreground",
            "--color-success",
            "--color-info-foreground",
            "--color-info",
            "--color-destructive-foreground",
            "--color-ring",
            "--color-input",
            "--color-border",
            "--color-destructive",
            "--color-accent-foreground",
            "--color-accent",
            "--color-muted-foreground",
            "--color-muted",
            "--color-secondary-foreground",
            "--color-secondary",
            "--color-primary-foreground",
            "--color-primary",
            "--color-popover-foreground",
            "--color-popover",
            "--color-card-foreground",
            "--color-card",
            "--color-foreground",
            "--color-background",
            "--radius-sm",
            "--radius-md",
            "--radius-lg",
            "--radius-xl",
            "--radius-2xl",
            "--radius-3xl",
            "--radius-4xl",
        ],
        root_light_tokens: BTreeMap::from([
            ("color-scheme", "light"),
            ("--radius", "0.625rem"),
            ("--background", "var(--color-white)"),
            ("--app-chrome-background", "var(--background)"),
            ("--foreground", "var(--color-neutral-800)"),
            ("--primary", "oklch(0.488 0.217 264)"),
            ("--ring", "oklch(0.488 0.217 264)"),
            ("--warning", "var(--color-amber-500)"),
        ]),
        root_dark_tokens: BTreeMap::from([
            ("color-scheme", "dark"),
            (
                "--background",
                "color-mix(in srgb, var(--color-neutral-950) 95%, var(--color-white))",
            ),
            ("--foreground", "var(--color-neutral-100)"),
            ("--primary", "oklch(0.588 0.217 264)"),
            ("--ring", "oklch(0.588 0.217 264)"),
            ("--warning-foreground", "var(--color-amber-400)"),
        ]),
        safe_area_utilities: vec![
            ("pt-safe", "padding-top"),
            ("pb-safe", "padding-bottom"),
            ("pl-safe", "padding-left"),
            ("pr-safe", "padding-right"),
        ],
        body_font_stack: vec![
            "DM Sans",
            "-apple-system",
            "BlinkMacSystemFont",
            "Segoe UI",
            "system-ui",
            "sans-serif",
        ],
        root_layout_rules: BTreeMap::from([
            (
                "html,body",
                "min-height: calc(100svh + env(safe-area-inset-top))",
            ),
            ("body", "height: 100%; overflow: hidden"),
            (
                "#root",
                "height: 100%; width: 100%; overflow-x: clip; overflow-y: hidden",
            ),
        ]),
        noise_overlay_opacity: "0.035",
        scrollbar_width_px: 6,
        terminal_scrollbar_width_px: 6,
        class_hooks: vec![
            ".no-transitions",
            ".drag-region",
            ".turn-chip-strip",
            ".thread-terminal-drawer",
            ".chat-markdown",
            ".chat-markdown-file-link",
            ".chat-markdown-codeblock",
            ".provider-update-pill-progress",
            ".diff-panel-viewport",
            ".diff-render-file",
            ".ultrathink-frame",
            ".ultrathink-chroma",
            ".ultrathink-pill",
            ".ultrathink-word",
            ".model-picker-list",
        ],
        keyframes: vec![
            "skeleton",
            "provider-update-pill-countdown",
            "ultrathink-rainbow",
            "ultrathink-chroma-shift",
        ],
    }
}

pub fn web_vite_env_surface() -> WebViteEnvSurface {
    WebViteEnvSurface {
        reference_types: "vite/client",
        env_keys: vec![
            "VITE_HTTP_URL",
            "VITE_WS_URL",
            "VITE_HOSTED_APP_URL",
            "VITE_HOSTED_APP_CHANNEL",
            "APP_VERSION",
        ],
        window_globals: BTreeMap::from([
            ("nativeApi", "LocalApi"),
            ("desktopBridge", "DesktopBridge"),
        ]),
    }
}

pub fn web_auth_http_handlers_surface() -> WebAuthHttpHandlersSurface {
    WebAuthHttpHandlersSurface {
        test_session_expires_at: "2026-05-01T12:00:00.000Z",
        environment_id: "environment-local",
        label: "Local environment",
        platform_os: "darwin",
        platform_arch: "arm64",
        server_version: "0.0.0-test",
        repository_identity_capability: true,
        routes: vec![
            ("GET", "*/.well-known/t3/environment"),
            ("GET", "*/api/auth/session"),
            ("POST", "*/api/auth/bootstrap"),
        ],
        session_method: "browser-session-cookie",
    }
}

pub fn web_ws_rpc_harness_surface() -> WebWsRpcHarnessSurface {
    WebWsRpcHarnessSurface {
        parser: "RpcSerialization.json.makeUnsafe",
        server_factory: "RpcServer.makeNoSerialization",
        stream_methods: vec![
            "orchestration.subscribeShell",
            "orchestration.subscribeThread",
            "gitRunStackedAction",
            "subscribeVcsStatus",
            "subscribeTerminalEvents",
            "subscribeServerConfig",
            "subscribeServerLifecycle",
        ],
        records_requests: true,
        responds_to_ping_with_pong: true,
        resets_scope_before_reconnect: true,
        stream_pubsub_kind: "PubSub.unbounded",
        missing_stream_error_prefix: "No stream registered for ",
    }
}

pub fn web_mock_service_worker_surface() -> WebMockServiceWorkerSurface {
    WebMockServiceWorkerSurface {
        package_version: "2.12.11",
        integrity_checksum: "4db4a41e972cec1b64cc569c66952d82",
        install_skip_waiting: true,
        activate_claims_clients: true,
        message_types: vec![
            "KEEPALIVE_REQUEST",
            "INTEGRITY_CHECK_REQUEST",
            "MOCK_ACTIVATE",
            "CLIENT_CLOSED",
        ],
        bypasses_navigation_requests: true,
        bypasses_only_if_cached_cross_origin: true,
        unregisters_when_last_client_closes: true,
        response_event_type: "RESPONSE",
    }
}

pub fn resolve_web_build_sourcemap(env: Option<&str>) -> WebBuildSourcemap {
    match env.map(|value| value.trim().to_ascii_lowercase()) {
        Some(value) if value == "0" || value == "false" => WebBuildSourcemap::Disabled,
        Some(value) if value == "hidden" => WebBuildSourcemap::Hidden,
        _ => WebBuildSourcemap::Enabled,
    }
}

pub fn resolve_web_dev_proxy_target(ws_url: Option<&str>) -> Option<String> {
    let ws_url = ws_url?.trim();
    if ws_url.is_empty() {
        return None;
    }
    let protocol_end = ws_url.find("://")?;
    let protocol = match &ws_url[..protocol_end] {
        "ws" => "http",
        "wss" => "https",
        other => other,
    };
    let rest = &ws_url[protocol_end + 3..];
    let authority_end = rest
        .find(|ch| ch == '/' || ch == '?' || ch == '#')
        .unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.is_empty() {
        return None;
    }
    Some(format!("{protocol}://{authority}/"))
}

pub fn resolve_web_hosted_app_url(
    explicit_hosted_app_url: Option<&str>,
    vercel_env: Option<&str>,
    vercel_project_production_url: Option<&str>,
    vercel_url: Option<&str>,
) -> Option<String> {
    if let Some(explicit_hosted_app_url) = trim_non_empty(explicit_hosted_app_url) {
        return Some(explicit_hosted_app_url.to_string());
    }
    if vercel_env == Some("production") {
        if let Some(production_url) = trim_non_empty(vercel_project_production_url) {
            return Some(format!("https://{production_url}"));
        }
    }
    trim_non_empty(vercel_url).map(|url| format!("https://{url}"))
}

pub fn resolve_web_host_and_port<'a>(host: Option<&'a str>, port: Option<&str>) -> (&'a str, u16) {
    let host = trim_non_empty(host).unwrap_or("localhost");
    let port = port
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(5733);
    (host, port)
}

pub fn web_channel_cookie(channel: &str) -> String {
    let prefix = format!("r3code_web_channel={channel}");
    std::iter::once(prefix.as_str())
        .chain(web_vercel_config_surface().channel_cookie_parts)
        .collect::<Vec<_>>()
        .join("; ")
}

fn trim_non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
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
        dependencies: BTreeMap::from([("effect", "catalog:")]),
        dev_dependencies: BTreeMap::from([
            ("@effect/language-service", "catalog:"),
            ("@effect/vitest", "catalog:"),
            ("tsdown", "catalog:"),
            ("typescript", "catalog:"),
            ("vitest", "catalog:"),
        ]),
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
        dependencies: BTreeMap::from([
            ("@t3tools/contracts", "workspace:*"),
            ("effect", "catalog:"),
        ]),
        dev_dependencies: BTreeMap::from([
            ("@effect/language-service", "catalog:"),
            ("@effect/platform-node", "catalog:"),
            ("@effect/vitest", "catalog:"),
            ("@types/node", "catalog:"),
            ("typescript", "catalog:"),
            ("vitest", "catalog:"),
        ]),
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

pub fn client_runtime_package_surface() -> UtilityPackageSurface {
    UtilityPackageSurface {
        metadata: PackageSurfaceMetadata {
            name: "@r3tools/client-runtime",
            upstream_name: "@t3tools/client-runtime",
            version: "0.0.0-alpha.1",
            private: true,
            module_type: "module",
            main: None,
            product_name: None,
            files: Vec::new(),
        },
        exports: BTreeMap::from([shared_export(".", "./src/index.ts")]),
        scripts: BTreeMap::from([("typecheck", "tsc --noEmit"), ("test", "vitest run")]),
        dependencies: BTreeMap::from([
            ("@t3tools/contracts", "workspace:*"),
            ("effect", "catalog:"),
        ]),
        dev_dependencies: BTreeMap::from([
            ("@effect/language-service", "catalog:"),
            ("typescript", "catalog:"),
            ("vitest", "catalog:"),
        ]),
        tsconfig_extends: "../../tsconfig.base.json",
        tsconfig_include: vec!["src"],
    }
}

pub fn ssh_package_surface() -> UtilityPackageSurface {
    UtilityPackageSurface {
        metadata: PackageSurfaceMetadata {
            name: "@r3tools/ssh",
            upstream_name: "@t3tools/ssh",
            version: "0.0.0-alpha.1",
            private: true,
            module_type: "module",
            main: None,
            product_name: None,
            files: Vec::new(),
        },
        exports: BTreeMap::from([
            shared_export("./auth", "./src/auth.ts"),
            shared_export("./command", "./src/command.ts"),
            shared_export("./config", "./src/config.ts"),
            shared_export("./errors", "./src/errors.ts"),
            shared_export("./tunnel", "./src/tunnel.ts"),
        ]),
        scripts: BTreeMap::from([("typecheck", "tsc --noEmit"), ("test", "vitest run")]),
        dependencies: BTreeMap::from([
            ("@t3tools/contracts", "workspace:*"),
            ("@t3tools/shared", "workspace:*"),
            ("effect", "catalog:"),
        ]),
        dev_dependencies: BTreeMap::from([
            ("@effect/language-service", "catalog:"),
            ("@effect/platform-node", "catalog:"),
            ("@effect/vitest", "catalog:"),
            ("@types/node", "catalog:"),
            ("typescript", "catalog:"),
            ("vitest", "catalog:"),
        ]),
        tsconfig_extends: "../../tsconfig.base.json",
        tsconfig_include: vec!["src"],
    }
}

pub fn tailscale_package_surface() -> UtilityPackageSurface {
    UtilityPackageSurface {
        metadata: PackageSurfaceMetadata {
            name: "@r3tools/tailscale",
            upstream_name: "@t3tools/tailscale",
            version: "0.0.0-alpha.1",
            private: true,
            module_type: "module",
            main: None,
            product_name: None,
            files: Vec::new(),
        },
        exports: BTreeMap::from([shared_export(".", "./src/index.ts")]),
        scripts: BTreeMap::from([("typecheck", "tsc --noEmit"), ("test", "vitest run")]),
        dependencies: BTreeMap::from([
            ("@effect/platform-node", "catalog:"),
            ("effect", "catalog:"),
        ]),
        dev_dependencies: BTreeMap::from([
            ("@effect/language-service", "catalog:"),
            ("@effect/vitest", "catalog:"),
            ("@types/node", "catalog:"),
            ("typescript", "catalog:"),
            ("vitest", "catalog:"),
        ]),
        tsconfig_extends: "../../tsconfig.base.json",
        tsconfig_include: vec!["src"],
    }
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
        assert_eq!(contracts.dependencies["effect"], "catalog:");
        assert_eq!(
            contracts.dev_dependencies["@effect/language-service"],
            "catalog:"
        );
        assert_eq!(contracts.dev_dependencies["@effect/vitest"], "catalog:");
        assert_eq!(contracts.dev_dependencies["tsdown"], "catalog:");
        assert_eq!(contracts.tsconfig_extends, "../../tsconfig.base.json");
        assert_eq!(contracts.tsconfig_include, vec!["src"]);
    }

    #[test]
    fn ports_web_package_build_and_deploy_surface() {
        let web = web_package_surface();
        assert_eq!(web.metadata.name, "@r3tools/web");
        assert_eq!(web.metadata.upstream_name, "@t3tools/web");
        assert_eq!(web.metadata.version, "0.0.23");
        assert_eq!(web.scripts["dev"], "vite");
        assert_eq!(
            web.scripts["test:browser"],
            "vitest run --config vitest.browser.config.ts"
        );
        assert_eq!(web.dependencies["react"], "19.2.6");
        assert_eq!(web.dependencies["@tanstack/react-router"], "^1.160.2");
        assert_eq!(web.dependencies["@t3tools/shared"], "workspace:*");
        assert_eq!(web.dev_dependencies["vite"], "^8.0.0");
        assert_eq!(web.dev_dependencies["@vitejs/plugin-react"], "^6.0.0");
        assert_eq!(web.dev_dependencies["babel-plugin-react-compiler"], "1.0.0");

        assert_eq!(web.tsconfig.extends, "../../tsconfig.base.json");
        assert_eq!(web.tsconfig.module, "Preserve");
        assert_eq!(web.tsconfig.module_resolution, "Bundler");
        assert_eq!(web.tsconfig.lib, vec!["ES2023", "DOM", "DOM.Iterable"]);
        assert_eq!(web.tsconfig.paths["~/*"], vec!["./src/*"]);
        assert_eq!(
            web.tsconfig.effect_diagnostic_severity["unsafeEffectTypeAssertion"],
            "error"
        );
        assert_eq!(
            web.tsconfig.include,
            vec!["src", "vite.config.ts", "vercel.ts", "test"]
        );

        assert_eq!(web.components.style, "base-mira");
        assert_eq!(web.components.tailwind_css, "src/index.css");
        assert_eq!(web.components.icon_library, "lucide");
        assert_eq!(web.components.aliases["ui"], "~/components/ui");
        assert_eq!(
            web.components.registries["@coss"],
            "https://coss.com/ui/r/{name}.json"
        );

        assert_eq!(
            web.index_html.viewport,
            "width=device-width, initial-scale=1.0, viewport-fit=cover, interactive-widget=resizes-content"
        );
        assert_eq!(web.index_html.theme_storage_key, "r3code:theme");
        assert_eq!(web.index_html.upstream_theme_storage_key, "t3code:theme");
        assert_eq!(web.index_html.title, "R3 Code (Alpha)");
        assert_eq!(web.index_html.upstream_title, "T3 Code (Alpha)");
        assert_eq!(web.index_html.boot_shell_card_size_px, 96);
        assert_eq!(web.index_html.boot_shell_logo_size_px, 64);
        assert_eq!(web.index_html.splash_aria_label, "R3 Code splash screen");
        assert_eq!(web.index_html.logo_alt, "R3 Code");

        assert_eq!(web.vite.default_port, 5733);
        assert_eq!(
            web.vite.plugins,
            vec![
                "tanstackRouter",
                "@vitejs/plugin-react",
                "@rolldown/plugin-babel",
                "@tailwindcss/vite"
            ]
        );
        assert_eq!(web.vite.babel_parser_plugins, vec!["typescript", "jsx"]);
        assert_eq!(
            web.vite.optimize_deps_include,
            vec![
                "@pierre/diffs",
                "@pierre/diffs/react",
                "@pierre/diffs/worker/worker.js",
                "effect/Array",
                "effect/Order"
            ]
        );
        assert_eq!(
            web.vite.proxy_paths,
            vec!["/.well-known", "/api", "/attachments"]
        );
        assert_eq!(web.vite.hmr_protocol, "ws");
        assert_eq!(web.vite.upstream_sourcemap_env, "T3CODE_WEB_SOURCEMAP");
        assert_eq!(web.vite.sourcemap_env, "R3CODE_WEB_SOURCEMAP");

        assert!(web.vitest_browser.merged_from_vite_config);
        assert_eq!(web.vitest_browser.src_alias, "~");
        assert_eq!(
            web.vitest_browser.include,
            vec!["src/components/**/*.browser.tsx"]
        );
        assert_eq!(web.vitest_browser.provider, "playwright");
        assert_eq!(web.vitest_browser.instances, vec!["chromium"]);
        assert_eq!(web.vitest_browser.test_timeout_ms, 30_000);

        assert_eq!(web.vercel.router_host, "app.r3.codes");
        assert_eq!(web.vercel.upstream_router_host, "app.t3.codes");
        assert_eq!(web.vercel.channel_route, "/__r3code/channel");
        assert_eq!(
            web.vercel.clean_channel_query_transform,
            ("request.query", "delete", "channel")
        );
        assert_eq!(web.vercel.channels, vec!["latest", "nightly"]);
        assert_eq!(web.vercel.app_rewrite_destination, "/index.html");
    }

    #[test]
    fn ports_web_vite_runtime_resolution_helpers() {
        assert_eq!(
            resolve_web_dev_proxy_target(Some("ws://localhost:3773/ws?token=1#frag")),
            Some("http://localhost:3773/".to_string())
        );
        assert_eq!(
            resolve_web_dev_proxy_target(Some("wss://example.test/api/socket")),
            Some("https://example.test/".to_string())
        );
        assert_eq!(resolve_web_dev_proxy_target(Some("not a url")), None);
        assert_eq!(resolve_web_dev_proxy_target(None), None);

        assert_eq!(
            resolve_web_build_sourcemap(None),
            WebBuildSourcemap::Enabled
        );
        assert_eq!(
            resolve_web_build_sourcemap(Some(" false ")),
            WebBuildSourcemap::Disabled
        );
        assert_eq!(
            resolve_web_build_sourcemap(Some("0")),
            WebBuildSourcemap::Disabled
        );
        assert_eq!(
            resolve_web_build_sourcemap(Some("hidden")),
            WebBuildSourcemap::Hidden
        );

        assert_eq!(
            resolve_web_hosted_app_url(
                Some(" https://explicit.example "),
                Some("production"),
                Some("prod.example"),
                Some("preview.example"),
            ),
            Some("https://explicit.example".to_string())
        );
        assert_eq!(
            resolve_web_hosted_app_url(None, Some("production"), Some("prod.example"), None),
            Some("https://prod.example".to_string())
        );
        assert_eq!(
            resolve_web_hosted_app_url(
                None,
                Some("preview"),
                Some("prod.example"),
                Some("vercel.example")
            ),
            Some("https://vercel.example".to_string())
        );
        assert_eq!(resolve_web_hosted_app_url(None, None, None, None), None);

        assert_eq!(
            resolve_web_host_and_port(Some(" 0.0.0.0 "), Some("5734")),
            ("0.0.0.0", 5734)
        );
        assert_eq!(
            resolve_web_host_and_port(Some(" "), Some("invalid")),
            ("localhost", 5733)
        );
        assert_eq!(
            web_channel_cookie("nightly"),
            "r3code_web_channel=nightly; Path=/; Max-Age=31536000; HttpOnly; Secure; SameSite=Lax"
        );
    }

    #[test]
    fn ports_web_css_vite_env_and_test_support_surfaces() {
        let css = web_index_css_surface();
        assert_eq!(css.imports, vec!["tailwindcss"]);
        assert_eq!(
            css.custom_variants,
            vec![
                ("dark", "&:is(.dark, .dark *)"),
                ("wco", "&:is(.wco, .wco *)")
            ]
        );
        assert!(css.theme_inline_vars.contains(&"--animate-skeleton"));
        assert!(css.theme_inline_vars.contains(&"--color-background"));
        assert_eq!(css.root_light_tokens["--radius"], "0.625rem");
        assert_eq!(css.root_light_tokens["--primary"], "oklch(0.488 0.217 264)");
        assert_eq!(css.root_dark_tokens["--primary"], "oklch(0.588 0.217 264)");
        assert_eq!(
            css.safe_area_utilities,
            vec![
                ("pt-safe", "padding-top"),
                ("pb-safe", "padding-bottom"),
                ("pl-safe", "padding-left"),
                ("pr-safe", "padding-right")
            ]
        );
        assert_eq!(css.body_font_stack[0], "DM Sans");
        assert_eq!(css.noise_overlay_opacity, "0.035");
        assert_eq!(css.scrollbar_width_px, 6);
        assert_eq!(css.terminal_scrollbar_width_px, 6);
        assert!(css.class_hooks.contains(&".chat-markdown"));
        assert!(css.class_hooks.contains(&".ultrathink-frame"));
        assert!(css.keyframes.contains(&"provider-update-pill-countdown"));

        let vite_env = web_vite_env_surface();
        assert_eq!(vite_env.reference_types, "vite/client");
        assert_eq!(
            vite_env.env_keys,
            vec![
                "VITE_HTTP_URL",
                "VITE_WS_URL",
                "VITE_HOSTED_APP_URL",
                "VITE_HOSTED_APP_CHANNEL",
                "APP_VERSION"
            ]
        );
        assert_eq!(vite_env.window_globals["nativeApi"], "LocalApi");
        assert_eq!(vite_env.window_globals["desktopBridge"], "DesktopBridge");

        let auth = web_auth_http_handlers_surface();
        assert_eq!(auth.test_session_expires_at, "2026-05-01T12:00:00.000Z");
        assert_eq!(auth.environment_id, "environment-local");
        assert_eq!(auth.platform_os, "darwin");
        assert_eq!(auth.platform_arch, "arm64");
        assert_eq!(auth.server_version, "0.0.0-test");
        assert!(auth.repository_identity_capability);
        assert_eq!(
            auth.routes,
            vec![
                ("GET", "*/.well-known/t3/environment"),
                ("GET", "*/api/auth/session"),
                ("POST", "*/api/auth/bootstrap")
            ]
        );
        assert_eq!(auth.session_method, "browser-session-cookie");

        let harness = web_ws_rpc_harness_surface();
        assert_eq!(harness.parser, "RpcSerialization.json.makeUnsafe");
        assert_eq!(harness.server_factory, "RpcServer.makeNoSerialization");
        assert_eq!(
            harness.stream_methods,
            vec![
                "orchestration.subscribeShell",
                "orchestration.subscribeThread",
                "gitRunStackedAction",
                "subscribeVcsStatus",
                "subscribeTerminalEvents",
                "subscribeServerConfig",
                "subscribeServerLifecycle"
            ]
        );
        assert!(harness.records_requests);
        assert!(harness.responds_to_ping_with_pong);
        assert!(harness.resets_scope_before_reconnect);
        assert_eq!(harness.stream_pubsub_kind, "PubSub.unbounded");
        assert_eq!(
            harness.missing_stream_error_prefix,
            "No stream registered for "
        );

        let msw = web_mock_service_worker_surface();
        assert_eq!(msw.package_version, "2.12.11");
        assert_eq!(msw.integrity_checksum, "4db4a41e972cec1b64cc569c66952d82");
        assert!(msw.install_skip_waiting);
        assert!(msw.activate_claims_clients);
        assert_eq!(
            msw.message_types,
            vec![
                "KEEPALIVE_REQUEST",
                "INTEGRITY_CHECK_REQUEST",
                "MOCK_ACTIVATE",
                "CLIENT_CLOSED"
            ]
        );
        assert!(msw.bypasses_navigation_requests);
        assert!(msw.bypasses_only_if_cached_cross_origin);
        assert!(msw.unregisters_when_last_client_closes);
        assert_eq!(msw.response_event_type, "RESPONSE");
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
        assert_eq!(shared.dependencies["@t3tools/contracts"], "workspace:*");
        assert_eq!(shared.dependencies["effect"], "catalog:");
        assert_eq!(shared.dev_dependencies["@effect/platform-node"], "catalog:");
        assert_eq!(shared.dev_dependencies["@types/node"], "catalog:");
        assert_eq!(shared.tsconfig_extends, "../../tsconfig.base.json");
        assert_eq!(shared.tsconfig_include, vec!["src"]);
    }

    #[test]
    fn ports_client_runtime_package_surface() {
        let client_runtime = client_runtime_package_surface();
        assert_eq!(client_runtime.metadata.name, "@r3tools/client-runtime");
        assert_eq!(
            client_runtime.metadata.upstream_name,
            "@t3tools/client-runtime"
        );
        assert_eq!(client_runtime.metadata.version, "0.0.0-alpha.1");
        assert!(client_runtime.metadata.private);
        assert_eq!(client_runtime.metadata.module_type, "module");
        assert_eq!(client_runtime.exports["."]["types"], "./src/index.ts");
        assert_eq!(client_runtime.exports["."]["import"], "./src/index.ts");
        assert_eq!(client_runtime.scripts["typecheck"], "tsc --noEmit");
        assert_eq!(client_runtime.scripts["test"], "vitest run");
        assert_eq!(
            client_runtime.dependencies["@t3tools/contracts"],
            "workspace:*"
        );
        assert_eq!(client_runtime.dependencies["effect"], "catalog:");
        assert_eq!(
            client_runtime.dev_dependencies["@effect/language-service"],
            "catalog:"
        );
        assert_eq!(client_runtime.dev_dependencies["typescript"], "catalog:");
        assert_eq!(client_runtime.dev_dependencies["vitest"], "catalog:");
        assert_eq!(client_runtime.tsconfig_extends, "../../tsconfig.base.json");
        assert_eq!(client_runtime.tsconfig_include, vec!["src"]);
    }

    #[test]
    fn ports_ssh_and_tailscale_package_surfaces() {
        let ssh = ssh_package_surface();
        assert_eq!(ssh.metadata.name, "@r3tools/ssh");
        assert_eq!(ssh.metadata.upstream_name, "@t3tools/ssh");
        assert_eq!(ssh.metadata.version, "0.0.0-alpha.1");
        assert!(ssh.metadata.private);
        assert_eq!(ssh.metadata.module_type, "module");
        assert_eq!(ssh.exports["./auth"]["types"], "./src/auth.ts");
        assert_eq!(ssh.exports["./command"]["import"], "./src/command.ts");
        assert_eq!(ssh.exports["./config"]["types"], "./src/config.ts");
        assert_eq!(ssh.exports["./errors"]["import"], "./src/errors.ts");
        assert_eq!(ssh.exports["./tunnel"]["types"], "./src/tunnel.ts");
        assert_eq!(ssh.scripts["typecheck"], "tsc --noEmit");
        assert_eq!(ssh.scripts["test"], "vitest run");
        assert_eq!(ssh.dependencies["@t3tools/contracts"], "workspace:*");
        assert_eq!(ssh.dependencies["@t3tools/shared"], "workspace:*");
        assert_eq!(ssh.dependencies["effect"], "catalog:");
        assert_eq!(ssh.dev_dependencies["@effect/platform-node"], "catalog:");
        assert_eq!(ssh.dev_dependencies["@types/node"], "catalog:");
        assert_eq!(ssh.tsconfig_extends, "../../tsconfig.base.json");
        assert_eq!(ssh.tsconfig_include, vec!["src"]);

        let tailscale = tailscale_package_surface();
        assert_eq!(tailscale.metadata.name, "@r3tools/tailscale");
        assert_eq!(tailscale.metadata.upstream_name, "@t3tools/tailscale");
        assert_eq!(tailscale.metadata.version, "0.0.0-alpha.1");
        assert_eq!(tailscale.exports["."]["types"], "./src/index.ts");
        assert_eq!(tailscale.scripts["typecheck"], "tsc --noEmit");
        assert_eq!(tailscale.dependencies["@effect/platform-node"], "catalog:");
        assert_eq!(tailscale.dependencies["effect"], "catalog:");
        assert_eq!(tailscale.dev_dependencies["@effect/vitest"], "catalog:");
        assert_eq!(tailscale.tsconfig_extends, "../../tsconfig.base.json");
        assert_eq!(tailscale.tsconfig_include, vec!["src"]);
    }
}
