use cgf::{cli::ChessGameFinderCLI, error::ChessError};

fn main() -> Result<(), ChessError> {
    openssl_probe::init_ssl_cert_env_vars();
    env_logger::init();
    let cli = ChessGameFinderCLI::new();
    cli.run()
}
