# Tonic Client Reference

> tonic 0.14 | Source: https://docs.rs/tonic/0.14

## Basic Client Connection

```rust
use pb::my_service_client::MyServiceClient;

let mut client = MyServiceClient::connect("http://[::1]:50051").await?;
```

## Making Requests

```rust
// Unary RPC
let request = tonic::Request::new(GetItemRequest { id: 42 });
let response = client.get_item(request).await?;
let item = response.into_inner();

// Shorthand (auto-wraps in Request)
let response = client.get_item(GetItemRequest { id: 42 }).await?;
```

## Error Handling

```rust
match client.get_item(request).await {
    Ok(response) => {
        let item = response.into_inner();
        // use item
    }
    Err(status) => {
        match status.code() {
            tonic::Code::NotFound => { /* handle not found */ }
            tonic::Code::InvalidArgument => { /* handle bad input */ }
            tonic::Code::Internal => { /* handle server error */ }
            _ => { /* handle other errors */ }
        }
        let message = status.message();
    }
}
```

## Request Metadata (Headers)

```rust
let mut request = tonic::Request::new(GetItemRequest { id: 42 });
request.metadata_mut().insert(
    "authorization",
    format!("Bearer {}", token).parse().unwrap(),
);

let response = client.get_item(request).await?;
```

## Channel Configuration

```rust
use tonic::transport::{Channel, Endpoint};
use std::time::Duration;

let channel = Channel::from_static("http://[::1]:50051")
    .connect_timeout(Duration::from_secs(5))
    .timeout(Duration::from_secs(10))
    .connect()
    .await?;

let client = MyServiceClient::new(channel);
```

## Client Interceptor

```rust
use tonic::service::Interceptor;

fn add_auth(mut req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
    req.metadata_mut().insert(
        "authorization",
        "Bearer my-token".parse().unwrap(),
    );
    Ok(req)
}

let channel = Channel::from_static("http://[::1]:50051").connect().await?;
let client = MyServiceClient::with_interceptor(channel, add_auth);
```

## Client-Side Streaming

```rust
use tokio_stream::iter;

let items = vec![
    CreateItemRequest { name: "A".into() },
    CreateItemRequest { name: "B".into() },
];

let request = tonic::Request::new(iter(items));
let response = client.batch_create(request).await?;
```

## Server-Side Streaming (Client Consuming)

```rust
let request = tonic::Request::new(ListItemsRequest {});
let mut stream = client.list_items(request).await?.into_inner();

while let Some(item) = stream.message().await? {
    println!("received: {:?}", item);
}
```

## Bidirectional Streaming

```rust
let (tx, rx) = tokio::sync::mpsc::channel(128);

// Send requests in background
tokio::spawn(async move {
    for i in 0..10 {
        tx.send(MyRequest { value: i }).await.unwrap();
    }
});

let request = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(rx));
let mut response_stream = client.bidi_rpc(request).await?.into_inner();

while let Some(response) = response_stream.message().await? {
    println!("got: {:?}", response);
}
```

## Reusing Clients

```rust
// Client is Clone — share across tasks
let client = MyServiceClient::connect("http://[::1]:50051").await?;

let client_clone = client.clone();
tokio::spawn(async move {
    client_clone.get_item(request).await
});
```
