use lambda::handler_fn;

use hearts::deliver;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send + 'static>> {
    env_logger::init();
    lambda::run(handler_fn(deliver)).await?;
    Ok(())
}
