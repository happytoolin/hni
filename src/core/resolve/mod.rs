mod build;
mod context;
mod detect;
mod flags;
mod map;

pub use build::{
    resolve_na, resolve_nci, resolve_ni, resolve_nlx, resolve_node_passthrough,
    resolve_node_routed, resolve_nr, resolve_nu, resolve_nun,
};
pub use context::ResolveContext;
pub use detect::detected_package_manager;
pub use flags::exclude_flag;
pub use map::version_command_for_pm;
