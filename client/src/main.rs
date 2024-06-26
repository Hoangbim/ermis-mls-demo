// #[macro_use]
// extern crate clap;
// use clap::App;

use futures_util::{
    io::BufReader,
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    io::{self, stdin, stdout, StdoutLock, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread,
};
use termion::input::TermRead;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
    runtime::Runtime,
    sync::{Mutex, RwLock},
};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use ws_client::connect_websocket;

mod backend;
mod conversation;
mod identity;
mod long_polling;
mod networking;
mod openmls_rust_persistent_crypto;
mod serialize_any_hashmap;
mod user;
mod ws_client;

const HELP: &str = "
>>> Available commands:
>>>     - update                                update the client state
>>>     - reset                                 reset the server
>>>     - register {client name}                register a new client
>>>     - save {client name}                    serialize and save the client state
>>>     - load {client name}                    load and deserialize the client state as a new client
>>>     - autosave                              enable automatic save of the current client state upon each update
>>>     - create kp                             create a new key package
>>>     - create group {group name}             create a new group
>>>     - group {group name}                    group operations
>>>         - send {message}                    send message to group
>>>         - invite {client name}              invite a user to the group
>>>         - read                              read messages sent to the group (max 100)
>>>         - update                            update the client state

";



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    // let (tx, rx): (Sender<()>, Receiver<()>) = std::sync::mpsc::channel();

    // tokio::spawn(async move {
    connect_websocket("ws://0.0.0.0:6543").await;
    // });

    // handle_command(rx);

    Ok(())
}
// fn main() -> anyhow::Result<()> {
//     pretty_env_logger::init();

//     let (tx, rx): (Sender<()>, Receiver<()>) = std::sync::mpsc::channel();

//     let handle = thread::spawn(move || {
//         let rt = Runtime::new().unwrap();
//         rt.block_on(async {
//             connect_websocket("ws://0.0.0.0:6543").await;
//         });
//         // tx.send(()).unwrap();
//     });

//     handle.join().unwrap();
//     let handle_main = thread::spawn(move || {
//         handle_command(rx);
//     });
//     handle_main.join().unwrap();

//     Ok(())
// }





// fn handle_command() {
//     let stdout = stdout();
//     let mut stdout = stdout.lock();
//     let stdin = stdin();
//     let mut stdin = stdin.lock();

//     // send_message(ws_stream.clone(), "Hello WebSocket");

//     stdout
//         .write_all(b" >>> Welcome to the OpenMLS CLI :)\nType help to get a list of commands\n\n")
//         .unwrap();
//     let mut client = None;
//     // long_polling::call_long_polling();
//     // health check
//     let backend = backend::Backend::default();
//     match backend.health_check() {
//         Ok(()) => {
//             stdout
//                 .write_all(b" >>> Health check passed :)\n\n")
//                 .unwrap();
//         }
//         Err(e) => {
//             stdout
//                 .write_all(format!(" >>> Health check failed: {e}\n\n").as_bytes())
//                 .unwrap();
//         }
//     }

//     loop {
//         stdout.flush().unwrap();
//         let op = stdin.read_line().unwrap().unwrap();

//         // Register a client.
//         // There's no persistence. So once the client app stops you have to
//         // register a new client.
//         if let Some(client_name) = op.strip_prefix("register ") {
//             // send_message(ws_stream.clone(), "send from register command");
//             client = Some(user::User::new(client_name.to_string()));
//             client.as_mut().unwrap().add_key_package();
//             client.as_mut().unwrap().add_key_package();
//             client.as_mut().unwrap().register();
//             stdout
//                 .write_all(format!("registered new client {client_name}\n\n").as_bytes())
//                 .unwrap();
//             continue;
//         }

//         if let Some(client_name) = op.strip_prefix("load ") {
//             match user::User::load(client_name.to_string()) {
//                 Ok(user) => {
//                     client = Some(user);
//                     stdout
//                         .write_all(format!("recovered client {client_name}\n\n").as_bytes())
//                         .unwrap();
//                 }
//                 Err(e) => stdout
//                     .write_all(
//                         format!("Error recovering client {client_name} : {e}\n\n").as_bytes(),
//                     )
//                     .unwrap(),
//             }
//             continue;
//         }

//         // Create a new KeyPackage.
//         if op == "create kp" {
//             if let Some(client) = &mut client {
//                 client.create_kp();
//                 stdout
//                     .write_all(b" >>> New key package created\n\n")
//                     .unwrap();
//             } else {
//                 stdout
//                     .write_all(b" >>> No client to update :(\n\n")
//                     .unwrap();
//             }
//             continue;
//         }

