mod cli;
mod config;
mod stackexchange;

use config::Config;
use stackexchange::StackExchange;

fn main() {
    let matches = cli::mk_app().get_matches();

    // TODO merge config from ArgMatch
    let se = StackExchange::new(Config {
        api_key: String::from("8o9g7WcfwnwbB*Qp4VsGsw(("),
        limit: 1,
        site: String::from("stackoverflow"),
    });

    (|| -> Option<_> {
        let q = cli::get_query(matches)?;
        let que = se.search(&q).unwrap(); // TODO eventually be graceful
        let ans = que.first()?.answers.first()?;
        println!("{}", ans.body);
        Some(())
    })();
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_main() {
        //TODO
    }
}
