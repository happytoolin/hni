use nirs::package_managers::yarn_berry::YarnBerryFactory;
use nirs::CommandExecutor;
use nirs::PackageManagerFactory;

#[allow(clippy::borrowed_box)]
fn assert_command(
    executor: &Box<dyn CommandExecutor>,
    action: &str,
    args: Vec<&str>,
    expected_bin: &str,
    expected_args: Vec<&str>,
) {
    let command = match action {
        "run" => executor.run(args),
        "install" => executor.install(args),
        "add" => executor.add(args),
        "execute" => executor.execute(args),
        "upgrade" => executor.upgrade(args),
        "uninstall" => executor.uninstall(args),
        "clean_install" => executor.clean_install(args),
        _ => panic!("Unknown action: {}", action),
    }
    .expect("Command should be resolved");

    assert_eq!(command.bin, expected_bin);
    assert_eq!(
        command.args,
        expected_args
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_yarn_berry_commands() {
    let factory: Box<dyn PackageManagerFactory> = Box::new(YarnBerryFactory {});
    let executor = factory.create_commands();

    assert_command(
        &executor,
        "run",
        vec!["dev", "--port=3000"],
        "yarn",
        vec!["run", "dev", "--port=3000"],
    );
    assert_command(
        &executor,
        "install",
        vec!["vite"],
        "yarn",
        vec!["install", "vite"],
    );
    assert_command(
        &executor,
        "add",
        vec!["@types/node", "-D"],
        "yarn",
        vec!["add", "@types/node", "-D"],
    );
    assert_command(
        &executor,
        "execute",
        vec!["vitest"],
        "yarn",
        vec!["exec", "vitest"],
    );
    assert_command(&executor, "upgrade", vec![], "yarn", vec!["up"]);
    assert_command(
        &executor,
        "uninstall",
        vec!["webpack"],
        "yarn",
        vec!["remove", "webpack"],
    );
    assert_command(
        &executor,
        "clean_install",
        vec![],
        "yarn",
        vec!["install", "--immutable"],
    );
}

#[test]
fn test_yarn_berry_interactive_upgrade() {
    let factory: Box<dyn PackageManagerFactory> = Box::new(YarnBerryFactory {});
    let executor = factory.create_commands();

    assert_command(&executor, "upgrade", vec!["-i"], "yarn", vec!["up", "-i"]);
}
