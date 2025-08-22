use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
};

use daemonize::Daemonize;
use iced::futures::{channel::mpsc::Sender, SinkExt as _};
use log::{error, info};
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::{UnixListener, UnixStream},
    sync::oneshot,
};

use crate::{ipc::IpcRequest, message::Message, state::State, window::WindowRuntimeOptions};

pub fn run(
    open_window: bool,
    window_opts: WindowRuntimeOptions,
    daemonize: bool,
    log_dir: &Path,
    socket_path: PathBuf,
    pid_path: PathBuf,
) -> anyhow::Result<()> {
    if daemonize {
        daemonize_process(log_dir)?;
    }

    let mut pid_file = fs::File::create(&pid_path)?;
    pid_file.write_all(std::process::id().to_string().as_bytes())?;
    drop(pid_file);

    iced::daemon(State::title, State::update, State::view)
        .subscription(State::subscribe)
        .theme(State::theme)
        .run_with(move || State::new(socket_path, pid_path, open_window, window_opts))?;

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
    sender: Sender<Message>,
    listener: UnixListener,
) -> anyhow::Result<()> {
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let sender = sender.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, sender).await {
                        error!("Communication with client failed: {e}")
                    }
                });
            }
            Err(e) => error!("Failed to accept an incoming IPC connection: {e}"),
        }
    }
}

async fn handle_client(mut stream: UnixStream, mut sender: Sender<Message>) -> anyhow::Result<()> {
    let mut msg_len = [0; 4];
    stream.read_exact(&mut msg_len).await?;
    let msg_len = u32::from_ne_bytes(msg_len) as usize;

    let mut msg = vec![0; msg_len];
    stream.read_exact(&mut msg).await?;
    let msg = String::from_utf8_lossy(&msg);

    match ron::from_str::<IpcRequest>(&msg) {
        Ok(cmd) => {
            let (sx, rx) = oneshot::channel();
            sender
                .send(Message::IpcCommand {
                    request: cmd,
                    responder: sx,
                })
                .await?;
            let response = rx.await?;
            stream
                .write_all(ron::to_string(&response)?.as_bytes())
                .await?;
        }
        Err(e) => error!("Received an invalid IPC command: {msg}\n{e}"),
    }
    Ok(())
}

pub fn exit_cleanup(socket_path: &Path, pid_path: &Path) {
    if let Err(e) = fs::remove_file(&socket_path) {
        error!("Could not remove socket file at {socket_path:?}: {e}");
    }
    if let Err(e) = fs::remove_file(&pid_path) {
        error!("Could not remove PID file at {pid_path:?}: {e}");
    }
}

fn daemonize_process(log_dir: &Path) -> anyhow::Result<()> {
    let path = log_dir.join("crabbar.log");
    let file = fs::File::create(&path)?;

    let daemon = Daemonize::new().stdout(file.try_clone()?).stderr(file);

    info!(
        "Daemonizeing this process. \
        Run `crabbar close` to terminate the daemon."
    );

    daemon.start()?;

    Ok(())
}
