use clap::*;

use crate::prelude::NetworkingPlugin;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct NetworkingArgs {
    #[arg(short, long)]
    username: String,
    #[arg(short, long)]
    server_addr: Option<String>,
}

const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:5000";

impl NetworkingPlugin {
    pub fn from_args(args: NetworkingArgs) -> Result<Self, String> {
        let server_addr = args.server_addr.unwrap_or_else(|| String::from(DEFAULT_SERVER_ADDR));

        NetworkingPlugin::new(&server_addr, args.username)
    }
}
