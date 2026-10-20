use samotop::mail::SmtpClient;
use samotop::mail::{EmailAddress, Envelope};
use samotop_delivery::Transport;

pub (crate) struct LmtpClient {
    from: String,
    to: Vec<String>,
    raw_email: String
}

impl LmtpClient {
    pub(crate) fn new(from: String, to: Vec<String>, raw_email: String) -> Self {
        let client = Self { from, to, raw_email };
        print!("Creating LmtpClient with from: {}, to: {:?}\n", client.from, client.to);
        client
    }
    pub async fn send(&self) -> Result<(), Box<dyn std::error::Error>> {
        let sender = self.from.parse::<EmailAddress>().unwrap();
        let recipient_list: Vec<_> = self.to.iter().map(|addr| addr.parse().unwrap()).collect(); 
        let envelope = Envelope::new(
            Some(sender),
            recipient_list,
            String::from("client-61429")
        ).expect("bad envelope");
        let dovecot_addr = std::env::var("DOVECOT").expect("no DOVECOT address specified");
       let client = SmtpClient::new(&dovecot_addr).expect("send error");
       let c1 = client.lmtp(true);
       let conn = c1.connect();
       let if_result = conn.send(envelope, self.raw_email.as_bytes()).await;
       match if_result {
           Ok(_) => Ok(()),
           Err(e) => { print!("LMTP send error: {}", e);
               Err(Box::new(e))
           }
       }    
    }
}