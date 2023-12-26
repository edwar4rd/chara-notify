use image_notify_bot::prelude::*;

const PARALLEL_EVALUATER_COUNT: usize = 3;

#[tokio::main]
async fn main() {
    // load previously written datas

    let previous_data = match std::fs::File::open("image-notify-bot_data") {
        Ok(file) => serde_cbor::from_reader(std::io::BufReader::new(file))
            .expect("Failed to read previous data: data is possibly corrupted"),
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => WrittenData {
                evaluation_caches: dashmap::DashMap::new(),
                monitored_channels: std::collections::BTreeMap::new(),
            },
            _ => panic!("Failed to read previous data"),
        },
    };

    let monitored_channels =
        std::sync::Arc::from(std::sync::Mutex::new(previous_data.monitored_channels));

    let evaluation_caches = std::sync::Arc::from(previous_data.evaluation_caches);

    let data = {
        let monitored_channels_clone = monitored_channels.clone();
        let evaluater = MyEvaluater::new().await.unwrap();
        let evaluater = std::sync::Arc::new(tokio::sync::Mutex::new(evaluater));
        // let evaluation_caches_clone = evaluation_caches.clone();
        // let evaluation_semaphore = tokio::sync::Semaphore::new(PARALLEL_EVALUATER_COUNT);
        Data {
            monitored_channels: monitored_channels_clone,
            evaluater: evaluater
        }
    };

    let commands = {
        let mut commands = Vec::new();

        commands.push(commands::config::config());
        commands.push(commands::help());
        commands.push(commands::shutdown());

        commands
    };

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            event_handler: |ctx, event, _framework, data| {
                Box::pin(handler::handler(ctx, event, data))
            },
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        });

    framework.run().await.unwrap();
    println!("{}", monitored_channels.lock().unwrap().len());
    let writing_data = WritingData {
        monitored_channels: &(*monitored_channels.lock().unwrap()),
        evaluation_caches: &evaluation_caches,
    };
    std::fs::write(
        std::path::Path::new("image-notify-bot_data"),
        serde_cbor::to_vec(&writing_data).unwrap(),
    )
    .unwrap();
}
