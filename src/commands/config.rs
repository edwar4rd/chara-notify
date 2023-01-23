use crate::prelude::*;

/// Config related commands
#[poise::command(
    slash_command,
    subcommands("channel_add", "channel_remove", "channel_list",)
)]

pub async fn config(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Add a channel to be monitored by the bot
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    default_member_permissions = "MANAGE_CHANNELS",
    rename = "channel-add"
)]
pub async fn channel_add(
    ctx: Context<'_>,
    #[description = "Channel to be monitored"] channel: serenity::Channel,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    if {
        let mut monitored_channels = ctx.data().monitored_channels.lock().unwrap();

        if !monitored_channels.contains_key(&ctx.guild_id()) {
            monitored_channels.insert(ctx.guild_id().clone(), std::collections::BTreeSet::new());
        }

        let guild_channels = monitored_channels.get_mut(&ctx.guild_id()).unwrap();

        guild_channels.insert(channel.id())
    } {
        ctx.say("Success...").await?;
    } else {
        ctx.say("Channel is already monitored...").await?;
    }

    Ok(())
}

/// Remove a channel in the monitor list of the bot
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    default_member_permissions = "MANAGE_CHANNELS",
    rename = "channel-remove"
)]
pub async fn channel_remove(
    ctx: Context<'_>,
    #[description = "Channel to be monitored"] channel: serenity::Channel,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    if {
        let mut monitored_channels = ctx.data().monitored_channels.lock().unwrap();

        if monitored_channels.contains_key(&ctx.guild_id()) {
            let guild_channels = monitored_channels.get_mut(&ctx.guild_id()).unwrap();
            let result = guild_channels.remove(&channel.id());
            if result && guild_channels.is_empty() {
                drop(guild_channels);
                monitored_channels.remove(&ctx.guild_id());
            }

            result
        } else {
            true
        }
    } {
        ctx.say("Success...").await?;
    } else {
        ctx.say("Channel is already not monitored...").await?;
    }

    Ok(())
}

/// List all channel monitored by the bot in a guild
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS",
    default_member_permissions = "MANAGE_CHANNELS",
    rename = "channel-list"
)]
async fn channel_list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    if ctx.guild_id().is_none() {
        ctx.say("This command is not available outside guilds...")
            .await?;
        return Ok(());
    }

    let mut response = String::new();

    {
        let monitored_channels = ctx.data().monitored_channels.lock().unwrap();
        match monitored_channels.get(&ctx.guild_id()) {
            Some(channels) => {
                if channels.len() > 0 {
                    for channel in channels.iter() {
                        response.push_str(&format!("<#{}>\n", channel));
                    }
                } else {
                    response.push_str("There's no monitored channel in the server yet...");
                }
            }
            None => {
                response.push_str("There's no monitored channel in the server yet...");
            }
        };
    }

    ctx.say(response).await?;
    Ok(())
}
