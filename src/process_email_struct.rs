use mail_parser::MessageParser;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct ProcessEmail {
    pub email_data: String,
}

impl ProcessEmail {
    pub fn new(email_data: String) -> Self {
        Self { email_data }
    }

    pub fn process(&self) -> (String, Vec<String>) {
        println!("Processing email: {}", self.email_data);
        let message: Arc<mail_parser::Message<'_>> = Arc::new(
            MessageParser::new()
                .parse(&self.email_data)
                .expect("Failed to parse email"),
        );
        let from = Self::get_from(Arc::clone(&message));
        println!("From: {:?}", from);
        let to = Self::get_to(Arc::clone(&message));
        println!("To: {:?}", to.join(", "));
        return (from, to);
    }

    fn get_from(message: Arc<mail_parser::Message<'_>>) -> String {
        let froms = message.from().unwrap().iter();
        let mut result = Vec::new();

        for addr in froms {
            let mut full_address = String::new();
            if let Some(name) = &addr.name {
                full_address.push_str(name);
                full_address.push(' ');
            }
            if let Some(address) = &addr.address {
                full_address.push_str(&format!("<{}>", address));
            }

            result.push(full_address);
        }
        result.join(", ").to_string()
    }

    fn get_to(message: Arc<mail_parser::Message<'_>>) -> Vec<String> {
        let tos = message.to().unwrap().iter();
        let mut result = Vec::new();

        for addr in tos {
            let mut full_address = String::new();
            if let Some(name) = &addr.name {
                full_address.push_str(name);
                full_address.push(' ');
            }
            if let Some(address) = &addr.address {
                full_address.push_str(&format!("<{}>", address));
            }

            result.push(full_address);
        }
        result
    }
}

#[cfg(test)]
mod tests_process_email_struct {
    use super::*;

    #[test]
    fn test_process_email() {
        let email_data = r#"From: "Sender Name" <sender@example.com>
To: Rec <recipient@example.com>, Jack <jack@example.com>
Subject: Hello from Rust mail-parser
Message-ID: <1234@example.com>
Content-Type: text/plain; charset="utf-8"

This is the body of the email.
"#;

        let process_email = ProcessEmail::new(email_data.to_string());
        let (from, to) = process_email.process();
        
        assert_eq!(from, "Sender Name <sender@example.com>");
        assert_eq!(
            to.join(", "),
            "Rec <recipient@example.com>, Jack <jack@example.com>"
        );
    }
}
