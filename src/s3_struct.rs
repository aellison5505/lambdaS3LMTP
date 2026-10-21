use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use lambda_runtime::tracing;

#[derive(serde::Deserialize, Debug)]
pub(crate) struct ProcessFile {
    pub s3file_name: String,
    pub s3bucket_name: String,
}

impl ProcessFile {
    pub fn new(s3file_name: String, s3bucket_name: String) -> Self {
        Self {
            s3file_name,
            s3bucket_name,
        }
    }

    pub async fn read_s3_file(&self) -> Option<String> {
        tracing::event!(
            tracing::Level::INFO,
            "S3 File: {:?} S3 Bucket: {:?}",
            self.s3file_name,
            self.s3bucket_name
        );
        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        let client_s3 =  aws_sdk_s3::config::Builder::from(&config)
        .force_path_style(true) // <-- CRITICAL LINE FOR CUSTOM ENDPOINTS
        .build();

        let client = aws_sdk_s3::Client::from_conf(client_s3);
        let resp = client
            .get_object()
            .bucket(&self.s3bucket_name)
            .key(&self.s3file_name)
            .send()
            .await;

        match resp {
            Ok(output) => {
                let body_bytes = output
                    .body
                    .collect()
                    .await
                    .expect("Failed to collect body bytes")
                    .into_bytes();
                match str::from_utf8(&body_bytes) {
                    Ok(body_str) => {
                        let email_data = body_str.to_string();
                        tracing::event!(
                            tracing::Level::DEBUG,
                            "Successfully read S3 file: {:?}",
                            email_data
                        );
                        return Some(email_data);
                    }
                    Err(e) => {
                        tracing::event!(
                            tracing::Level::ERROR,
                            "Failed to parse body as UTF-8: {:?}",
                            e
                        );
                        return None;
                    }
                }
            }
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "Failed to read S3 file: {:?}", e);
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests_s3 {
    use std::env;

use super::*;
    use httpmock::prelude::*;
    use httpmock::MockServer;
    use lambda_runtime::tracing::subscriber;

    #[tokio::test]
    async fn test_read_s3_file() {
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
        let s3bucket_name = std::env::var("BUCKET").expect("BUCKET not set in .env");
        let s3file = std::env::var("FILE").expect("FILE not set in .env");
        
        
       let server = MockServer::start();
        println!("Mock server running at: {}", server.port());
       env::set_var("AWS_ENDPOINT_URL_S3", format!("http://127.0.0.1:{}", server.port()));

        let _mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/{}/{}", s3bucket_name, s3file).as_str());
            then.status(200).body( r#"From: "Sender Name" <sender@example.com>
To: Rec <recipient@example.com>, Jack <jack@example.com>
Subject: Hello from Rust mail-parser
Message-ID: <1234@example.com>
Content-Type: text/plain; charset="utf-8"

This is the body of the email.
"#);
        });

        let test_file = ProcessFile::new(s3file, s3bucket_name).read_s3_file().await;
        assert_eq!(test_file.is_none(), false);
    }
}
