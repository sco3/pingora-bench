use pingora::connectors::http::v1::client::Client as HttpClient;
use pingora::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Define the Peer (the server you want to talk to)
    // HttpPeer(Addr, use_tls, SNI)
    let peer = HttpPeer::new(("127.0.0.1", 8888), false, "".to_string());

    // 2. Create a session (this establishes the connection)
    // Pingora handles connection pooling internally if you reuse the connector
    let mut session = Session::new_http(Box::new(peer));
    
    // 3. Establish the connection
    session.connect().await.map_err(|e| {
        eprintln!("Failed to connect: {}", e);
        e
    })?;

    // 4. Build your Request Header
    let mut req = RequestHeader::build("GET", b"/sse", None).unwrap();
    req.insert_header("Accept", "text/event-stream").unwrap();
    req.insert_header("Host", "localhost").unwrap();

    // 5. Send the request
    session.write_request_header(Box::new(req)).await?;

    // 6. Read the response
    let (mut resp, _body_reader) = session.read_response().await?;
    println!("Response Status: {}", resp.status);

    // If it's an SSE stream, you can loop to read the body chunks
    loop {
        if let Some(chunk) = session.read_response_body().await? {
            println!("Received chunk: {:?}", String::from_utf8_lossy(&chunk));
        } else {
            break;
        }
    }

    Ok(())
}