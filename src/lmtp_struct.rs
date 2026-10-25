use lambda_runtime::tracing;
use samotop::mail::SmtpClient;
use samotop::mail::{EmailAddress, Envelope};
use samotop_delivery::Transport;

pub(crate) struct LmtpClient {
    from: String,
    to: Vec<String>,
    raw_email: String,
}

impl LmtpClient {
    pub(crate) fn new(from: String, to: Vec<String>, raw_email: String) -> Self {
        let client = Self {
            from,
            to,
            raw_email,
        };
        tracing::event!(
            tracing::Level::DEBUG,
            "Creating LmtpClient with from: {}, to: {:?}",
            client.from,
            client.to
        );
        client
    }
    pub async fn send(&self) -> Result<(), Box<dyn std::error::Error>> {
        let sender = self.from.parse::<EmailAddress>().unwrap();
        let recipient_list: Vec<_> = self.to.iter().map(|addr| addr.parse().unwrap()).collect();
        let envelope = Envelope::new(Some(sender), recipient_list, String::from("client-61429"))
            .expect("bad envelope");
        let dovecot_addr = std::env::var("DOVECOT").expect("no DOVECOT address specified");
        let client = SmtpClient::new(&dovecot_addr).expect("send error");
        let c1 = client.lmtp(true);
        let conn = c1.connect();
        let if_result = conn.send(envelope, self.raw_email.as_bytes()).await;
        tracing::event!(tracing::Level::DEBUG, "LMTP send result: {:?}", if_result);
        match if_result {
            Ok(_) => {
                tracing::event!(tracing::Level::INFO, "LMTP send successful.");
                Ok(())
            }
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "LMTP send error: {}", e);
                Ok(())
            }
        }
    }
}
#[cfg(test)]
mod tests_lmtp {

    use super::*;
    use lambda_runtime::tracing::subscriber;

    use socket_server_mocker::Instruction::{
         ReceiveMessageWithMaxSize, SendMessage, StopExchange,SendMessageDependingOnLastReceivedMessage
    };
    use socket_server_mocker::ServerMocker;

  #[tokio::test]
    async fn test_send() {
        dotenvy::dotenv().ok();

        let _ = tracing::subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .try_init();
        if let Err(e) = subscriber::fmt()
            .with_max_level(tracing::Level::ERROR)
            .try_init()
        {
            eprintln!("{}", e);
        }

        let server = ServerMocker::tcp_with_port(2424).expect("bad server");
        std::env::set_var("DOVECOT", "127.0.0.1:2424");

        server
            .add_mock_instructions(vec![
                SendMessage(b"220 mail.example.com Dovecot ready.".to_vec()),
                ReceiveMessageWithMaxSize(10),
                SendMessage(b"250-Hello client\r\n250-ENHANCEDSTATUSCODES\r\n250 HELP\r\n".to_vec()),
                ReceiveMessageWithMaxSize(10),
                SendMessage(b"250 2.1.0 Ok\r\n".to_vec()),
                ReceiveMessageWithMaxSize(10),
                SendMessage(b"250 2.1.5 Ok\r\n".to_vec()),
                ReceiveMessageWithMaxSize(10),
                SendMessage(b"354 End data with <CR><LF>.<CR><LF>\r\n".to_vec()),
                ReceiveMessageWithMaxSize(10),
                SendMessage(b"250 2.0.0 <user@example.com> Delivered\r\n".to_vec()),
                ReceiveMessageWithMaxSize(10),
                SendMessage(b"221 2.0.0 Bye\r\n".to_vec()),
                StopExchange
            ])
            .expect("bad instructions");

      
        let client = LmtpClient::new(
            "from@example.com".to_string(),
            vec!["to@example.com".to_string()],
            r#"From: "Sender Name" <sender@example.com>
To: Rec <recipient@example.com>, Jack <jack@example.com>
Subject: Hello from Rust mail-parser
Message-ID: <1234@example.com>
Content-Type: text/plain; charset="utf-8"

This is the body of the email.
|"#
            .to_string(),
        );
        let result = client.send().await;
        print!("result {:?}", result);
        assert!(result.is_ok());
    }
}
