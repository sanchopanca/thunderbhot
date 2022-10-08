use std::env;

use rand::seq::SliceRandom;
use rand::thread_rng;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::{convert::Infallible, net::SocketAddr};

use tera::{Context as TeraContext, Tera};

use futures::join;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::ChannelId;
use serenity::prelude::*;

use lazy_static::lazy_static;

use dashmap::DashMap;

lazy_static! {
    static ref BOT_RULES: DashMap<Vec<&'static str>, Vec<&'static str>> = {
        let rules = DashMap::new();
        let kpop = vec![
            "https://youtu.be/9bZkp7q19f0",
            "https://youtu.be/POe9SOEKotk",
            "https://youtu.be/5UdoUmvu_n8",
            "https://youtu.be/2e-Q7GfCGbA",
            "https://youtu.be/id6q2EP2UqE",
            "https://youtu.be/8dJyRm2jJ-U",
            "https://youtu.be/JQGRg8XBnB4",
            "https://youtu.be/Hbb5GPxXF1w",
            "https://youtu.be/p1bjnyDqI9k",
            "https://youtu.be/k6jqx9kZgPM",
            "https://youtu.be/z8Eu-HU0sWQ",
            "https://youtu.be/eH8jn4W8Bqc",
            "https://youtu.be/IHNzOHi8sJs",
            "https://youtu.be/WPdWvnAAurg",
            "https://youtu.be/gdZLi9oWNZg",
            "https://youtu.be/H8kqPkEXP_E",
            "https://youtu.be/awkkyBH2zEo",
            "https://youtu.be/z3szNvgQxHo",
            "https://youtu.be/i0p1bmr0EmE",
            "https://youtu.be/WyiIGEHQP8o",
            "https://youtu.be/lcRV2gyEfMo",
        ];
        rules.insert(vec!["kpop time", "k p o p   t i m e", "kpop tijd"], kpop);
        rules.insert(
            vec!["hat a week huh", "hat a week, huh"],
            vec!["https://whataweek.eu"],
        );
        rules
    };
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        }
    };
}

struct Handler;

fn match_message(message: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| message.contains(p))
}

async fn send_message(channel: ChannelId, ctx: Context, message: &str) {
    if let Err(why) = channel.say(&ctx.http, message).await {
        println!("Error sending message: {:?}", why);
    }
}

fn random_choice<'a>(v: &[&'a str]) -> &'a str {
    v.choose(&mut thread_rng()).unwrap() // todo: empty vector
}

fn respond(message: &str) -> Option<&str> {
    for entry in BOT_RULES.iter() {
        let prompts = entry.key();
        let responses = entry.value();
        if match_message(message, prompts) {
            return Some(random_choice(responses));
        }
    }
    None
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        match respond(&msg.content) {
            Some(response) => send_message(msg.channel_id, ctx, response).await,
            None => (),
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

async fn handle_web_request(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut ctx = TeraContext::new();
    ctx.insert("who", "world");
    Ok(Response::new(
        TEMPLATES.render("index.html", &ctx).unwrap().into(),
    ))
}

// I'm not sure sqlite will work well in multithread env,
// so limit everything to one thread for now, even if we don't use sqlite currently
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let token = env::var("DISCORD_API_TOKEN").expect("Provide DISCORD_API_TOKEN env variable");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_web_request)) });

    let server = Server::bind(&addr).serve(make_svc);

    match join!(client.start(), server) {
        (Err(client_error), Err(server_error)) => {
            eprintln!("Discord client error: {:?}", client_error);
            eprintln!("Error starting web server: {:?}", server_error);
        }
        (Err(client_error), _) => eprintln!("Discord client error: {:?}", client_error),
        (_, Err(server_error)) => eprintln!("Error starting web server: {:?}", server_error),
        _ => (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_message_works() {
        let patterns = &["kpop time", "kpop tijd"];
        assert!(match_message("Is it kpop time yet", patterns));
        assert!(match_message("Is het al kpop tijd?", patterns));
        assert!(!match_message("It's Britney time", patterns));
    }
}
