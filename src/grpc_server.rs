use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

pub mod services {
    tonic::include_proto!("services");
}

use services::payment_service_server::{PaymentService, PaymentServiceServer};
use services::transaction_service_server::{TransactionService, TransactionServiceServer};
use services::{PaymentRequest, PaymentResponse, TransactionRequest, TransactionResponse};

#[derive(Default)]
pub struct MyPaymentService {}

#[tonic::async_trait]
impl PaymentService for MyPaymentService {
    async fn process_payment(
        &self,
        request: Request<PaymentRequest>,
    ) -> Result<Response<PaymentResponse>, Status> {
        println!("Received payment request: {:?}", request);

        let response = PaymentResponse { success: true };

        Ok(Response::new(response))
    }
}

#[derive(Default)]
pub struct MyTransactionService {}

#[tonic::async_trait]
impl TransactionService for MyTransactionService {
    type GetTransactionHistoryStream = ReceiverStream<Result<TransactionResponse, Status>>;

    async fn get_transaction_history(
        &self,
        request: Request<TransactionRequest>,
    ) -> Result<Response<Self::GetTransactionHistoryStream>, Status> {
        println!("Received transaction history request: {:?}", request);

        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            for i in 1..=30 {
                let transaction = TransactionResponse {
                    transaction_id: format!("txn_{}", i),
                    status: "SUCCESS".to_string(),
                    amount: i as f64 * 10.0,
                    timestamp: format!("2026-01-{:02}", i),
                };

                if tx.send(Ok(transaction)).await.is_err() {
                    break;
                }

                if i % 10 == 0 {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;

    let payment_service = MyPaymentService::default();
    let transaction_service = MyTransactionService::default();

    Server::builder()
        .add_service(PaymentServiceServer::new(payment_service))
        .add_service(TransactionServiceServer::new(transaction_service))
        .serve(addr)
        .await?;

    Ok(())
}