use std::{
    fs,
    path::{Path, PathBuf},
};

use daemonize::Daemonize;
use iced::futures::{channel::mpsc::Sender, SinkExt as _};
use ipc::IpcRequest;
use log::{error, info};
use tokio::{io::AsyncReadExt as _, net::UnixListener, sync::oneshot};

use crate::{message::Message, state::State};

pub fn run(
    open_window: bool,
    daemonize: bool,
    log_dir: &Path,
    socket_path: PathBuf,
) -> anyhow::Result<()> {
    if daemonize {
        daemonize_process(
            log_dir,
            socket_path
                .parent()
                .ok_or(anyhow::anyhow!("RUN_DIR can't be empty"))?,
        )?;
    }

    iced::daemon(State::title, State::update, State::view)
        .subscription(State::subscribe)
        .run_with(move || State::new(socket_path, open_window))?;

    Ok(())
}

pub async fn bind_to_ipc(sender: &mut Sender<Message>) -> anyhow::Result<UnixListener> {
    let (sx, rx) = oneshot::channel();
    sender
        .send(Message::read_state(move |state| {
            sx.send(state.socket_path.clone()).unwrap();
        }))
        .await?;
    let socket_path = rx.await?;
    Ok(UnixListener::bind(&socket_path)?)
}

pub async fn publish_ipc_commands(
    mut sender: Sender<Message>,
    listener: UnixListener,
) -> anyhow::Result<()> {
    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                let mut msg = String::new();
                stream.read_to_string(&mut msg).await?;
                match ron::from_str::<IpcRequest>(&msg) {
                    Ok(cmd) => sender.send(Message::IpcCommand(cmd)).await?,
                    Err(e) => error!("Received an invalid IPC command: {msg}\n{e}"),
                }
            }
            Err(e) => error!("Failed to accept an incoming IPC connection: {e}"),
        }
    }
}

fn daemonize_process(log_dir: &Path, run_dir: &Path) -> anyhow::Result<()> {
    let path = log_dir.join("crabbar.log");
    let stdout = fs::File::create(&path)?;
    let stderr = fs::File::options().write(true).open(&path)?;

    let daemon = Daemonize::new()
        .stdout(stdout)
        .stderr(stderr)
        .pid_file(run_dir.join("crabbar.pid"));

    info!(
        "Daemonizeing this process. \
        Run `crabbar close` to terminate the daemon."
    );

    daemon.start()?;

    Ok(())
}
