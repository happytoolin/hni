use crate::core::types::InvocationKind;

pub fn print_help(invocation: InvocationKind) {
    match invocation {
        InvocationKind::NodeShim => {
            println!("hni node shim");
            println!("Routes npm-like verbs through detected package manager.");
            println!("Passthrough: node <file.js>, node -v, node -- <args>");
            println!("Routed verbs: p, s, install|i, add, run, exec|x|dlx, update|upgrade, uninstall|remove, ci");
        }
        _ => {
            println!("hni - use the right package manager");
            println!();
            println!("Commands:");
            println!("  ni      install/add packages");
            println!("  nr      run scripts");
            println!("  nlx     execute package binaries");
            println!("  nu      upgrade dependencies");
            println!("  nun     uninstall dependencies");
            println!("  nci     clean install");
            println!("  na      agent alias");
            println!("  np      run shell commands in parallel");
            println!("  ns      run shell commands sequentially");
            println!("  node    package-manager-aware node shim");
            println!();
            println!("Global flags:");
            println!("  ?               print resolved command");
            println!("  -C <dir>        run as if in <dir>");
            println!("  -v, --version   show versions");
            println!("  -h, --help      show help");
        }
    }
}
