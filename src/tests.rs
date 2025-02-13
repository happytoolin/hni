#[cfg(test)]
mod tests {
    use crate::{
        package_managers::{
            bun::BunFactory, deno::DenoFactory, npm::NpmFactory, pnpm::PnpmFactory,
            pnpm6::Pnpm6Factory, yarn::YarnFactory, yarn_berry::YarnBerryFactory,
        },
        CommandExecutor, PackageManagerFactory,
    };

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
        assert_eq!(command.args, expected_args);
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
            vec!["run", "dev --port=3000"],
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
            vec!["add", "@types/node -D"],
        );
        assert_command(
            &executor,
            "execute",
            vec!["vitest"],
            "bun",
            vec!["x", "vitest"],
        );
        assert_command(&executor, "upgrade", vec![], "bun", vec!["update", ""]);
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

    #[test]
    fn test_deno_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(DenoFactory {});
        let executor = factory.create_commands();

        assert_command(
            &executor,
            "run",
            vec!["dev", "--port=3000"],
            "deno",
            vec!["task", "dev --port=3000"],
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
            vec!["add", "@types/node -D"],
        );
        assert_command(
            &executor,
            "execute",
            vec!["vitest"],
            "deno",
            vec!["run", "npm:vitest"],
        );
        assert_command(&executor, "upgrade", vec![], "deno", vec!["update", ""]);
        assert_command(
            &executor,
            "uninstall",
            vec!["webpack"],
            "deno",
            vec!["remove", "webpack"],
        );
        assert_command(
            &executor,
            "clean_install",
            vec![],
            "deno",
            vec!["install", ""],
        );
    }

    #[test]
    fn test_npm_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(NpmFactory {});
        let executor = factory.create_commands();

        assert_command(
            &executor,
            "run",
            vec!["dev", "--port=3000"],
            "npm",
            vec!["run", "dev --port=3000"],
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
            vec!["install", "@types/node -D"],
        );
        assert_command(&executor, "execute", vec!["vitest"], "npx", vec!["vitest"]);
        assert_command(&executor, "upgrade", vec![], "npm", vec!["update", ""]);
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
    fn test_pnpm_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(PnpmFactory {});
        let executor = factory.create_commands();

        assert_command(
            &executor,
            "run",
            vec!["dev", "--port=3000"],
            "pnpm",
            vec!["run", "dev --port=3000"],
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
            vec!["add", "@types/node -D"],
        );
        assert_command(
            &executor,
            "execute",
            vec!["vitest"],
            "pnpm",
            vec!["dlx", "vitest"],
        );
        assert_command(&executor, "upgrade", vec![], "pnpm", vec!["update", ""]);
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
    fn test_pnpm6_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(Pnpm6Factory {});
        let executor = factory.create_commands();

        assert_command(
            &executor,
            "run",
            vec!["dev", "--port=3000"],
            "pnpm",
            vec!["run", "dev --port=3000"],
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
            vec!["add", "@types/node -D"],
        );
        assert_command(
            &executor,
            "execute",
            vec!["vitest"],
            "pnpm",
            vec!["dlx", "vitest"],
        );
        assert_command(&executor, "upgrade", vec![], "pnpm", vec!["update", ""]);
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
    fn test_yarn_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(YarnFactory {});
        let executor = factory.create_commands();

        assert_command(
            &executor,
            "run",
            vec!["dev", "--port=3000"],
            "yarn",
            vec!["run", "dev --port=3000"],
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
            vec!["add", "@types/node -D"],
        );
        assert_command(
            &executor,
            "execute",
            vec!["vitest"],
            "yarn",
            vec!["exec", "vitest"],
        );
        assert_command(&executor, "upgrade", vec![], "yarn", vec!["upgrade", ""]);
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
            vec!["install", "--frozen-lockfile"],
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
            vec!["run", "dev --port=3000"],
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
            vec!["add", "@types/node -D"],
        );
        assert_command(
            &executor,
            "execute",
            vec!["vitest"],
            "yarn",
            vec!["exec", "vitest"],
        );
        assert_command(&executor, "upgrade", vec![], "yarn", vec!["up", ""]);
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
    fn test_interactive_upgrade() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(YarnFactory {});
        let executor = factory.create_commands();

        assert_command(
            &executor,
            "upgrade",
            vec!["-i"],
            "yarn",
            vec!["upgrade", "-i"],
        );

        let factory: Box<dyn PackageManagerFactory> = Box::new(YarnBerryFactory {});
        let executor = factory.create_commands();

        assert_command(&executor, "upgrade", vec!["-i"], "yarn", vec!["up", "-i"]);

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

    #[test]
    fn test_interactive_uninstall() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(NpmFactory {});
        let executor = factory.create_commands();

        // Test interactive single uninstall
        assert_command(
            &executor,
            "uninstall",
            vec!["webpack"],
            "npm",
            vec!["uninstall", "webpack"],
        );

        // Test interactive multiple uninstall
        assert_command(
            &executor,
            "uninstall",
            vec!["webpack", "babel", "typescript"],
            "npm",
            vec!["uninstall", "webpack babel typescript"],
        );
    }

    #[test]
    fn test_edge_cases() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(NpmFactory {});
        let executor = factory.create_commands();

        // Test empty args
        assert_command(&executor, "install", vec![], "npm", vec!["install", ""]);

        // Test args with spaces
        assert_command(
            &executor,
            "run",
            vec!["test:unit", "--coverage"],
            "npm",
            vec!["run", "test:unit --coverage"],
        );

        // Test multiple flags
        assert_command(
            &executor,
            "add",
            vec!["@types/node", "-D", "--exact"],
            "npm",
            vec!["install", "@types/node -D --exact"],
        );

        // Test global with other flags
        assert_command(
            &executor,
            "uninstall",
            vec!["-g", "webpack", "--save"],
            "npm",
            vec!["uninstall", "-g webpack --save"],
        );
    }
}
