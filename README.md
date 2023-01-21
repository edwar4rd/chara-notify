# Image Notification Bot

This discord bot attempts to deduce whether a sended image/embed includes a certain figure,
then ping certain user in the server that had suscribed to the character.

## Features

- [ ] the bot detects messages containing images in certain channel when activated
    - [ ] exact channel in a server is configurable via commands
- [ ] users can suscribe to a tag via commands
  - [ ] since the accuracy of evaluation process is at best questionable, the user can configure whether they want to be pinged even when the bot if very uncertain if a picture contains a character, or only when the bot can pretty much be sure that the character is presented
- [ ] the bot evaluates whether the image contains certain features and ping suscribed users respectively
  - [ ] the bot can be configured to ping certain roles instead of individual users (per guild)
  - [ ] the bot can log the evaluation result to a separate channel if configured so

## Commands

### Normal Commands

### Admin Commands

These commands are only available for guild admins.

- `/config channel-add <channel>` add a channel for the bot to monitor
- `/config channel-remove <channel>` remove a channel from the bot's monitor list

## Usage

- If a image (or a message with embeded image) is posted in one of the selected channels, the bot react to the image with a custom magnifying glass emote.
- A user can also react with the emote, and by so tell the bot to evaluate the image.
  The bot will than evaluate what feature is presented in the image and print all recognized feature with a score higher than 0.1 in a message, or the features with the highest scores that fit in a message.
