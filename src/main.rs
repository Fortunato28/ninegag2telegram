use ninegag2telegram as n2t;

#[tokio::main]
async fn main() {

    //n2t::handle_message("test");
    n2t::run().await;
}
