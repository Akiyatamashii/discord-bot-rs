# Discord Bot in Rust

This is a multi-functional Discord bot developed using the Rust programming language.

[中文](README_TW.md)

## Features

1. **Reminder System**

   - Set timed reminders
   - View current reminders
   - Remove specific reminders

2. **OpenAI Integration**

   - Chat with ChatGPT
   - Generate images
   - View available AI models

3. **Debt Tracking System**

   - Record debt information
   - View debt list
   - Delete debt records

4. **Basic Commands**

   - View bot information
   - Test bot response time

5. **Ban and Punishment System**

   - Ban users
   - Unban users

6. **Anti-TikTok Feature**
   - Automatically handle TikTok links (currently limited to specific groups)

## Installation and Setup

1. Ensure you have Rust and Cargo installed.
2. Clone this repository:
   ```
   git clone https://github.com/Akiyatamashii/discord-bot-rs.git
   ```
3. Navigate to the project directory:
   ```
   cd discord-bot-rs
   ```
4. Create a `.env` file and add the following content:
   ```
   TOKEN=your_discord_bot_token // Discord bot token
   API_KEY=your_openai_api_key // OpenAI API key (optional if not needed)
   ```
5. Compile and run the bot:
   ```
   cargo run // Run
   ```
   or
   ```
   cargo build --release // Compile
   ./target/release/discord-bot // Run
   ```
6. Add the bot to your Discord server:
   - Create a new bot in the Discord Developer Portal using your bot token.
   - Add the bot to your server.
   - Use the `/register` command in the server to register bot commands.
   - Use `info` to get more information about the bot.
7. Enjoy your Discord bot!

## Usage

The bot supports the following slash commands:

### Basic Functions (Common)

- `/info` - View basic bot information
- `/info [type]` - View detailed instructions for specific features
- `/ping` - Test connection

### Reminder

- `/remind [weekdays] [time] [message]` - Set a reminder
- `/rm_remind [index]` - Remove a specific reminder
- `/look` - View current reminders

### AI Generation (OpenAI)

- `/chat [message] [public] [model]` - Chat with ChatGPT
- `/image [prompt] [public] [model]` - Generate an image
- `/model_list` - View available AI models

### Debt Tracking System (Cash)

- `/cash look` - View current debts
- `/cash add [debtor] [creditor] [debt] [ps]` - Add a debt record
- `/cash del [index]` - Delete a specific debt record

### Ban and Punishment System (Ban)

- `/ban [member] [mins]` - Ban a user
- `/unban [member]` - Unban a user

For detailed command usage instructions, please refer to the corresponding markdown files in the `info/` directory.

## Development Notes

This project uses the following main dependencies:

- serenity: Discord API client
- async-openai: OpenAI API client
- tokio: Asynchronous runtime
- chrono: Date and time handling
- serde: Serialization and deserialization

For a complete list of dependencies, please check the `Cargo.toml` file.

## Contributing

Pull requests are welcome to improve this project. For major changes, please open an issue first to discuss what you would like to change.

## Version

Current version: 0.7.7

## License

This project is licensed under the [MIT License](LICENSE).

## Author

Made By **_Akiyatamashii_**
