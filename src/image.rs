use crate::prelude::*;

pub fn message_check_image<'a>(message: &'a serenity::Message) -> bool {
    for attachment in &message.attachments {
        if attachment.content_type.is_some() {
            if (|t| t == "image/jpeg" || t == "image/png" || t == "image/gif" || t == "image/webp")(
                attachment.content_type.as_ref().unwrap(),
            ) {
                return true;
            }
        }
    }
    for embed in &message.embeds {
        if embed.image.is_some() && embed.image.as_ref().unwrap().proxy_url.is_some() {
            return true;
        }
        if embed.thumbnail.is_some() && embed.thumbnail.as_ref().unwrap().proxy_url.is_some() {
            return true;
        }
    }
    return false;
}

pub fn update_check_image<'a>(update: &'a serenity::MessageUpdateEvent) -> bool {
    if update.attachments.is_some() {
        for attachment in update.attachments.as_ref().unwrap() {
            if attachment.content_type.is_some() {
                if (|t| {
                    t == "image/jpeg" || t == "image/png" || t == "image/gif" || t == "image/webp"
                })(attachment.content_type.as_ref().unwrap())
                {
                    return true;
                }
            }
        }
    }
    if update.embeds.is_some() {
        for embed in update.embeds.as_ref().unwrap() {
            if embed.image.is_some() && embed.image.as_ref().unwrap().proxy_url.is_some() {
                return true;
            }
            if embed.thumbnail.is_some() && embed.thumbnail.as_ref().unwrap().proxy_url.is_some() {
                return true;
            }
        }
    }
    return false;
}

pub fn message_list_images<'a>(message: &'a serenity::Message) -> Vec<&'a str> {
    let mut image_list: Vec<&str> = Vec::new();
    for attachment in &message.attachments {
        if attachment.content_type.is_some() {
            if (|t| t == "image/jpeg" || t == "image/png" || t == "image/gif" || t == "image/webp")(
                attachment.content_type.as_ref().unwrap(),
            ) {
                image_list.push(&attachment.proxy_url);
            }
        }
    }
    for embed in &message.embeds {
        if embed.image.is_some() && embed.image.as_ref().unwrap().proxy_url.is_some() {
            image_list.push(&embed.image.as_ref().unwrap().proxy_url.as_ref().unwrap());
        }
        if embed.thumbnail.is_some() && embed.thumbnail.as_ref().unwrap().proxy_url.is_some() {
            image_list.push(
                &embed
                    .thumbnail
                    .as_ref()
                    .unwrap()
                    .proxy_url
                    .as_ref()
                    .unwrap(),
            );
        }
    }
    image_list
}

/// return the evaluation result of a pair of (url, proxy_url)
pub async fn evaluate_image<'a, 'b>(
    url: &'b str,
) -> Result<std::collections::BTreeMap<u32, f32>, i32> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::process::Command;

    let evaluater = Command::new("./eval_pic")
        .current_dir("./eval")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn();
    if evaluater.is_err() {
        return Err(-5);
    }

    let mut evaluater = evaluater.unwrap();
    evaluater
        .stdin
        .as_mut()
        .unwrap()
        .write_all(format!("{url}\n").as_bytes())
        .await
        .unwrap();
    evaluater.stdin.as_mut().unwrap().flush().await.unwrap();
    let mut reader = BufReader::new(evaluater.stdout.as_mut().unwrap());
    let mut read_buf = String::new();
    reader.read_line(&mut read_buf).await.unwrap();
    let mut read_buf = String::new();
    reader.read_line(&mut read_buf).await.unwrap();
    let line_count = read_buf.trim().parse::<i32>().unwrap();
    if line_count >= 0 {
        let mut result = std::collections::BTreeMap::new();
        for _ in 0..line_count {
            read_buf.clear();
            reader.read_line(&mut read_buf).await.unwrap();
            let tag = read_buf.trim().split(' ').collect::<Vec<&str>>();
            assert_eq!(tag.len(), 2);
            result.insert(
                tag[0].parse::<u32>().unwrap(),
                tag[1].parse::<f32>().unwrap(),
            );
        }
        return Ok(result);
    } else {
        return Err(line_count);
    }
}
