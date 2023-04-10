use std::error::Error;
use std::{
    io::{stdin, stdout, Write},
};
use dotenvy::dotenv;

use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().unwrap();

    let client = Client::new();

    let mut memory_vec = Vec::new();

    loop {
        let mut lock = stdout().lock();

        write!(lock, ">    ").unwrap();
        stdout().flush()?;

        let mut user_message_content = String::new();
        stdin().read_line(&mut user_message_content).unwrap();

        let message_arg = ChatCompletionRequestMessageArgs::default()
                .content(user_message_content)
                .role(Role::User)
                .build()?;

        memory_vec.push(message_arg.clone());

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .max_tokens(512u16)
            .messages(memory_vec.clone())
            .build()?;

        let mut stream = client.chat().create_stream(request).await?;

        let mut assistant_message_content = String::new();

        write!(lock, "GPT: ").unwrap();
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    response.choices.iter().for_each(|chat_choice| {
                        if let Some(ref content) = chat_choice.delta.content {
                            assistant_message_content += content;
                            let Ok(_) = write!(lock, "{}", content) else {
                                return;
                            };
                        }
                    });
                }
                Err(err) => {
                    writeln!(lock, "error: {err}")?;
                }
            }
            stdout().flush()?;
        }
        writeln!(lock, "").unwrap();

        memory_vec.push(
            ChatCompletionRequestMessageArgs::default()
                .content(assistant_message_content)
                .role(Role::Assistant)
                .build()?
        )
    }
}
