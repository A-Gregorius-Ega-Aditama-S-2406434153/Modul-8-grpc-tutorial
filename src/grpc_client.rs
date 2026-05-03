use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;

pub mod services {
    tonic::include_proto!("services");
}

use services::chat_service_client::ChatServiceClient;
use services::payment_service_client::PaymentServiceClient;
use services::transaction_service_client::TransactionServiceClient;
use services::{ChatMessage, PaymentRequest, TransactionRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut payment_client = PaymentServiceClient::connect("http://127.0.0.1:50051").await?;

    let request = tonic::Request::new(PaymentRequest {
        user_id: "user_123".to_string(),
        amount: 100.0,
    });

    let response = payment_client.process_payment(request).await?;

    println!("Payment Response: {:?}", response.into_inner());

    let mut transaction_client =
        TransactionServiceClient::connect("http://127.0.0.1:50051").await?;

    let request = tonic::Request::new(TransactionRequest {
        user_id: "user_123".to_string(),
    });

    let mut stream = transaction_client
        .get_transaction_history(request)
        .await?
        .into_inner();

    while let Some(transaction) = stream.message().await? {
        println!("Transaction: {:?}", transaction);
    }

    let channel = Channel::from_static("http://127.0.0.1:50051")
        .connect()
        .await?;

    let mut chat_client = ChatServiceClient::new(channel);

    let (tx, rx) = mpsc::channel(10);

    tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin).lines();

        println!("Type a chat message and press Enter:");

        while let Ok(Some(line)) = reader.next_line().await {
            let message = ChatMessage {
                user_id: "user_123".to_string(),
                message: line,
            };

            if tx.send(message).await.is_err() {
                eprintln!("Failed to send message");
                break;
            }
        }
    });

    let request = tonic::Request::new(ReceiverStream::new(rx));

    let mut response_stream = chat_client.chat(request).await?.into_inner();

    while let Some(response) = response_stream.message().await? {
        println!("Server: {:?}", response);
    }

    Ok(())
}