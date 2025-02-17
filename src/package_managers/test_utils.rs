use crate::{CommandExecutor, PackageManagerFactory};

pub fn test_basic_commands(
    factory: Box<dyn PackageManagerFactory>,
    expected_bin: &str,
    command_map: &[(&str, &str)],
) {
    let executor = factory.create_commands();
    let command_map: std::collections::HashMap<_, _> = command_map.iter().cloned().collect();

    // Test run command
    assert_command_with_bin(
        &*executor,
        "run",
        vec!["dev", "--port=3000"],
        expected_bin,
        command_map.get("run").unwrap_or(&"run"),
        vec!["dev", "--port=3000"],
    );

    // Test install command
    assert_command_with_bin(
        &*executor,
        "install",
        vec!["vite"],
        expected_bin,
        command_map.get("install").unwrap_or(&"install"),
        vec!["vite"],
    );

    // Test add command with dev dependency
    let (add_cmd, add_dev_flag) = match command_map.get("add_dev_flag") {
        Some(flag) => (command_map.get("add").unwrap_or(&"add"), *flag),
        None => (command_map.get("add").unwrap_or(&"add"), "--save-dev"),
    };
    assert_command_with_bin(
        &*executor,
        "add",
        vec!["@types/node", "-D"],
        expected_bin,
        add_cmd,
        vec!["@types/node", add_dev_flag],
    );

    // Test clean install
    let clean_install_args = command_map
        .get("clean_install_args")
        .map(|&args| {
            if args.is_empty() {
                vec![]
            } else {
                args.split(' ').collect::<Vec<_>>()
            }
        })
        .unwrap_or_else(|| vec!["--frozen-lockfile"]);

    assert_command_with_bin(
        &*executor,
        "clean_install",
        vec![],
        expected_bin,
        command_map.get("clean_install").unwrap_or(&"install"),
        clean_install_args,
    );
}

pub fn test_edge_cases(
    factory: Box<dyn PackageManagerFactory>,
    expected_bin: &str,
    command_map: &[(&str, &str)],
) {
    let executor = factory.create_commands();
    let command_map: std::collections::HashMap<_, _> = command_map.iter().cloned().collect();

    // Test empty args
    assert_command_with_bin(
        &*executor,
        "install",
        vec![],
        expected_bin,
        command_map.get("install").unwrap_or(&"install"),
        vec![],
    );

    // Test args with spaces
    assert_command_with_bin(
        &*executor,
        "run",
        vec!["test:unit", "--coverage"],
        expected_bin,
        command_map.get("run").unwrap_or(&"run"),
        vec!["test:unit", "--coverage"],
    );

    // Test multiple flags
    let (add_cmd, add_dev_flag) = match command_map.get("add_dev_flag") {
        Some(flag) => (command_map.get("add").unwrap_or(&"add"), *flag),
        None => (command_map.get("add").unwrap_or(&"add"), "--save-dev"),
    };
    assert_command_with_bin(
        &*executor,
        "add",
        vec!["@types/node", "-D", "--exact"],
        expected_bin,
        add_cmd,
        vec!["@types/node", add_dev_flag, "--exact"],
    );

    // Handle global package installation
    if let Some(global_cmd) = command_map.get("global_install") {
        let parts: Vec<&str> = global_cmd.split(' ').collect();
        assert_command_with_bin(
            &*executor,
            "install",
            vec!["-g", "webpack"],
            expected_bin,
            parts.first().unwrap(),
            parts[1..].iter().chain(&["webpack"]).cloned().collect(),
        );
    }
}

fn assert_command_with_bin(
    executor: &dyn CommandExecutor,
    action: &str,
    args: Vec<&str>,
    expected_bin: &str,
    expected_command: &str,
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

    let mut expected = if expected_command.is_empty() {
        vec![]
    } else {
        vec![expected_command.to_string()]
    };
    expected.extend(expected_args.iter().map(|s| s.to_string()));
    assert_eq!(command.args, expected);
}
