use tracing_subscriber::EnvFilter;

fn main() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("greek_core=info,tower_http=warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();

    match greek_core::server::spawn_server() {
        Ok(info) => {
            println!("Greek core listo.");
            println!("  URL local: {}", info.url);
            println!("  Bootstrap: {}/api/dev/bootstrap", info.url);
            println!("  Red local: {}", info.lan_url);
            println!("  Bind: {}", info.bind);
            println!("Pulsa Ctrl C para detenerlo.");
            loop {
                std::thread::park();
            }
        }
        Err(error) => {
            eprintln!("No se pudo iniciar Greek: {error:#}");
            std::process::exit(1);
        }
    }
}
