use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::io::{self, AsyncBufReadExt};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufWriter, Stdout};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};

use crate::{user, HELP};

async fn register_bot(
    write: &mut SplitSink<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, Message>,
    bot_name: &str,
) {
    let registration_message = Message::Text(format!("register as {}", bot_name));
    write
        .send(registration_message)
        .await
        .expect("Failed to send registration message");
}
pub async fn send_message(
    write: &mut SplitSink<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, Message>,
    msg: &str,
) {
    write
        .send(Message::Text(msg.to_string()))
        .await
        .expect("Failed to send registration message");
}

pub async fn handle_incoming_messages(
    mut read: SplitStream<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>>,
) {
    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => println!("Received a message: {}", msg),
            Err(e) => eprintln!("Error receiving message: {}", e),
        }
    }
}

async fn update(client: &mut user::User, group_id: Option<String>, writer: &mut BufWriter<Stdout>) {
    let messages = client.update(group_id).await.unwrap();
    writer.write_all(b" >>> Updated client :)\n").await.unwrap();
    writer.flush().await.expect("Failed to flush stdout");
    if !messages.is_empty() {
        writer.write_all(b"     New messages:\n\n").await.unwrap();
        writer.flush().await.expect("Failed to flush stdout");
    }

    for cm in messages.iter() {
        writer
            .write_all(format!("         {0} from {1}\n", cm.message, cm.author).as_bytes())
            .await
            .unwrap();
        writer.flush().await.expect("Failed to flush stdout");
    }
    writer.write_all(b"\n").await.unwrap();
}

pub async fn connect_websocket(url: &str) {
    // let url = "ws://localhost:6543";

    println!("Connecting to - {}", url);
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("Connected to Agent Network");

    let (mut write, mut read) = ws_stream.split();

    // register the timebot
    register_bot(&mut write, "RustClient").await;

    // Handle incoming messages in a separate task
    let read_handle = tokio::spawn(handle_incoming_messages(read));

    // Read from command line and send messages
    let write_handle = tokio::spawn(read_and_send_messages(write));

    // Await both tasks (optional, depending on your use case)
    let _ = tokio::try_join!(read_handle, write_handle);
}