//         // Save the current client state.
//         if op == "save" {
//             if let Some(client) = &mut client {
//                 client.save();
//                 let name = &client.identity.borrow().identity_as_string();
//                 stdout
//                     .write_all(format!(" >>> client {name} state saved\n\n").as_bytes())
//                     .unwrap();
//             } else {
//                 stdout
//                     .write_all(b" >>> No client to update :(\n\n")
//                     .unwrap();
//             }
//             continue;
//         }

//         // Enable automatic saving of the client state.
//         if op == "autosave" {
//             if let Some(client) = &mut client {
//                 client.enable_auto_save();
//                 let name = &client.identity.borrow().identity_as_string();
//                 stdout
//                     .write_all(format!(" >>> autosave enabled for client {name} \n\n").as_bytes())
//                     .unwrap();
//             } else {
//                 stdout
//                     .write_all(b" >>> No client to update :(\n\n")
//                     .unwrap();
//             }
//             continue;
//         }

//         // Create a new group.
//         if let Some(group_name) = op.strip_prefix("create group ") {
//             if let Some(client) = &mut client {
//                 client.create_group(group_name.to_string());
//                 stdout
//                     .write_all(format!(" >>> Created group {group_name} :)\n\n").as_bytes())
//                     .unwrap();
//             } else {
//                 stdout
//                     .write_all(b" >>> No client to create a group :(\n\n")
//                     .unwrap();
//             }
//             continue;
//         }

//         // Group operations.
//         if let Some(group_name) = op.strip_prefix("group ") {
//             if let Some(client) = &mut client {
//                 loop {
//                     stdout.write_all(b" > ").unwrap();
//                     stdout.flush().unwrap();
//                     let op2 = stdin.read_line().unwrap().unwrap();

//                     // Send a message to the group.
//                     if let Some(msg) = op2.strip_prefix("send ") {
//                         match client.send_msg(msg, group_name.to_string()) {
//                             Ok(()) => stdout
//                                 .write_all(format!("sent message to {group_name}\n\n").as_bytes())
//                                 .unwrap(),
//                             Err(e) => println!("Error sending group message: {e:?}"),
//                         }
//                         continue;
//                     }

//                     // Invite a client to the group.
//                     if let Some(new_client) = op2.strip_prefix("invite ") {
//                         client
//                             .invite(new_client.to_string(), group_name.to_string())
//                             .unwrap();
//                         stdout
//                             .write_all(
//                                 format!("added {new_client} to group {group_name}\n\n").as_bytes(),
//                             )
//                             .unwrap();
//                         continue;
//                     }

//                     // Remove a client from the group.
//                     if let Some(rem_client) = op2.strip_prefix("remove ") {
//                         client
//                             .remove(rem_client.to_string(), group_name.to_string())
//                             .unwrap();
//                         stdout
//                             .write_all(
//                                 format!("Removed {rem_client} from group {group_name}\n\n")
//                                     .as_bytes(),
//                             )
//                             .unwrap();
//                         continue;
//                     }

//                     // Read messages sent to the group.
//                     if op2 == "read" {
//                         let messages = client.read_msgs(group_name.to_string()).unwrap();
//                         if let Some(messages) = messages {
//                             stdout
//                                 .write_all(
//                                     format!(
//                                         "{} has received {} messages\n\n",
//                                         group_name,
//                                         messages.len()
//                                     )
//                                     .as_bytes(),
//                                 )
//                                 .unwrap();
//                         } else {
//                             stdout
//                                 .write_all(format!("{group_name} has no messages\n\n").as_bytes())
//                                 .unwrap();
//                         }
//                         continue;
//                     }

//                     // Update the client state.
//                     if op2 == "update" {
//                         update(client, Some(group_name.to_string()), &mut stdout);
//                         continue;
//                     }

//                     // Exit group.
//                     if op2 == "exit" {
//                         stdout.write_all(b" >>> Leaving group \n\n").unwrap();
//                         break;
//                     }

//                     stdout
//                         .write_all(b" >>> Unknown group command :(\n\n")
//                         .unwrap();
//                 }
//             } else {
//                 stdout.write_all(b" >>> No client :(\n\n").unwrap();
//             }
//             continue;
//         }

//         // Update the client state.
//         if op == "update" {
//             if let Some(client) = &mut client {
//                 update(client, None, &mut stdout);
//             } else {
//                 stdout
//                     .write_all(b" >>> No client to update :(\n\n")
//                     .unwrap();
//             }
//             continue;
//         }

//         // Reset the server and client.
//         if op == "reset" {
//             backend::Backend::default().reset_server();
//             client = None;
//             stdout.write_all(b" >>> Reset server :)\n\n").unwrap();
//             continue;
//         }

//         // Print help
//         if op == "help" {
//             stdout.write_all(HELP.as_bytes()).unwrap();
//             continue;
//         }

//         stdout
//             .write_all(b" >>> unknown command :(\n >>> try help\n\n")
//             .unwrap();

       
//     }
// }

