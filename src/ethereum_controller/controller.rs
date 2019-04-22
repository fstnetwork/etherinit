use futures::{sync::oneshot, Async, Future, Poll};
use tokio_process::Child as ChildProcess;

use crate::ethereum_launcher::EthereumLauncher;

use super::{Error, RestartPolicy};

pub struct Controller {
    restart_policy: RestartPolicy,
    ethereum_launcher: EthereumLauncher,
    ethereum_process: Option<ChildProcess>,

    shutdown_sender: Option<oneshot::Sender<()>>,
    shutdown_receiver: oneshot::Receiver<()>,
}

impl Controller {
    pub fn new(ethereum_launcher: EthereumLauncher, restart_policy: RestartPolicy) -> Controller {
        let (shutdown_sender, shutdown_receiver) = oneshot::channel();

        let ethereum_process = Some(
            ethereum_launcher
                .execute_async()
                .expect("spawn Ethereum client process"),
        );

        Controller {
            restart_policy,
            ethereum_launcher,
            ethereum_process,

            shutdown_receiver,
            shutdown_sender: Some(shutdown_sender),
        }
    }

    pub fn restart(&mut self) {
        std::mem::replace(
            &mut self.ethereum_process,
            Some(
                self.ethereum_launcher
                    .execute_async()
                    .expect("spawn Ethereum client process"),
            ),
        );
    }

    #[allow(dead_code)]
    pub fn close(&mut self) {
        if let Some(sender) = std::mem::replace(&mut self.shutdown_sender, None) {
            sender.send(()).expect("receiver always exists");
        }
    }
}

impl Future for Controller {
    type Item = bool;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Error> {
        if let Ok(Async::Ready(_)) = self.shutdown_receiver.poll() {
            self.shutdown_receiver.close();
            std::mem::replace(&mut self.ethereum_process, None);
            return Ok(Async::Ready(true));
        }

        match self.ethereum_process {
            None => Ok(Async::NotReady),
            Some(ref mut process) => match process.poll() {
                Err(err) => Err(Error::from(err)),
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Ok(Async::Ready(exit_status)) => match self.restart_policy {
                    RestartPolicy::No => Ok(Async::Ready(exit_status.success())),
                    RestartPolicy::OnFailure | RestartPolicy::Always => {
                        self.restart();
                        Ok(Async::NotReady)
                    }
                },
            },
        }
    }
}
