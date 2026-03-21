# Tonic Server Reference

> tonic 0.14 | Source: https://docs.rs/tonic/0.14

## Service Implementation

```rust
use tonic::{Request, Response, Status};

pub mod pb {
    tonic::include_proto!("mypackage");
}

use pb::my_service_server::{MyService, MyServiceServer};

pub struct MyServiceImpl {
    db: DatabaseConnection,
}

#[tonic::async_trait]
impl MyService for MyServiceImpl {
    async fn create_item(
        &self,
        request: Request<CreateItemRequest>,
    ) -> Result<Response<CreateItemResponse>, Status> {
        let req = request.into_inner();

        // Validate
        if req.name.is_empty() {
            return Err(Status::invalid_argument("name cannot be empty"));
        }

        // Business logic
        let item = self.db.create_item(&req.name).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateItemResponse {
            id: item.id,
        }))
    }

    async fn get_item(
        &self,
        request: Request<GetItemRequest>,
    ) -> Result<Response<GetItemResponse>, Status> {
        let req = request.into_inner();
        let item = self.db.find_item(req.id).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("item not found"))?;

        Ok(Response::new(GetItemResponse {
            item: Some(item.into()),
        }))
    }
}
```

## Server Setup

```rust
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = MyServiceImpl::new(db);

    Server::builder()
        .add_service(MyServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
```

## Multiple Services

```rust
Server::builder()
    .add_service(UserServiceServer::new(user_svc))
    .add_service(AuthServiceServer::new(auth_svc))
    .serve(addr)
    .await?;
```

## Interceptor

```rust
use tonic::{Request, Status, service::Interceptor};

fn auth_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    let token = req.metadata()
        .get("authorization")
        .ok_or_else(|| Status::unauthenticated("missing auth token"))?;
    // Validate token...
    Ok(req)
}

// Apply to a service
let service = MyServiceServer::with_interceptor(
    service_impl,
    auth_interceptor,
);
```

## Request Metadata

```rust
// Server: reading metadata from request
async fn handler(&self, request: Request<MyReq>) -> Result<Response<MyResp>, Status> {
    let metadata = request.metadata();
    let user_id = metadata.get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Status::unauthenticated("missing user id"))?;

    let inner = request.into_inner();
    // ...
}
```

## Response Metadata

```rust
async fn handler(&self, request: Request<MyReq>) -> Result<Response<MyResp>, Status> {
    let mut response = Response::new(MyResp { ... });
    response.metadata_mut().insert(
        "x-request-id",
        "abc-123".parse().unwrap(),
    );
    Ok(response)
}
```

## Server-Side Streaming

```rust
use tokio_stream::wrappers::ReceiverStream;

#[tonic::async_trait]
impl MyService for MyServiceImpl {
    type ListItemsStream = ReceiverStream<Result<Item, Status>>;

    async fn list_items(
        &self,
        request: Request<ListItemsRequest>,
    ) -> Result<Response<Self::ListItemsStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(128);

        tokio::spawn(async move {
            for item in items {
                tx.send(Ok(item)).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
```

## Status Codes Reference

| Code | Constructor | HTTP equiv |
|------|------------|------------|
| OK | (implicit on success) | 200 |
| CANCELLED | `Status::cancelled(msg)` | 499 |
| INVALID_ARGUMENT | `Status::invalid_argument(msg)` | 400 |
| NOT_FOUND | `Status::not_found(msg)` | 404 |
| ALREADY_EXISTS | `Status::already_exists(msg)` | 409 |
| PERMISSION_DENIED | `Status::permission_denied(msg)` | 403 |
| UNAUTHENTICATED | `Status::unauthenticated(msg)` | 401 |
| RESOURCE_EXHAUSTED | `Status::resource_exhausted(msg)` | 429 |
| INTERNAL | `Status::internal(msg)` | 500 |
| UNAVAILABLE | `Status::unavailable(msg)` | 503 |
| UNIMPLEMENTED | `Status::unimplemented(msg)` | 501 |
