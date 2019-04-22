use futures::{sync::mpsc, Async, Future, Poll, Stream};
use std::path::Path;

mod error;
mod importer;
mod register;

pub use self::error::Error;

use crate::bootnode::BootnodeClient;
use crate::primitives::EthereumProgram;

use self::importer::Importer;
use self::register::Register;

type Web3 = web3::Web3<web3::transports::Ipc>;

enum Event {
    ImportPeers,
    RegisterEthereumNode,
    Shutdown,
}

pub struct NetworkKeeper {
    web3_ipc: web3::transports::Ipc,
    importer: Importer,
    register: Register,

    event_sender: mpsc::UnboundedSender<Event>,
    event_receiver: mpsc::UnboundedReceiver<Event>,
}

impl NetworkKeeper {
    pub fn new<P>(
        network_name: String,
        ethereum_program: EthereumProgram,
        bootnode_host: String,
        bootnode_port: u16,
        ethereum_node_endpoint: P,
    ) -> NetworkKeeper
    where
        P: AsRef<Path>,
    {
        let (event_sender, event_receiver) = mpsc::unbounded();
        let web3_ipc = web3::transports::Ipc::new(ethereum_node_endpoint).unwrap();
        let web3 = web3::Web3::new(web3_ipc.clone());
        let bootnode_client = BootnodeClient::new(bootnode_host, bootnode_port);

        let register = Register::new(
            ethereum_program,
            network_name.clone(),
            web3.clone(),
            bootnode_client.clone(),
        );

        let importer = Importer::new(ethereum_program, network_name, web3, bootnode_client);

        NetworkKeeper {
            web3_ipc,
            importer,
            register,

            event_sender,
            event_receiver,
        }
    }

    #[inline]
    pub fn register_enode(&mut self) {
        self.send_event(Event::RegisterEthereumNode);
    }

    #[inline]
    pub fn import_peers(&mut self) {
        self.send_event(Event::ImportPeers);
    }

    #[inline]
    #[allow(unused)]
    pub fn shutdown(&mut self) {
        self.send_event(Event::Shutdown);
    }

    #[inline]
    fn send_event(&mut self, event: Event) {
        if !self.event_sender.is_closed() {
            self.event_sender
                .unbounded_send(event)
                .expect("receiver always existed; qed");
        }
    }
}

impl Future for NetworkKeeper {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let _ = self.web3_ipc.poll();
        let _ = self.register.poll();
        let _ = self.importer.poll();

        match self.event_receiver.poll() {
            Ok(Async::Ready(Some(event))) => match event {
                Event::Shutdown => Ok(Async::Ready(())),
                Event::ImportPeers => {
                    self.importer.import();
                    Ok(Async::NotReady)
                }
                Event::RegisterEthereumNode => {
                    self.register.update();
                    Ok(Async::NotReady)
                }
            },
            _ => Ok(Async::NotReady),
        }
    }
}
