mod utils;

use crate::utils::{get_client_names, Client};
use ethportal_api::{Discv5ApiClient, Enr, HistoryNetworkApiClient};
use std::borrow::Cow;
use std::fmt::format;
use std::str::FromStr;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = testground::client::Client::new_and_init().await?;

    match client.run_parameters().test_case.as_str() {
        "example" => example(client).await,
        "publish-subscribe" => publish_subscribe(client).await,
        "ping" => ping(client).await,
        _ => panic!("Unknown test case: {}", client.run_parameters().test_case),
    }
}

async fn example(client: testground::client::Client) -> Result<(), Box<dyn std::error::Error>> {
    client.record_message(format!(
        "{}, sdk-rust!",
        client
            .run_parameters()
            .test_instance_params
            .get("greeting")
            .unwrap()
    ));

    client.record_success().await?;

    Ok(())
}

async fn publish_subscribe(
    client: testground::client::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client.record_message("running the publish_subscribe test");

    match client.global_seq() {
        1 => {
            client.record_message("I am instance 1: acting as the leader");

            let json = serde_json::json!({"foo": "bar"});
            client.publish("demonstration", Cow::Owned(json)).await?;
            client.record_success().await?;
        }
        _ => {
            client.record_message(format!(
                "I am instance {}: acting as a follower",
                client.global_seq()
            ));

            let payload = client
                .subscribe("demonstration", u16::MAX.into())
                .await
                .take(1)
                .map(|x| x.unwrap())
                .next()
                .await
                .unwrap();

            client.record_message(format!("I received the payload: {}", payload));

            if payload["foo"].as_str() == Some("bar") {
                client.record_success().await?;
            } else {
                client
                    .record_failure(format!("invalid payload: {}", payload))
                    .await?;
            }
        }
    }
    Ok(())
}

async fn ping(client: testground::client::Client) -> Result<(), Box<dyn std::error::Error>> {
    client.record_message("running the publish_subscribe test");
    let global_seq = client.global_seq();
    match global_seq {
        1 => {
            client.record_message("I am instance 1: acting as the leader");
            let client_type = client
                .run_parameters()
                .test_instance_params
                .get(&format!("client{}", global_seq))
                .unwrap()
                .clone();
            if !get_client_names().contains(&client_type) {
                client
                    .record_failure(format!(
                        "invalid {}: {}",
                        &format!("client{}", global_seq),
                        &client_type
                    ))
                    .await?;
                return Ok(());
            }
            let portal_client = Client::start_client(client_type, None).await;
            let our_enr = match portal_client.rpc.node_info().await {
                Ok(node_info) => node_info.enr,
                Err(err) => {
                    panic!("Error getting node info: {err:?}");
                }
            };

            let json = serde_json::json!({"enr1": our_enr.to_base64()});
            client.publish("send-to-node-2", Cow::Owned(json)).await?;

            let payload = client
                .subscribe("send-to-node-1", u16::MAX.into())
                .await
                .take(1)
                .map(|x| x.unwrap())
                .next()
                .await
                .unwrap();

            client.record_message(format!("I received the payload: {}", payload));

            if let Some(enr) = payload.get("send-to-node-1") {
                let target_enr = Enr::from_str(enr.as_str().unwrap()).unwrap();

                if let Err(err) = portal_client.rpc.ping(target_enr).await {
                    panic!("Unable to receive pong info: {err:?}");
                }
            } else {
                client
                    .record_failure(format!("invalid payload: {}", payload))
                    .await?;
                return Ok(());
            }

            portal_client.stop_client().await;
            client.record_success().await?;
        }
        _ => {
            client.record_message(format!(
                "I am instance {}: acting as a follower",
                global_seq
            ));

            let client_type = client
                .run_parameters()
                .test_instance_params
                .get(&format!("client{}", global_seq))
                .unwrap()
                .clone();
            if !get_client_names().contains(&client_type) {
                client
                    .record_failure(format!(
                        "invalid {}: {}",
                        &format!("client{}", global_seq),
                        &client_type
                    ))
                    .await?;
                return Ok(());
            }
            let portal_client = Client::start_client(client_type, None).await;
            let our_enr = match portal_client.rpc.node_info().await {
                Ok(node_info) => node_info.enr,
                Err(err) => {
                    panic!("Error getting node info: {err:?}");
                }
            };

            let payload = client
                .subscribe("send-to-node-2", u16::MAX.into())
                .await
                .take(1)
                .map(|x| x.unwrap())
                .next()
                .await
                .unwrap();

            client.record_message(format!("I received the payload: {}", payload));

            if let Some(enr) = payload.get("send-to-node-2") {
                let target_enr = Enr::from_str(enr.as_str().unwrap()).unwrap();

                if let Err(err) = portal_client.rpc.ping(target_enr).await {
                    panic!("Unable to receive pong info: {err:?}");
                }
            } else {
                client
                    .record_failure(format!("invalid payload: {}", payload))
                    .await?;
                return Ok(());
            }

            let json = serde_json::json!({"enr2": our_enr.to_base64()});
            client.publish("send-to-node-1", Cow::Owned(json)).await?;
            portal_client.stop_client().await;
            client.record_success().await?;
        }
    }
    Ok(())
}
