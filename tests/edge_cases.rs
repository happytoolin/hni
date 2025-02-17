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

use nirs::package_managers::{
    bun::BunFactory, deno::DenoFactory, npm::NpmFactory, pnpm::PnpmFactory, pnpm6::Pnpm6Factory,
    yarn::YarnFactory, yarn_berry::YarnBerryFactory,
};

struct PackageManagerTest {
    factory: Box<dyn PackageManagerFactory>,
    bin: &'static str,
    install_cmd: &'static str,
    add_cmd: &'static str,
    exec_cmd: &'static str,
    upgrade_cmd: &'static str,
    uninstall_cmd: &'static str,
    run_cmd: &'static str,
}

fn get_package_managers() -> Vec<PackageManagerTest> {
    vec![
        PackageManagerTest {
            factory: Box::new(NpmFactory {}),
            bin: "npm",
            install_cmd: "install",
            add_cmd: "install",
            exec_cmd: "npx",
            upgrade_cmd: "update",
            uninstall_cmd: "uninstall",
            run_cmd: "run",
        },
        PackageManagerTest {
            factory: Box::new(YarnFactory {}),
            bin: "yarn",
            install_cmd: "install",
            add_cmd: "add",
            exec_cmd: "exec",
            upgrade_cmd: "upgrade",
            uninstall_cmd: "remove",
            run_cmd: "run",
        },
        PackageManagerTest {
            factory: Box::new(YarnBerryFactory {}),
            bin: "yarn",
            install_cmd: "install",
            add_cmd: "add",
            exec_cmd: "exec",
            upgrade_cmd: "up",
            uninstall_cmd: "remove",
            run_cmd: "run",
        },
        PackageManagerTest {
            factory: Box::new(PnpmFactory {}),
            bin: "pnpm",
            install_cmd: "install",
            add_cmd: "add",
            exec_cmd: "dlx",
            upgrade_cmd: "update",
            uninstall_cmd: "remove",
            run_cmd: "run",
        },
        PackageManagerTest {
            factory: Box::new(Pnpm6Factory {}),
            bin: "pnpm",
            install_cmd: "install",
            add_cmd: "add",
            exec_cmd: "dlx",
            upgrade_cmd: "update",
            uninstall_cmd: "remove",
            run_cmd: "run",
        },
        PackageManagerTest {
            factory: Box::new(BunFactory {}),
            bin: "bun",
            install_cmd: "install",
            add_cmd: "add",
            exec_cmd: "x",
            upgrade_cmd: "update",
            uninstall_cmd: "remove",
            run_cmd: "run",
        },
        PackageManagerTest {
            factory: Box::new(DenoFactory {}),
            bin: "deno",
            install_cmd: "install",
            add_cmd: "add",
            exec_cmd: "run",
            upgrade_cmd: "update",
            uninstall_cmd: "remove",
            run_cmd: "task",
        },
    ]
}

#[test]
fn test_empty_args() {
    for pm in get_package_managers() {
        let executor = pm.factory.create_commands();

        // Test empty install
        assert_command(&executor, "install", vec![], pm.bin, vec![pm.install_cmd]);

        // Test empty run (should default to start script)
        assert_command(&executor, "run", vec![], pm.bin, vec![pm.run_cmd]);

        // Test empty upgrade (should update all packages)
        assert_command(&executor, "upgrade", vec![], pm.bin, vec![pm.upgrade_cmd]);
    }
}

#[test]
fn test_special_characters() {
    for pm in get_package_managers() {
        let executor = pm.factory.create_commands();

        // Test package names with special characters
        assert_command(
            &executor,
            "add",
            vec!["@scope/package-name@1.0.0"],
            pm.bin,
            vec![pm.add_cmd, "@scope/package-name@1.0.0"],
        );

        // Test script names with colons
        assert_command(
            &executor,
            "run",
            vec!["test:unit", "--coverage"],
            pm.bin,
            vec![pm.run_cmd, "test:unit", "--coverage"],
        );

        // Test paths with spaces
        assert_command(
            &executor,
            "run",
            vec!["build", "--config", "./config with spaces.js"],
            pm.bin,
            vec![pm.run_cmd, "build", "--config", "./config with spaces.js"],
        );
    }
}

#[test]
fn test_multiple_commands() {
    for pm in get_package_managers() {
        let executor = pm.factory.create_commands();

        // Test multiple packages
        assert_command(
            &executor,
            "add",
            vec!["react", "react-dom", "@types/react", "-D"],
            pm.bin,
            vec![pm.add_cmd, "react", "react-dom", "@types/react", "-D"],
        );

        // Test multiple flags
        assert_command(
            &executor,
            "install",
            vec!["-g", "--production", "--no-optional"],
            pm.bin,
            vec![pm.install_cmd, "-g", "--production", "--no-optional"],
        );

        // Test complex script commands
        assert_command(
            &executor,
            "run",
            vec!["build", "--mode", "production", "--base", "/app/"],
            pm.bin,
            vec![
                pm.run_cmd,
                "build",
                "--mode",
                "production",
                "--base",
                "/app/",
            ],
        );
    }
}

