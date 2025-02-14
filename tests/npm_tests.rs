mod common;
use common::assert_command;
use nirs::package_managers::npm::NpmFactory;
use nirs::PackageManagerFactory;

#[test]
fn test_npm_commands() {
    let factory: Box<dyn PackageManagerFactory> = Box::new(NpmFactory {});
    let executor = factory.create_commands();

    assert_command(
        &executor,
        "run",
        vec!["dev", "--port=3000"],
        "npm",
        vec!["run", "dev", "--port=3000"],
    );
    assert_command(
        &executor,
        "install",
        vec!["vite"],
        "npm",
        vec!["install", "vite"],
    );
    assert_command(
        &executor,
        "add",
        vec!["@types/node", "-D"],
        "npm",
        vec!["install", "@types/node", "-D"],
    );
    assert_command(&executor, "execute", vec!["vitest"], "npx", vec!["vitest"]);
    assert_command(&executor, "upgrade", vec![], "npm", vec!["update"]);
    assert_command(
        &executor,
        "uninstall",
        vec!["webpack"],
        "npm",
        vec!["uninstall", "webpack"],
    );
    assert_command(&executor, "clean_install", vec![], "npm", vec!["ci"]);
}

#[test]
fn test_npm_edge_cases() {
    let factory: Box<dyn PackageManagerFactory> = Box::new(NpmFactory {});
    let executor = factory.create_commands();

    // Test empty args
    assert_command(&executor, "install", vec![], "npm", vec!["install"]);

    // Test args with spaces
    assert_command(
        &executor,
        "run",
        vec!["test:unit", "--coverage"],
        "npm",
        vec!["run", "test:unit", "--coverage"],
    );

    // Test multiple flags
    assert_command(
        &executor,
        "add",
        vec!["@types/node", "-D", "--exact"],
        "npm",
        vec!["install", "@types/node", "-D", "--exact"],
    );

    // Test global with other flags
    assert_command(
        &executor,
        "uninstall",
        vec!["-g", "webpack", "--save"],
        "npm",
        vec!["uninstall", "-g", "webpack", "--save"],
    );
}
