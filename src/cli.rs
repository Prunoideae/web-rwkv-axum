use std::{
    fs::File,
    io::{BufReader, Read},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::PathBuf,
};

use crate::config::ModelConfig;
use anyhow::{Ok, Result};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct LaunchArgs {
    /// Worker count for the Tokio runtime
    #[arg(short, long, value_name = "THREAD", default_value_t = 8)]
    tokio_worker_count: usize,

    /// The path to configuration file
    #[arg(value_name = "PATH")]
    config: String,

    /// The IP Address to listen on
    #[arg(value_name = "IP", default_value_t = String::from("127.0.0.1"))]
    address: String,

    /// The port to listen on
    #[arg(default_value_t = 5678)]
    port: u16,
}

impl LaunchArgs {
    pub fn get_workers(&self) -> usize {
        self.tokio_worker_count.min(num_cpus::get())
    }

    pub fn get_addr_port(&self) -> Result<SocketAddr> {
        Ok(SocketAddr::V4(SocketAddrV4::new(
            self.address.parse()?,
            self.port,
        )))
    }

    pub fn get_config(&self) -> Result<ModelConfig> {
        let content = {
            let file = PathBuf::from(&self.config);
            let mut file = BufReader::with_capacity(1024 * 128, File::open(file)?);
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;
            buf
        };
        Ok(toml::from_str(content.as_str())?)
    }
}
