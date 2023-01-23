use image_notify_bot::prelude::*;

/// Shutdown the bot (and save data)
#[poise::command(slash_command, owners_only)]
async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    ctx.say("Shutting down the bot...").await?;
    ctx.framework()
        .shard_manager()
        .lock()
        .await
        .shutdown_all()
        .await;
    Ok(())
}

#[tokio::main]
async fn main() {
    // load previously loaded datas
    let previous_data = match std::fs::File::open("image-notify-bot_data") {
        Ok(file) => serde_cbor::from_reader(std::io::BufReader::new(file))
            .expect("Failed to read previous data: data is possibly corrupted"),
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => WrittenData {
                monitored_channels: std::collections::BTreeMap::new(),
            },
            _ => panic!("Failed to read previous data"),
        },
    };

    let monitored_channels =
        std::sync::Arc::from(std::sync::Mutex::new(previous_data.monitored_channels));
    let monitored_channels_clone = monitored_channels.clone();

    let mut commands = Vec::new();

    commands.push(commands::config::config());
    commands.push(shutdown());

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    monitored_channels: monitored_channels_clone,
                })
            })
        });

    framework.run().await.unwrap();
    println!("{}", monitored_channels.lock().unwrap().len());
    let written_data = WrittenData {
        monitored_channels: monitored_channels.lock().unwrap().clone(),
    };
    std::fs::write(
        std::path::Path::new("image-notify-bot_data"),
        serde_cbor::to_vec(&written_data).unwrap(),
    )
    .unwrap();
}
