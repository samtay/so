use reqwest::Client;

use crate::config::Config;

const SE_URL: String = "http://api.stackexchange.com/2.2/";
const SE_ENDPOINT_SEARCH: String = "search/advanced";

struct StackExchange {
    client: Client,
    config: Config,
}

struct Question {
    id: u32,
    score: i32,
    answers: Vec<Answer>,
    title: String,
    body: String,
}

struct Answer {
    id: u32,
    score: i32,
    body: String,
    accepted: bool,
}

impl StackExchange {
    pub fn new(config: Config) -> Self {
        let client = Client::new();
        StackExchange { client, config }
    }

    // https://stackoverflow.com/a/57770687 is the right way to do this
    pub fn query(q: &str) -> Result<Vec<Question>, String> {
        let request_url = format!(
            "{url}{endpoint}",
            url = SE_URL,
            endpoint = SE_ENDPOINT_SEARCH
        );
        println!("{}", request_url);
        let mut response = reqwest::get(&request_url)?;

        let users: Vec<User> = response.json()?;
        println!("{:?}", users);
        Ok(())
        //Err("Not implemented".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stackexchange() {
        assert!(true)
    }
}
