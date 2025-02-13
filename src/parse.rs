use crate::{
    detect::PackageManagerFactoryEnum, ResolvedCommand,
    CommandExecutor,
};

pub struct PackageManager {
    pub name: PackageManagerFactoryEnum,
    pub executor: Box<dyn CommandExecutor>,
}

impl PackageManagerFactoryEnum {
    pub fn create_package_manager(&self) -> PackageManager {
        let executor = self.get_factory().create_commands();
        PackageManager {
            name: *self,
            executor,
        }
    }
}

pub fn parse_ni(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    let package_manager = agent.create_package_manager();
    package_manager.executor.add(args.to_vec().iter().map(|s| s.as_str()).collect())
        .expect("Failed to execute add command")
}

pub fn parse_nu(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    let package_manager = agent.create_package_manager();
    package_manager.executor.upgrade(args.to_vec().iter().map(|s| s.as_str()).collect())
        .expect("Failed to execute upgrade command")
}

pub fn parse_nun(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    let package_manager = agent.create_package_manager();
    package_manager.executor.uninstall(args.to_vec().iter().map(|s| s.as_str()).collect())
        .expect("Failed to execute uninstall command")
}

pub fn parse_nci(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    let package_manager = agent.create_package_manager();
    package_manager.executor.clean_install(args.to_vec().iter().map(|s| s.as_str()).collect())
        .expect("Failed to execute clean_install command")
}

pub fn parse_na(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    let package_manager = agent.create_package_manager();
    package_manager.executor.run(args.to_vec().iter().map(|s| s.as_str()).collect())
        .expect("Failed to execute run command")
}

pub fn parse_nlx(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    let package_manager = agent.create_package_manager();
    package_manager.executor.execute(args.to_vec().iter().map(|s| s.as_str()).collect())
        .expect("Failed to execute execute command")
}
