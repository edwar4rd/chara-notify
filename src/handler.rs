use crate::prelude::*;

/// test handler to help debugging
pub async fn print_everything(
    _: &serenity::Context,
    event: &poise::Event<'_>,
    _data: &Data,
) -> Result<(), Error> {
    println!("{:#?}", event);
    Ok(())
}

pub async fn handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    data: &Data,
) -> Result<(), Error> {
    println!("Handler invoked! {}", event.name());
    match event {
        poise::Event::Message { new_message } => {
            {
                let monitored_channels = data.monitored_channels.lock().unwrap();
                if !monitored_channels.contains_key(&new_message.guild_id) {
                    return Ok(());
                }
                let guild_channels = monitored_channels.get(&new_message.guild_id).unwrap();
                if !guild_channels.contains(&new_message.channel_id) {
                    return Ok(());
                }
            }
            if image::message_check_image(new_message) {
                new_message
                    .react(
                        &ctx,
                        serenity::ReactionType::Custom {
                            animated: false,
                            id: serenity::EmojiId(1066602611303780393),
                            name: Some("botmag".to_string()),
                        },
                    )
                    .await?;
            }
        }
        poise::Event::MessageUpdate {
            old_if_available: _,
            new: _,
            event,
        } => {
            {
                let monitored_channels = data.monitored_channels.lock().unwrap();
                if !monitored_channels.contains_key(&event.guild_id) {
                    return Ok(());
                }
                let guild_channels = monitored_channels.get(&event.guild_id).unwrap();
                if !guild_channels.contains(&event.channel_id) {
                    return Ok(());
                }
            }
            if image::update_check_image(event) {
                event
                    .channel_id
                    .message(ctx, event.id)
                    .await
                    .unwrap()
                    .react(
                        &ctx,
                        serenity::ReactionType::Custom {
                            animated: false,
                            id: serenity::EmojiId(1066602611303780393),
                            name: Some("botmag".to_string()),
                        },
                    )
                    .await?;
            }
        }
        poise::Event::ReactionAdd { add_reaction } => match &add_reaction.emoji {
            serenity::ReactionType::Custom {
                animated: _,
                id,
                name: _,
            } => {
                // ignore other reactions
                if id.0 != 1066602611303780393 {
                    return Ok(());
                }

                // check if channel is being monitored by the bot
                {
                    let monitored_channels = data.monitored_channels.lock().unwrap();
                    if !monitored_channels.contains_key(&add_reaction.guild_id) {
                        return Ok(());
                    }
                    let guild_channels = monitored_channels.get(&add_reaction.guild_id).unwrap();
                    if !guild_channels.contains(&add_reaction.channel_id) {
                        return Ok(());
                    }
                }

                // ignore bot user
                if add_reaction.user_id.is_none()
                    || add_reaction.user_id.unwrap().to_user(ctx).await?.bot
                {
                    return Ok(());
                }

                let message = add_reaction.message(ctx).await.unwrap();
                let image_list = image::message_list_images(&message);

                for image_url in image_list {
                    let response = {
                        let cache_exists = { data.evaluation_caches.contains_key(image_url) };
                        if cache_exists {
                            let caches = &data.evaluation_caches;
                            let result = caches.get(image_url).unwrap();
                            let mut response = String::new();
                            let mut result = result.iter().collect::<Vec<(&u32, &f32)>>();
                            response.push_str("```\n");
                            result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                            for tag in result {
                                let new = format!("{:04}: {:12}\n ", tag.0, tag.1);
                                if response.len() + new.len() > 1995 {
                                    break;
                                } else {
                                    response.push_str(&new);
                                }
                            }
                            response.push_str("```\n");
                            response
                        } else {
                            match image::evaluate_image(image_url).await {
                                Ok(result_map) => {
                                    let mut response = String::new();
                                    let mut result =
                                        result_map.iter().collect::<Vec<(&u32, &f32)>>();
                                    response.push_str("```\n");
                                    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                                    for tag in result {
                                        let new = format!("{:04}: {:12}\n", tag.0, tag.1);
                                        if response.len() + new.len() > 1995 {
                                            break;
                                        } else {
                                            response.push_str(&new);
                                        }
                                    }
                                    {
                                        data.evaluation_caches
                                            .insert(image_url.to_string(), result_map);
                                    }
                                    response.push_str("```\n");
                                    response
                                }
                                Err(err_no) => match err_no {
                                    -5 => "Failed to start the evaluation subprocess",
                                    -1 => "Failed to get response from proxy_url",
                                    -2 => "Failed to retrieve file from proxy_url",
                                    -3 => "Failed to open file as image",
                                    _ => unreachable!(),
                                }
                                .to_string(),
                            }
                        }
                    };
                    message.reply(ctx, &format!("{}", response)).await?;
                }
            }
            _ => (),
        },
        _ => (),
    }
    Ok(())
}
