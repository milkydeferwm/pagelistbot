//! Main entry of Page List Bot

#[tokio::main]
async fn main() {
    let _host = host::Host::try_new("", "", "", "", false).await;
}
