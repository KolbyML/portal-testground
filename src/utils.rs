use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use std::fs;
use std::net::IpAddr;
use std::str::FromStr;
use tokio::process::Command;
use wait_for_them::{wait_for_them, ToCheck};

const CLIENTS_DIRECTOR: &str = "./src/clients/";
const IP_ADDR: &str = "IP_ADDR";

pub fn get_client_names() -> Vec<String> {
    let paths = fs::read_dir(CLIENTS_DIRECTOR).unwrap();
    let directories: Vec<String> = paths
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_dir() {
                path.file_name()?.to_str().map(|s| s.to_owned())
            } else {
                None
            }
        })
        .collect();
    directories
}

/// Represents a running client.
#[derive(Debug, Clone)]
pub struct Client {
    pub kind: String,
    pub container: String,
    pub ip: IpAddr,
    pub rpc: HttpClient,
}

impl Client {
    pub async fn start_client(client_type: String, ip: String) -> Client {
        Command::new(
            CLIENTS_DIRECTOR.to_string()
                + &client_type.clone()
                + "/"
                + &client_type.clone()
                + ".sh",
        )
        .env(IP_ADDR, ip)
        .spawn()
        .expect("Expect client to run successfully");

        let ip = IpAddr::from_str("0.0.0.0").expect("Failed to decode IpAddr from string");

        let rpc_url = format!("http://{}:8545", ip);

        wait_for_them(
            &[ToCheck::HostnameAndPort("0.0.0.0".into(), 8545)],
            Some(8000),
            None,
            true,
        )
        .await;

        let rpc_client = HttpClientBuilder::default()
            .build(rpc_url)
            .expect("Failed to build rpc_client");

        Self {
            kind: client_type,
            container: "Local not container".to_string(),
            ip,
            rpc: rpc_client,
        }
    }
}
