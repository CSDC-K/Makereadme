use groq_api_rs::completion::{client::Groq, message::Message, request::builder};

use crate::printd;
use crate::libs::errors::Error;


pub async fn create_communication(api_key: String, prompt: String, model_type: String) -> Result<bool, Error> {

    let mut client = Groq::new(api_key.as_str());
    client.add_messages(vec![Message::SystemMessage {
        role: Some("system".to_string()),
        content: Some(prompt),
        name: None,
        tool_call_id: None,
    }]);

    printd!("Communication created with Groq API", Success);
    printd!("Starting conversation loop...", Debug);

    loop {
        client.add_messages(vec![Message::UserMessage {
            role: Some("system".to_string()),
            content: Some("READMEMAKER AGENTIC LOOP IS STARTED, START TALKING".to_string()),
            name: None,
            tool_call_id: None,
        }]);

        loop {
            let request = builder::RequestBuilder::new(model_type.clone());
            let res = client.create(request).await;

            match res {
                Ok(groq_api_rs::completion::client::CompletionOption::NonStream(response)) => {
                    ()
                }
                Ok(groq_api_rs::completion::client::CompletionOption::Stream(_stream)) => {
                    // When i done with creating prompt file, i will implement this part for streaming responses
                }
                Err(e) => {
                    return Err(Error::RunError(format!("Groq API communication failed: {}", e)));
                }
            }

        }

    }

}