pub mod services {
    tonic::include_proto!("services");
}

use services::payment_service_client::PaymentServiceClient;
use services::transaction_service_client::TransactionServiceClient;
use services::{PaymentRequest, TransactionRequest};

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

    Ok(())
}