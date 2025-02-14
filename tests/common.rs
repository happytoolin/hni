use nirs::CommandExecutor;

pub fn assert_command(
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
