mod common;
use common::assert_command;
use nirs::package_managers::bun::BunFactory;
use nirs::PackageManagerFactory;

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
