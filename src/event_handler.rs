
use crate::process_email_struct::ProcessEmail;
use crate::s3_struct::ProcessFile;
use aws_lambda_events::event::s3::S3Event;
use lambda_runtime::{tracing, Error, LambdaEvent};

pub(crate) async fn function_handler(event: LambdaEvent<S3Event>) -> Result<(), Error> {
   
    // Extract some useful information from the request
    let payload = event.payload;
    let mut i = 0;
    for record in &payload.records {
        i += 1;
        tracing::event!(tracing::Level::INFO, "Processing record number: {}", i);
        let Some(object_key) = record.s3.object.key.as_ref() else {
            continue;
        };
        let Some(bucket) = record.s3.bucket.name.as_ref() else {
            continue;
        };
        tracing::event!(
            tracing::Level::INFO,
            "Record: {:?} {:?}",
            object_key,
            bucket
        );
        let email = ProcessFile::new(object_key.to_string(), bucket.to_string())
            .read_s3_file()
            .await
            .expect("email read failed");
        let (from, to) = ProcessEmail::new(email.clone()).process().await;
        let lmtp_client = crate::lmtp_struct::LmtpClient::new(from, to, email.clone());
        lmtp_client.send().await.unwrap();

    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_runtime::{tracing, Context, LambdaEvent};
    use std::fs::File;
    use std::io;

    fn init_tracing() {
        let _ = tracing::subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .try_init();
    }

    #[tokio::test]
    async fn test_event_handler() {
        init_tracing();
        dotenvy::dotenv().ok();
        let jfile = File::open("s3_event.json").expect("file not found");
        let reader = io::BufReader::new(jfile);
        let event: S3Event = serde_json::from_reader(reader).expect("failed to parse JSON");
        let context = Context::default();
        let lambda_event = LambdaEvent::new(event, context);
        let response = function_handler(lambda_event).await.unwrap();
        assert_eq!((), response);
    }
}
