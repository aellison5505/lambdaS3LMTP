use mail_parser::{MessageParser};
use std::rc::Rc;
use lambda_runtime::{tracing};

#[derive(Debug)]
pub(crate) struct ProcessEmail {
    pub email_data: String,
}

impl ProcessEmail {
    pub fn new(email_data: String) -> Self {
        Self { email_data }
    }

    pub async  fn process(&self) -> (String, Vec<String>) {
        tracing::event!(tracing::Level::DEBUG, "Processing email: {}", self.email_data);
        let message: Rc<mail_parser::Message<'_>> = Rc::new(
            MessageParser::new()
                .parse(&self.email_data)
                .expect("Failed to parse email"),
        );
        let from: String = Self::get_from(Rc::clone(&message));
        tracing::event!(tracing::Level::INFO, "From: {:?}", from);
        let to = Self::get_to(Rc::clone(&message));
        tracing::event!(tracing::Level::INFO, "To: {:?}", to.join(", "));
        return (from, to);
    }

    fn get_from(message: Rc<mail_parser::Message<'_>>) -> String {
        let from = message.from().and_then(|from| from.first());
        from.and_then(|addr| addr.address()).map(|addr| addr.trim().to_string()).unwrap_or_default()
    }

    fn get_to(message: Rc<mail_parser::Message<'_>>) -> Vec<String> {
        let tos = message.to().unwrap();
        let result:Vec<String> = tos.mailboxes().filter_map(|mailbox| mailbox.address().map(|addr| addr.to_string())).collect();
       result
      
    }
}

#[cfg(test)]
mod tests_process_email_struct {
    use super::*;

    #[tokio::test]
    async fn test_process_email() {
        let email_data = r#"From: "Sender Name" <sender@example.com>
To: Rec <recipient@example.com>, Jack <jack@example.com>
Subject: Hello from Rust mail-parser
Message-ID: <1234@example.com>
Content-Type: text/plain; charset="utf-8"

This is the body of the email.
"#;

        let process_email = ProcessEmail::new(email_data.to_string());
        let (from, to) = process_email.process().await;
        
        assert_eq!(from, "sender@example.com");
        assert_eq!(
            to,
            vec!["recipient@example.com".to_string(), "jack@example.com".to_string()]
        );
    }
}
