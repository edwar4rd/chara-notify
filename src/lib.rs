pub mod prelude {
    pub use crate::commands;
    pub use poise::serenity_prelude as serenity;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct WrittenData {
        pub monitored_channels: std::collections::BTreeMap<
            Option<serenity::GuildId>,
            std::collections::BTreeSet<serenity::ChannelId>,
        >,
    }
    pub struct Data {
        pub monitored_channels: std::sync::Arc<
            std::sync::Mutex<
                std::collections::BTreeMap<
                    Option<serenity::GuildId>,
                    std::collections::BTreeSet<serenity::ChannelId>,
                >,
            >,
        >,
    } // User data, which is stored and accessible in all command invocations
    pub type Error = Box<dyn std::error::Error + Send + Sync>;
    pub type Context<'a> = poise::Context<'a, Data, Error>;
}

pub mod commands;
