use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvocationKind {
    Hni,
    Ni,
    Nr,
    Nlx,
    Nu,
    Nun,
    Nci,
    Na,
    Np,
    Ns,
    NodeShim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    Install,
    Add,
    Run,
    Execute,
    Upgrade,
    Uninstall,
    CleanInstall,
    AgentAlias,
    PassthroughNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedExecution {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub passthrough: bool,
    pub mode: ExecutionMode,
    pub strategy: ExecutionStrategy,
    pub native_requested: bool,
    pub native_fallback_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    PackageManager,
    NodeRun,
    Native,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStrategy {
    External,
    Native(NativeExecution),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeExecution {
    RunScript(NativeScriptExecution),
    RunLocalBin(NativeLocalBinExecution),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeScriptExecution {
    pub package_root: PathBuf,
    pub package_json_path: PathBuf,
    pub script_name: String,
    pub steps: Vec<NativeScriptStep>,
    pub forwarded_args: Vec<String>,
    pub bin_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeScriptStep {
    pub event_name: String,
    pub command: String,
    pub forward_args: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeLocalBinExecution {
    pub bin_name: String,
    pub bin_path: PathBuf,
    pub bin_paths: Vec<PathBuf>,
}

impl ResolvedExecution {
    pub fn external(
        program: impl Into<String>,
        args: Vec<String>,
        cwd: PathBuf,
        passthrough: bool,
    ) -> Self {
        Self::external_with_mode(
            program,
            args,
            cwd,
            passthrough,
            ExecutionMode::PackageManager,
        )
    }

    pub fn external_with_mode(
        program: impl Into<String>,
        args: Vec<String>,
        cwd: PathBuf,
        passthrough: bool,
        mode: ExecutionMode,
    ) -> Self {
        Self {
            program: program.into(),
            args,
            cwd,
            passthrough,
            mode,
            strategy: ExecutionStrategy::External,
            native_requested: false,
            native_fallback_reason: None,
        }
    }

    pub fn external_with_native_fallback(
        program: impl Into<String>,
        args: Vec<String>,
        cwd: PathBuf,
        passthrough: bool,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            program: program.into(),
            args,
            cwd,
            passthrough,
            mode: ExecutionMode::PackageManager,
            strategy: ExecutionStrategy::External,
            native_requested: true,
            native_fallback_reason: Some(reason.into()),
        }
    }

    pub fn native_script(
        script_name: impl Into<String>,
        cwd: PathBuf,
        exec: NativeScriptExecution,
    ) -> Self {
        let script_name = script_name.into();
        Self {
            program: script_name.clone(),
            args: exec.forwarded_args.clone(),
            cwd,
            passthrough: false,
            mode: ExecutionMode::Native,
            strategy: ExecutionStrategy::Native(NativeExecution::RunScript(exec)),
            native_requested: true,
            native_fallback_reason: None,
        }
    }

    pub fn native_local_bin(
        bin_name: impl Into<String>,
        args: Vec<String>,
        cwd: PathBuf,
        exec: NativeLocalBinExecution,
    ) -> Self {
        Self {
            program: bin_name.into(),
            args,
            cwd,
            passthrough: false,
            mode: ExecutionMode::Native,
            strategy: ExecutionStrategy::Native(NativeExecution::RunLocalBin(exec)),
            native_requested: true,
            native_fallback_reason: None,
        }
    }

    pub fn is_native(&self) -> bool {
        matches!(self.strategy, ExecutionStrategy::Native(_))
    }

    pub fn execution_mode_name(&self) -> &'static str {
        match self.mode {
            ExecutionMode::PackageManager => "package-manager",
            ExecutionMode::NodeRun => "node-run",
            ExecutionMode::Native => "native",
            ExecutionMode::Internal => "internal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeShimMode {
    RouteToIntent(Intent),
    RunParallel,
    RunSequential,
    PassthroughNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeShimDecision {
    pub mode: NodeShimMode,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Npm,
    Yarn,
    YarnBerry,
    Pnpm,
    Bun,
    Deno,
}

impl PackageManager {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Yarn => "yarn",
            Self::YarnBerry => "yarn (berry)",
            Self::Pnpm => "pnpm",
            Self::Bun => "bun",
            Self::Deno => "deno",
        }
    }

    pub fn bin(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Yarn | Self::YarnBerry => "yarn",
            Self::Pnpm => "pnpm",
            Self::Bun => "bun",
            Self::Deno => "deno",
        }
    }

    pub fn global_package_name(self) -> &'static str {
        self.bin()
    }

    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "npm" => Some(Self::Npm),
            "yarn" => Some(Self::Yarn),
            "yarn-berry" | "yarnberry" => Some(Self::YarnBerry),
            "pnpm" => Some(Self::Pnpm),
            "bun" => Some(Self::Bun),
            "deno" => Some(Self::Deno),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionSource {
    PackageManagerField,
    Lockfile,
    Config,
    Fallback,
    None,
}

#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub agent: Option<PackageManager>,
    pub has_lock: bool,
    pub version_hint: Option<String>,
    pub source: DetectionSource,
}
