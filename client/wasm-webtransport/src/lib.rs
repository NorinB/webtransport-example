use std::str::FromStr;
use wasm_bindgen::prelude::*;
use web_sys::console;
use web_sys::js_sys::{Array, JsString};
use xwt_core::prelude::*;
use xwt_web_sys::{
    CertificateHash, Endpoint, HashAlgorithm, RecvStream, SendStream, Session, WebTransportOptions,
};

#[wasm_bindgen]
pub struct WebTransportClient {
    session: Option<Session>,
    first_receive_stream: Option<RecvStream>,
    second_receive_stream: Option<RecvStream>,
}

#[wasm_bindgen]
impl WebTransportClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            session: None,
            first_receive_stream: None,
            second_receive_stream: None,
        }
    }

    pub async fn init_session(&mut self, url: String, certificate_bytes: Vec<u8>) {
        let options = WebTransportOptions {
            server_certificate_hashes: vec![CertificateHash {
                algorithm: HashAlgorithm::Sha256,
                value: certificate_bytes,
            }],
            ..Default::default()
        };

        let endpoint = Endpoint {
            options: options.to_js(),
        };

        let connecting = endpoint.connect(url.as_str()).await.unwrap();

        let session = connecting.wait_connect().await.unwrap();

        self.session = Some(session);
    }

    pub async fn setup_bistream(
        &mut self,
        is_first: bool,
    ) -> Result<WebTransportSendStream, JsError> {
        let opening = match &self.session {
            Some(session) => session.open_bi().await.unwrap(),
            None => return Err(JsError::new("Session not active")),
        };

        let (send_stream, receive_stream) = opening.wait_bi().await.unwrap();

        if is_first {
            self.first_receive_stream = Some(receive_stream);
        } else {
            self.second_receive_stream = Some(receive_stream);
        }

        Ok(WebTransportSendStream {
            stream: send_stream,
        })
    }

    pub async fn start_bistreams(mut self) -> Result<(), JsError> {
        tokio::select! {
           _ = async {
                let mut stream = self.first_receive_stream.unwrap();
                loop {
                    let mut buffer = vec![0; 65536].into_boxed_slice();
                    let message_length: usize = stream.read(&mut buffer).await.unwrap().expect("Error");

                    let message = std::str::from_utf8(&buffer[..message_length]).unwrap();

                    let message = &JsString::from_str(
                        format!("Stream 1: {}", message).as_str(),
                    )
                        .unwrap();
                    console::log(&Array::of1(message));
                }
            } => {
                console::error(&Array::of1(
                    &JsString::from_str(
                        format!("Connection to First Stream lost").as_str(),
                    )
                    .unwrap(),
                ));
                }
           _ = async {
                let mut stream = self.second_receive_stream.unwrap();
                loop {
                    let mut buffer = vec![0; 65536].into_boxed_slice();
                    let message_length: usize = stream.read(&mut buffer).await.unwrap().expect("Error");

                    let message = std::str::from_utf8(&buffer[..message_length]).unwrap();

                    let message = &JsString::from_str(
                        format!("Stream 2: {}", message).as_str(),
                    )
                        .unwrap();
                    console::log(&Array::of1(message));
                }
            } => {
                console::error(&Array::of1(
                    &JsString::from_str(
                        format!("Connection to Second Stream lost").as_str(),
                    )
                    .unwrap(),
                ));
                }
        }
        Ok(())
    }
}

#[wasm_bindgen]
pub struct WebTransportReceiveStream {
    stream: RecvStream,
}

#[wasm_bindgen]
pub struct WebTransportSendStream {
    stream: SendStream,
}

#[wasm_bindgen]
impl WebTransportSendStream {
    pub async fn send_message(&mut self, message: String) -> Result<(), JsError> {
        self.stream.write(message.as_bytes()).await.unwrap();
        Ok(())
    }
}

#[wasm_bindgen]
pub struct WebTransportBistream {
    send_stream: WebTransportSendStream,
    receive_stream: WebTransportReceiveStream,
}

#[wasm_bindgen]
impl WebTransportBistream {
    pub fn get_receive_stream(self) -> WebTransportReceiveStream {
        self.receive_stream
    }

    pub fn get_send_stream(self) -> WebTransportSendStream {
        self.send_stream
    }
}
