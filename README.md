# Name : Gregorius Ega Aditama Sudjali
# NPM : 2406434153

## Reflection

### 1. What are the key differences between unary, server streaming, and bi-directional streaming RPC methods, 
and in what scenarios would each be most suitable?

Unary RPC, server streaming RPC, and bidirectional streaming RPC differ mainly in how many messages
flow between the client and server. Unary RPC is the simplest model: the client sends one request
and receives one response. It is suitable for operations such as `ProcessPayment`, where a payment
request only needs one final success or failure response.

Server streaming RPC starts with one client request, but the server returns many responses over
time. It is suitable for transaction history because one `TransactionRequest` can produce many
`TransactionResponse` records, and the server can send them gradually instead of building one large
response.

Bidirectional streaming RPC allows both sides to send streams of messages independently. It is
suitable for chat because the client and server need ongoing, real-time two-way communication.

### 2. What are the potential security considerations involved in implementing a gRPC service in Rust,
particularly regarding authentication, authorization, and data encryption?

A Rust gRPC service needs security at several layers. For authentication, the server should verify
who the client is, for example by validating tokens or client certificates passed through gRPC
metadata. For authorization, the service should check whether the authenticated user is allowed to
perform the requested action, such as preventing one user from requesting another user's transaction
history.

For data encryption, production gRPC should use TLS because gRPC runs over HTTP/2 and payment or
chat data should not be sent as plain text. The service should also validate input, avoid leaking
internal errors through `tonic::Status`, configure timeouts and message size limits, and protect
sensitive logs because the current tutorial code prints full requests.

### 3. What are the potential challenges or issues that may arise when handling bidirectional streaming in Rust gRPC,
especially in scenarios like chat applications?

Bidirectional streaming in Rust gRPC can be challenging because both sending and receiving happen
asynchronously at the same time. In a chat application, the server must handle long-lived
connections, clients that disconnect suddenly, slow receivers, and many concurrent users.

The code also has to manage channels carefully: if the `mpsc` buffer is too small, messages may
wait; if it is too large, memory use can grow under load. Error handling is also important because
simply converting stream errors into `None` can hide the reason a connection failed. For a real chat
system, the service would also need message ordering rules, user presence, broadcast or room
management, backpressure handling, and cleanup when a stream ends.

### 4. What are the advantages and disadvantages of using the `tokio_stream::wrappers::ReceiverStream` for streaming 
responses in Rust gRPC services?

`tokio_stream::wrappers::ReceiverStream` is useful because it converts a Tokio `mpsc` receiver into
a stream that tonic can return as a gRPC streaming response. This matches the tutorial
implementation well: a spawned async task can produce transaction or chat messages, send them
through a channel, and let tonic stream them to the client. It also helps separate message
production from network delivery.

The disadvantage is that channel buffering and task lifetimes must be managed manually. If the
receiver is dropped, sends fail; if the sender keeps producing faster than the client consumes, the
buffer can become a bottleneck. It can also make error propagation less direct because errors must
be sent through the stream as `Result<T, Status>`.

### 5. In what ways could the Rust gRPC code be structured to facilitate code reuse and modularity,
promoting maintainability and extensibility over time?

The Rust gRPC code could be structured more modularly by separating generated protobuf bindings,
service implementations, domain logic, and executable entry points. For example, the `services`
module from `tonic::include_proto!` could live in a shared module, while `payment`, `transaction`,
and `chat` implementations could be split into separate files.

Business logic such as validating a payment or fetching transactions should be placed outside the
tonic RPC method so it can be reused and tested without starting a gRPC server. Common configuration
such as server address, channel size, TLS setup, and error conversion could also be centralized.
This would make the code easier to extend when adding new RPC methods or replacing the simulated
data with a database.

### 6. In the MyPaymentService implementation, what additional steps might be necessary
to handle more complex payment processing logic?

The current `MyPaymentService` always returns `PaymentResponse { success: true }`, so more complex
payment processing would require additional steps. The service should validate the request fields,
reject invalid amounts, authenticate the user, check authorization, and make the payment operation
idempotent so duplicate requests do not double-charge a user.

It would also need integration with a payment provider or internal ledger, transaction persistence,
failure handling, retries with care, audit logs, and clear status codes for declined, pending, or
failed payments. For production use, the protobuf schema might also need more fields such as
currency, payment method, transaction ID, error message, and timestamp.

### 7. What impact does the adoption of gRPC as a communication protocol have on the overall architecture and
design of distributed systems, particularly in terms of interoperability with other technologies and platforms?

Adopting gRPC affects distributed system architecture by making service contracts explicit through
`.proto` files and generated client/server code. This improves interoperability because services
written in different languages can communicate using the same protobuf schema, which matches the
module's point that gRPC is popular for service-to-service communication and multi-language systems.

It also encourages an RPC-oriented design where remote calls feel like function calls. However,
teams must manage schema evolution carefully, generate code for each platform, and consider whether
clients such as browsers or external third parties are comfortable using gRPC instead of JSON-based
REST.

### 8. What are the advantages and disadvantages of using HTTP/2, the underlying protocol for gRPC, 
compared to HTTP/1.1 or HTTP/1.1 with WebSocket for REST APIs?

HTTP/2 gives gRPC several advantages over HTTP/1.1 and many REST-over-WebSocket designs.
Multiplexing allows many requests and responses to share one connection, reducing the head-of-line
blocking and repeated connection overhead common in HTTP/1.1. Binary framing and HPACK header
compression reduce overhead, which helps performance and latency. HTTP/2 also supports streaming
naturally, which is why gRPC can implement server streaming and bidirectional streaming cleanly.

The disadvantages are that HTTP/2 and gRPC can be harder to inspect manually than text-based
HTTP/1.1 JSON APIs, require more specific tooling, and may face compatibility issues with some
proxies, browsers, or legacy infrastructure. WebSocket can support real-time traffic, but it does
not provide gRPC's built-in schema, code generation, and RPC method structure.

### 9. How does the request-response model of REST APIs contrast with the bidirectional
streaming capabilities of gRPC in terms of real-time communication and responsiveness?

REST usually follows a request-response model where the client asks for a resource or action and the
server returns one response. This is easy to understand and works well for normal CRUD APIs, but it
is less natural for real-time interaction because the client often has to poll repeatedly or use a
separate WebSocket connection.

gRPC streaming allows the connection to stay open and lets messages arrive as soon as they are
available. In server streaming, the client receives continuous updates after one request. In
bidirectional streaming, both sides can send messages independently, which makes chat, live
analytics, progress updates, and collaborative features more responsive.

### 10. What are the implications of the schema-based approach of gRPC, using Protocol Buffers, 
compared to the more flexible, schema-less nature of JSON in REST API payloads?

gRPC's schema-based approach with Protocol Buffers makes the API contract explicit. The `.proto`
file defines services, methods, and message fields, and tooling generates strongly typed client and
server code. This improves consistency, reduces ambiguity, and produces compact binary payloads that
are efficient on the network.

The tradeoff is that schema changes require more planning and code regeneration, and protobuf
messages are not as easy for humans to read directly as JSON. JSON in REST is more flexible and
convenient for quick integrations because clients can send varied payload shapes, but that
flexibility can lead to weaker contracts, runtime validation errors, larger payloads, and
inconsistent API behavior if the structure is not documented and enforced carefully.
