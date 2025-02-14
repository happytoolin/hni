mod common;
use common::assert_command;
use nirs::package_managers::pnpm::PnpmFactory;
use nirs::PackageManagerFactory;

#[test]
fn test_pnpm_commands() {
    let factory: Box<dyn PackageManagerFactory> = Box::new(PnpmFactory {});
    let executor = factory.create_commands();

    assert_command(
        &executor,
        "run",
        vec!["dev", "--port=3000"],
        "pnpm",
        vec!["run", "dev", "--port=3000"],
    );
    assert_command(
        &executor,
        "install",
        vec!["vite"],
        "pnpm",
        vec!["install", "vite"],
    );
    assert_command(
        &executor,
        "add",
        vec!["@types/node", "-D"],
        "pnpm",
        vec!["add", "@types/node", "-D"],
    );
    assert_command(
        &executor,
        "execute",
        vec!["vitest"],
        "pnpm",
        vec!["dlx", "vitest"],
    );
    assert_command(&executor, "upgrade", vec![], "pnpm", vec!["update"]);
    assert_command(
        &executor,
        "uninstall",
        vec!["webpack"],
        "pnpm",
        vec!["remove", "webpack"],
    );
    assert_command(
        &executor,
        "clean_install",
        vec![],
        "pnpm",
        vec!["install", "--frozen-lockfile"],
    );
}

#[test]
fn test_pnpm_interactive_upgrade() {
    let factory: Box<dyn PackageManagerFactory> = Box::new(PnpmFactory {});
    let executor = factory.create_commands();

    assert_command(
        &executor,
        "upgrade",
        vec!["-i"],
        "pnpm",
        vec!["update", "-i"],
    );
}
