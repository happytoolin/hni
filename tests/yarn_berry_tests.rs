mod common;
use common::assert_command;
use nirs::package_managers::yarn_berry::YarnBerryFactory;
use nirs::PackageManagerFactory;

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
