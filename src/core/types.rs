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