#[test]
fn test_flag_variations() {
    for pm in get_package_managers() {
        let executor = pm.factory.create_commands();

        // Test short flags
        assert_command(
            &executor,
            "add",
            vec!["-D", "-E", "-O"],
            pm.bin,
            vec![pm.add_cmd, "-D", "-E", "-O"],
        );

        // Test long flags
        assert_command(
            &executor,
            "install",
            vec!["--save-dev", "--exact", "--save-optional"],
            pm.bin,
            vec![pm.install_cmd, "--save-dev", "--exact", "--save-optional"],
        );

        // Test mixed flags
        assert_command(
            &executor,
            "uninstall",
            vec!["-g", "webpack", "--save", "-D"],
            pm.bin,
            vec![pm.uninstall_cmd, "-g", "webpack", "--save", "-D"],
        );
    }
}

#[test]
fn test_version_specifiers() {
    for pm in get_package_managers() {
        let executor = pm.factory.create_commands();

        // Test exact version
        assert_command(
            &executor,
            "add",
            vec!["react@18.2.0"],
            pm.bin,
            vec![pm.add_cmd, "react@18.2.0"],
        );

        // Test version range
        assert_command(
            &executor,
            "add",
            vec!["typescript@>=4.0.0"],
            pm.bin,
            vec![pm.add_cmd, "typescript@>=4.0.0"],
        );

        // Test tag
        assert_command(
            &executor,
            "add",
            vec!["next@latest"],
            pm.bin,
            vec![pm.add_cmd, "next@latest"],
        );

        // Test git url
        assert_command(
            &executor,
            "add",
            vec!["user/repo#master"],
            pm.bin,
            vec![pm.add_cmd, "user/repo#master"],
        );
    }
}

#[test]
fn test_workspace_commands() {
    for pm in get_package_managers() {
        let executor = pm.factory.create_commands();

        // Test workspace root command
        assert_command(
            &executor,
            "run",
            vec!["build", "--workspace"],
            pm.bin,
            vec![pm.run_cmd, "build", "--workspace"],
        );

        // Test specific workspace
        assert_command(
            &executor,
            "add",
            vec!["lodash", "-W", "--filter", "package-a"],
            pm.bin,
            vec![pm.add_cmd, "lodash", "-W", "--filter", "package-a"],
        );

        // Test multiple workspaces
        assert_command(
            &executor,
            "run",
            vec!["test", "--filter", "./packages/*"],
            pm.bin,
            vec![pm.run_cmd, "test", "--filter", "./packages/*"],
        );
    }
}

#[test]
fn test_execute_commands() {
    for pm in get_package_managers() {
        let executor = pm.factory.create_commands();

        // Test basic execute
        assert_command(
            &executor,
            "execute",
            vec!["vitest"],
            if pm.bin == "npm" { "npx" } else { pm.bin },
            match pm.bin {
                "npm" => vec!["vitest"],
                "deno" => vec!["run", "npm:vitest"],
                _ => vec![pm.exec_cmd, "vitest"],
            },
        );

        // Test execute with arguments
        assert_command(
            &executor,
            "execute",
            vec!["prettier", "--write", "src/**/*.ts"],
            if pm.bin == "npm" { "npx" } else { pm.bin },
            match pm.bin {
                "npm" => vec!["prettier", "--write", "src/**/*.ts"],
                "deno" => vec!["run", "npm:prettier", "--write", "src/**/*.ts"],
                _ => vec![pm.exec_cmd, "prettier", "--write", "src/**/*.ts"],
            },
        );

        // Test execute with package version
        assert_command(
            &executor,
            "execute",
            vec!["tsc@4.9.5", "--init"],
            if pm.bin == "npm" { "npx" } else { pm.bin },
            match pm.bin {
                "npm" => vec!["tsc@4.9.5", "--init"],
                "deno" => vec!["run", "npm:tsc@4.9.5", "--init"],
                _ => vec![pm.exec_cmd, "tsc@4.9.5", "--init"],
            },
        );

        // Test execute with scoped package
        assert_command(
            &executor,
            "execute",
            vec!["@types/node", "--version"],
            if pm.bin == "npm" { "npx" } else { pm.bin },
            match pm.bin {
                "npm" => vec!["@types/node", "--version"],
                "deno" => vec!["run", "npm:@types/node", "--version"],
                _ => vec![pm.exec_cmd, "@types/node", "--version"],
            },
        );
    }
}
