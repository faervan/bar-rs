use std::{fs, io::Read, os::unix::net::UnixListener, path::Path};

use daemonize::Daemonize;
use ipc::IpcRequest;
use log::{error, info};

pub fn create_instance(path: &Path) -> anyhow::Result<()> {
    let listener = UnixListener::bind(path)?;

    for mut stream in listener.incoming().flatten() {
        let mut msg = String::new();
        stream.read_to_string(&mut msg)?;
        match ron::from_str::<IpcRequest>(&msg) {
            Ok(cmd) => {
                info!("Got command: {cmd:?}");
                if matches!(cmd, IpcRequest::CloseAll) {
                    info!("Closing `crabbar`");
                    fs::remove_file(path)?;
                    return Ok(());
                }
            }
            Err(e) => error!("Received an invalid IPC command: {msg}\n{e}"),
        }
    }

    Ok(())
}

pub fn daemonize(id: usize, log_dir: &Path, run_dir: &Path) -> anyhow::Result<()> {
    // `TODO`! The log directory `/var/log/crabbar` has to be created and chown'ed
    let stdout = fs::File::create(log_dir.join(format!("crabbar{id}.out")))?;
    let stderr = fs::File::create(log_dir.join(format!("crabbar{id}.err")))?;

    let daemon = Daemonize::new()
        .stdout(stdout)
        .stderr(stderr)
        .pid_file(run_dir.join(format!("crabbar{id}.pid")));

    info!(
        "Daemonizeing this process. \
        Run `crabbar close -i {id}` to terminate the daemon."
    );

    daemon.start()?;

    Ok(())
}
