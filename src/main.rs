use std::time::Duration;

use clap::Parser;
use rumqttc::{Client, MqttOptions, QoS, Transport};

mod clean_retained;
mod cli;
mod log;
mod mqtt;
mod publish;
mod read_one;

fn main() -> anyhow::Result<()> {
    let matches = cli::Cli::parse();

    let (client, connection) = setup_mqtt_client(&matches)?;

    match &matches.subcommands {
        Some(cli::SubCommands::CleanRetained { topic, dry_run, timeout }) => {
            client.subscribe(topic, QoS::AtLeastOnce)?;
            clean_retained::clean_retained(&client, &connection, *dry_run, *timeout);
        }
        Some(cli::SubCommands::Log { topic, verbose }) => {
            for topic in topic {
                client.subscribe(topic, QoS::AtLeastOnce)?;
            }
            log::show(&connection, *verbose);
        }
        Some(cli::SubCommands::ReadOne { topic, ignore_retained }) => {
            for topic in topic {
                client.subscribe(topic, QoS::AtLeastOnce)?;
            }
            read_one::show(&client, &connection, *ignore_retained);
        }
        Some(cli::SubCommands::Publish {
            topic,
            payload,
            retain,
            verbose,
        }) => {
            client.publish(topic, QoS::AtLeastOnce, *retain, payload)?;
            publish::eventloop(&client, &connection, *verbose);
        }
        None => {
            interactive::show(&client, &connection, &matches.broker, &matches.topic)?;
            client.disconnect()?;
        }
    }

    Ok(())
}

fn setup_mqtt_client(matches: &cli::Cli) -> anyhow::Result<(Client, rumqttc::Connection)> {
    let (transport, host, port) = match &matches.broker {
        cli::Broker::Tcp { host, port } => (Transport::Tcp, host, *port),
        cli::Broker::Ssl { host, port } => {
            let tls_config = mqtt::encryption::create_tls_configuration(
                matches.insecure,
                &matches.client_cert,
                &matches.client_key,
            )?;
            (Transport::Tls(tls_config), host, *port)
        }
        cli::Broker::WebSocket(url) => (Transport::Ws, url, 666),
        cli::Broker::WebSocketSsl(url) => {
            let tls_config = mqtt::encryption::create_tls_configuration(
                matches.insecure,
                &matches.client_cert,
                &matches.client_key,
            )?;
            (Transport::Wss(tls_config), url, 666)
        }
    };

    let client_id = matches
        .client_id
        .as_deref()
        .unwrap_or_else(|| &format!("mqttui-{:x}", rand::random::<u32>()));

    let mut mqttoptions = MqttOptions::new(client_id, host.clone(), port);
    mqttoptions.set_max_packet_size(usize::MAX, usize::MAX);
    mqttoptions.set_transport(transport);

    if let (Some(username), Some(password)) = (&matches.username, &matches.password) {
        mqttoptions.set_credentials(username, password);
    }

    if let Some(SubCommands::CleanRetained { timeout, .. }) = &matches.subcommands {
        mqttoptions.set_keep_alive(Duration::from_secs_f32(*timeout));
    }

    Ok((Client::new(mqttoptions, 10), rumqttc::Connection::new()))
}