pub async fn read_and_send_messages(
    mut write: SplitSink<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, Message>,
) {
    let mut reader = io::BufReader::new(io::stdin()).lines();

    let mut writer = io::BufWriter::new(io::stdout());
    let mut client = None;

    // writer
    //     .write_all("writing...".as_bytes())
    //     .await
    //     .expect("Failed to write to stdout");
    // writer.flush().await.expect("Failed to flush stdout");

    while let Some(line) = reader.next_line().await.expect("Failed to read line") {
        if !line.trim().is_empty() {
            write
                .send(Message::Text(line.clone()))
                .await
                .expect("Failed to send message");
        }

        if line == "help" {
            writer
                .write_all(HELP.as_bytes())
                .await
                .expect("Failed to write to stdout");
            writer.flush().await.expect("Failed to flush stdout");
            continue;
        }

        if let Some(client_name) = line.strip_prefix("register ") {
            // send_message(ws_stream.clone(), "send from register command");
            client = Some(user::User::new(client_name.to_string()));
            println!(
                "client identity: {:?}",
                client
                    .as_ref()
                    .unwrap()
                    .identity
                    .read()
                    .await
                    .identity_as_string()
            );
            client.as_mut().unwrap().add_key_package().await;
            client.as_mut().unwrap().add_key_package().await;
            client.as_mut().unwrap().register(&mut write).await;
            writer
                .write_all(format!("registered new client {client_name}\n\n").as_bytes())
                .await
                .expect("Failed to write to stdout");
            continue;
        }

        //         // Create a new KeyPackage.
        if line == "create kp" {
            if let Some(client) = &mut client {
                client.create_kp().await;
                writer
                    .write_all(b" >>> New key package created\n\n")
                    .await
                    .expect("Failed to write to stdout");
                writer.flush().await.expect("Failed to flush stdout");
            } else {
                writer
                    .write_all(b" >>> No client to update :(\n\n")
                    .await
                    .unwrap();
                writer.flush().await.expect("Failed to flush stdout");
            }
            continue;
        }

        // Create a new group.
        if let Some(group_name) = line.strip_prefix("create group ") {
            if let Some(client) = &mut client {
                client.create_group(group_name.to_string()).await;
                writer
                    .write_all(format!(" >>> Created group {group_name} :)\n\n").as_bytes())
                    .await
                    .unwrap();
                writer.flush().await.expect("Failed to flush stdout");
            } else {
                writer
                    .write_all(b" >>> No client to create a group :(\n\n")
                    .await
                    .unwrap();
                writer.flush().await.expect("Failed to flush stdout");
            }
            continue;
        }

        // Update the client state.
        if line == "update" {
            if let Some(client) = &mut client {
                update(client, None, &mut writer).await;
            } else {
                writer
                    .write_all(b" >>> No client to update :(\n\n")
                    .await
                    .unwrap();
            }
            continue;
        }

        // Group operations.
        if let Some(group_name) = line.strip_prefix("group ") {
            if let Some(client) = &mut client {
                loop {
                    writer.write_all(b" > ").await.unwrap();
                    writer.flush().await.expect("Failed to flush stdout");
                    // writer
                    //     .write_all(b" >>> No client to create a group :(\n\n")
                    //     .await
                    //     .unwrap();
                    // writer.flush().await.expect("Failed to flush stdout");
                    if let Some(op2) = reader.next_line().await.expect("Failed to read line") {
                        // Send a message to the group.
                        if let Some(msg) = op2.strip_prefix("send ") {
                            match client.send_msg(msg, group_name.to_string()).await {
                                Ok(()) => {
                                    writer
                                        .write_all(
                                            format!("sent message to {group_name}\n\n").as_bytes(),
                                        )
                                        .await
                                        .unwrap();
                                    writer.flush().await.expect("Failed to flush stdout")
                                }
                                Err(e) => println!("Error sending group message: {e:?}"),
                            }
                            continue;
                        }

                        // Invite a client to the group.
                        if let Some(new_client) = op2.strip_prefix("invite ") {
                            client
                                .invite(new_client.to_string(), group_name.to_string())
                                .await
                                .unwrap();
                            writer
                                .write_all(
                                    format!("added {new_client} to group {group_name}\n\n")
                                        .as_bytes(),
                                )
                                .await
                                .unwrap();
                            writer.flush().await.expect("Failed to flush stdout");
                            continue;
                        }

                        // Remove a client from the group.
                        if let Some(rem_client) = op2.strip_prefix("remove ") {
                            client
                                .remove(rem_client.to_string(), group_name.to_string())
                                .await
                                .unwrap();
                            writer
                                .write_all(
                                    format!("Removed {rem_client} from group {group_name}\n\n")
                                        .as_bytes(),
                                )
                                .await
                                .unwrap();
                            writer.flush().await.expect("Failed to flush stdout");
                            continue;
                        }

                        // Read messages sent to the group.
                        if op2 == "read" {
                            let messages = client.read_msgs(group_name.to_string()).await.unwrap();
                            if let Some(messages) = messages {
                                writer
                                    .write_all(
                                        format!(
                                            "{} has received {} messages\n\n",
                                            group_name,
                                            messages.len()
                                        )
                                        .as_bytes(),
                                    )
                                    .await
                                    .unwrap();
                                writer.flush().await.expect("Failed to flush stdout");
                            } else {
                                writer
                                    .write_all(
                                        format!("{group_name} has no messages\n\n").as_bytes(),
                                    )
                                    .await
                                    .unwrap();
                                writer.flush().await.expect("Failed to flush stdout");
                            }
                            continue;
                        }

                        // Update the client state.
                        if op2 == "update" {
                            println!("Updating client");
                            update(client, Some(group_name.to_string()), &mut writer).await;
                            continue;
                        }

                        // Exit group.
                        if op2 == "exit" {
                            writer.write_all(b" >>> Leaving group \n\n").await.unwrap();
                            writer.flush().await.expect("Failed to flush stdout");
                            break;
                        }

                        writer
                            .write_all(b" >>> Unknown group command :(\n\n")
                            .await
                            .unwrap();
                    }
                    writer.flush().await.expect("Failed to flush stdout");
                }
            } else {
                writer.write_all(b" >>> No client :(\n\n").await.unwrap();
                writer.flush().await.expect("Failed to flush stdout");
            }
            continue;
        }
    }
}
