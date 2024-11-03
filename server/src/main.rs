use std::time::Duration;

use anyhow::Result;
use tracing::{error, info};
use wtransport::{
    endpoint::IncomingSession, tls::Sha256DigestFmt, Endpoint, Identity, RecvStream, SendStream,
    ServerConfig,
};

fn init_logging() {
    tracing_subscriber::fmt()
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer()
        .with_target(false)
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    let identity = Identity::self_signed(["localhost", "127.0.0.1", "::1"])?;
    info!(
        "Certificate hash: {}",
        identity.certificate_chain().as_slice()[0]
            .hash()
            .fmt(Sha256DigestFmt::BytesArray)
    );
    let port = 3030;
    let config = ServerConfig::builder()
        .with_bind_default(port)
        .with_identity(identity)
        .keep_alive_interval(Some(Duration::from_secs(3)))
        .build();

    let server = Endpoint::server(config)?;
    info!("Started WebTransport server on port {}", port);

    loop {
        let incoming_session = server.accept().await;
        tokio::spawn(handle_session(incoming_session));
    }
}

async fn handle_session(incoming_session: IncomingSession) -> Result<()> {
    let session_request = incoming_session.await?;

    let connection = session_request.accept().await?;

    loop {
        tokio::select! {
            stream = connection.accept_bi() => {
                let stream = stream?;
                tokio::spawn(handle_bi_stream(stream));
            }
            stream = connection.accept_uni() => {
                let stream = stream?;
                tokio::spawn(handle_uni_stream(stream));
            }
            datagram = connection.receive_datagram() => {
                let datagram = datagram?;
                let string = std::str::from_utf8(&datagram)?;
                info!("Recieved Datagram: {}", string);
                connection.send_datagram(format!("Recieved: {}", string).as_bytes())?;
            }
        }
    }
}

async fn handle_bi_stream(mut stream: (SendStream, RecvStream)) -> Result<()> {
    loop {
        let mut buffer = vec![0; 65536].into_boxed_slice();
        let bytes_num = match stream.1.read(&mut buffer).await? {
            Some(bytes_num) => bytes_num,
            None => continue,
        };

        let string = std::str::from_utf8(&buffer[..bytes_num])?;

        info!("Recieved from bistream: {}", string);

        stream
            .0
            .write_all(format!("Recieved {}", string).as_bytes())
            .await?;
    }
}

async fn handle_uni_stream(mut stream: RecvStream) -> Result<()> {
    loop {
        let mut buffer = vec![0; 65536].into_boxed_slice();
        let bytes_num = match stream.read(&mut buffer).await? {
            Some(bytes_num) => bytes_num,
            None => continue,
        };

        let string = std::str::from_utf8(&buffer[..bytes_num])?;

        info!("Recieved from unistream: {}", string);
    }
}
