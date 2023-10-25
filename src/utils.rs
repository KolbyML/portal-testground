#![allow(unused)]
use bollard::container::Config;
use bollard::image::{BuildImageOptions, BuilderVersion};
use bollard::Docker;
use flate2::write::GzEncoder;
use flate2::Compression;
use futures_util::stream::StreamExt;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use rand::random;
use std::fs;
use std::io::Write;
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use tar::Builder;
use tokio::process::Command;
use wait_for_them::{wait_for_them, ToCheck};

const CLIENTS_DIRECTOR: &str = "./src/clients/";
const CLIENT_TYPE: &str = "client_type";
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

pub fn get_dockerfile_tar(docker_file_path: PathBuf) -> Vec<u8> {
    let mut tar = Builder::new(Vec::new());
    tar.append_dir_all(".", docker_file_path).unwrap();
    let uncompressed = tar.into_inner().unwrap();
    let mut c = GzEncoder::new(Vec::new(), Compression::default());
    c.write_all(&uncompressed).unwrap();
    c.finish().unwrap()
}

pub async fn build_docker_image_for_client(client_type: &String) -> String {
    let mut docker_file_path = PathBuf::from(CLIENTS_DIRECTOR);
    docker_file_path.push(client_type);

    let id = random::<u64>();
    let options = BuildImageOptions {
        dockerfile: "Dockerfile",
        version: BuilderVersion::BuilderBuildKit,
        t: &format!("portal-testground/{}", id),
        session: Some(format!("portal-testground/{}", id)),
        rm: true,
        ..Default::default()
    };

    let docker_tar = get_dockerfile_tar(docker_file_path);
    let docker = Docker::connect_with_socket_defaults().expect("Try to install docker");
    let mut info = docker.build_image(options, None, Some(docker_tar.into()));
    while let Some(msg) = info.next().await {
        println!("Message: {msg:?}");
    }
    format!("portal-testground/{}", id)
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
    pub async fn start_client(client_type: String, container_image: Option<String>) -> Client {
        let docker = Docker::connect_with_socket_defaults().expect("Try to install docker");
        let container_image = match container_image {
            Some(container_image) => container_image,
            None => build_docker_image_for_client(&client_type).await,
        };

        let config = Config {
            image: Some(container_image),
            tty: Some(true),
            ..Default::default()
        };

        let container = docker
            .create_container::<String, String>(None, config)
            .await
            .unwrap()
            .id;
        docker
            .start_container::<String>(&container, None)
            .await
            .unwrap();
        let inspect_result = docker.inspect_container(&container, None).await.unwrap();
        let ip = IpAddr::from_str(&inspect_result.network_settings.unwrap().ip_address.unwrap())
            .expect("Failed to decode IpAddr from string");

        let rpc_url = format!("http://{}:8545", ip);

        let rpc_client = HttpClientBuilder::default()
            .build(rpc_url)
            .expect("Failed to build rpc_client");

        Self {
            kind: client_type,
            container,
            ip,
            rpc: rpc_client,
        }
    }

    pub async fn start_client_local(client_type: String, ip: String) -> Client {
        Command::new(CLIENTS_DIRECTOR.to_string() + "getdocker.sh")
            .env(CLIENT_TYPE, client_type.clone())
            .output()
            .await
            .expect("Failed to execute command");
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

    /// only use this if you are using start_client
    pub async fn stop_client(&self) {
        let docker = Docker::connect_with_socket_defaults().expect("Try to install docker");
        docker
            .stop_container(&self.container, None)
            .await
            .expect("Container didn't stop correctly");
    }
}
