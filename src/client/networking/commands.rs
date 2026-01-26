use clap::*;

use crate::prelude::NetworkingPlugin;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct NetworkingArgs {
    #[arg(short, long)]
    username: String,
}

impl NetworkingPlugin {
    pub fn from_args(args: NetworkingArgs) -> Result<Self, String> {
        NetworkingPlugin::new(args.username)
    }
}
