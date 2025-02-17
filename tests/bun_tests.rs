use nirs::package_managers::bun::BunFactory;
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
fn test_bun_commands() {
    let factory: Box<dyn PackageManagerFactory> = Box::new(BunFactory {});
    let executor = factory.create_commands();

    assert_command(
        &executor,
        "run",
        vec!["dev", "--port=3000"],
        "bun",
        vec!["run", "dev", "--port=3000"],
    );
    assert_command(
        &executor,
        "install",
        vec!["vite"],
        "bun",
        vec!["install", "vite"],
    );
    assert_command(
        &executor,
        "add",
        vec!["@types/node", "-D"],
        "bun",
        vec!["add", "@types/node", "-D"],
    );
    assert_command(
        &executor,
        "execute",
        vec!["vitest"],
        "bun",
        vec!["x", "vitest"],
    );
    assert_command(&executor, "upgrade", vec![], "bun", vec!["update"]);
    assert_command(
        &executor,
        "uninstall",
        vec!["webpack"],
        "bun",
        vec!["remove", "webpack"],
    );
    assert_command(
        &executor,
        "clean_install",
        vec![],
        "bun",
        vec!["install", "--frozen-lockfile"],
    );
}
