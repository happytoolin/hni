mod common;
use common::assert_command;
use nirs::package_managers::deno::DenoFactory;
use nirs::PackageManagerFactory;

#[test]
fn test_deno_commands() {
    let factory: Box<dyn PackageManagerFactory> = Box::new(DenoFactory {});
    let executor = factory.create_commands();

    assert_command(
        &executor,
        "run",
        vec!["dev", "--port=3000"],
        "deno",
        vec!["task", "dev", "--port=3000"],
    );
    assert_command(
        &executor,
        "install",
        vec!["vite"],
        "deno",
        vec!["install", "vite"],
    );
    assert_command(
        &executor,
        "add",
        vec!["@types/node", "-D"],
        "deno",
        vec!["add", "@types/node", "-D"],
    );
    assert_command(
        &executor,
        "execute",
        vec!["vitest"],
        "deno",
        vec!["run", "npm:vitest"],
    );
    assert_command(&executor, "upgrade", vec![], "deno", vec!["update"]);
    assert_command(
        &executor,
        "uninstall",
        vec!["webpack"],
        "deno",
        vec!["remove", "webpack"],
    );
    assert_command(&executor, "clean_install", vec![], "deno", vec!["install"]);
}
