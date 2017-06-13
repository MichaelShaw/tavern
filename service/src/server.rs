
use aphid::HashMap;

use game::{BoardWithMoves, Players};

use std::net::SocketAddr;
use std::{thread};
use std::thread::JoinHandle;
use std::sync::mpsc;

use futures::sync::mpsc::UnboundedSender;


use psyk::game::{GameId, Human, Player};
use psyk::network::server::ServerInboundEvent;
use psyk::network::server::ServerEventHandler;

// some notion of time for cleanup

use {ToClientEvent, ToServerEvent};

#[derive(Debug, Clone)]
pub struct ClientAuth {
    pub sender: UnboundedSender<ToClientEvent>,
    pub auth: AuthState,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AuthState {
    Connected,
    Authenticated(Human),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Client {
    pub socket: SocketAddr,
    pub state: ClientState,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ClientState {
    Lobby,
    InGame(GameId),
}

#[derive(Debug, Clone)]
pub struct ServerGame {
    pub board : BoardWithMoves,
    pub players : Players, 
}

pub struct Server {
    pub client_auth : HashMap<SocketAddr, ClientAuth>,
    pub clients : HashMap<Player, Client>,
    pub games : HashMap<GameId, ServerGame>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            client_auth: HashMap::default(),
            clients : HashMap::default(),
            games: HashMap::default(),
        }
    }
}

fn spawn_tavern_server() -> (ServerEventHandler<ToServerEvent, ToClientEvent>, JoinHandle<u32>) {
    // just verbose due to previous errors :-/
    let (sender, receiver) = mpsc::channel::<ServerInboundEvent<ToServerEvent, ToClientEvent>>();

    let mut server = Server::new();

    let join_handle = thread::spawn(move || {
        use psyk::network::server::ServerInboundEvent::*;

        loop {
            match receiver.recv() {
                Ok(event) => {
                    match event {
                        FailureToBind { address } => {
                            println!("Tavern Server :: failure to bind on address -> {:?}", address);
                            break;
                        },
                        ServerFinished { address } => {
                            println!("Tavern Server :: server process @ {:?} finished", address);
                            break;
                        }
                        ClientConnected { address, client_sender } => {
                            println!("Tavern Server :: client @ {:?} connected :D", address);
                            let client_auth = ClientAuth {
                                sender: client_sender,
                                auth: AuthState::Connected,
                            };
                            server.client_auth.insert(address, client_auth);
                        },
                        ClientMessage { address, event } => {
                            println!("Tavern Server :: received client message -> {:?} from {:?}", event, address);
                            let evs = handle_server_event(&mut server, &event);
                            for ev in evs {
                                if let Some(client_auth) = server.client_auth.get(&ev.socket) {
                                    client_auth.sender.send(ev.event).expect("Tavern Server :: expects to be able to ship a client event");
                                } else {
                                    println!("Tavern Server :: couldn't find client for {:?} ... event will go undelivered", ev)
                                }
                            }
                        },
                        ClientDisconnected { address } => {
                            println!("Tavern Server :: client disconnected {:?}", address);
                            // handle disconnection
                            server.client_auth.remove(&address);
                        },
                    }
                },
                Err(e) => {
                    println!("Tavern Server :: problem receiving event :-( {:?}", e);
                    break;
                },
            }
        }

        32
    });

    (ServerEventHandler {
        sender : sender,
    }, join_handle)
}

#[derive(Debug)]
pub struct EventForClient {
    pub socket: SocketAddr,
    pub event: ToClientEvent,
}

fn handle_server_event(server: &mut Server, event: &ToServerEvent) -> Vec<EventForClient> {
    use psyk::event::to_server::Payload::*;

    let client_events = Vec::new();

    match event.payload {
        Auth => (),
        ListGames => (),
        NewGame => (), 
        JoinGame(game_id) => (), // state transition
        AbandonGame(game_id) => (), // state transition
        GameEvent(game_id, ref game_event) => (),
    }

    client_events
}