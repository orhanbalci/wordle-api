use serde::{Deserialize, Serialize};
use worker::*;
use chrono::{DateTime, Utc};
use chrono::serde::ts_milliseconds;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[derive(Serialize, Deserialize, Default)]
pub struct Dictionary {
    pub words: Vec<String>,
}


#[derive(Serialize, Deserialize )]
pub struct Daily {
    pub word: String,
    #[serde(with = "ts_milliseconds")]
    pub date: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Previous {
    pub previous: Vec<Daily>,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get("/", |_, _| Response::ok("Hello from Wordle!"))
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .get_async("/words/:lang", |_req, ctx| async move {
            if let Some(lang) = ctx.param("lang") {
                if lang != "tr" {
                    return Response::error("Language not found", 404);
                } else {
                    let wordle_namespace = ctx.kv("wordle")?;
                    let dictionary = wordle_namespace.get("dictionary").await?;
                    return Response::from_json::<Dictionary>(&dictionary.map_or(
                        Dictionary::default(),
                        |a| {
                            a.as_json::<Dictionary>()
                                .expect("can not serialize dictionary kv value")
                        },
                    ));
                }
            }
            return Response::error("Bad request", 404);
        })
        .get_async("/word/today/:lang", |_req, ctx| async move {
            if let Some(lang) = ctx.param("lang") {
                if lang != "tr" {
                    return Response::error("Language not found", 404);
                } else {
                    let wordle_namespace = ctx.kv("wordle")?;
                    let todays_word = wordle_namespace.get(&format!("today_{}",lang)).await?;
                    return Response::from_json::<Daily>(&todays_word.map_or(
                        Daily{word: String::new(), date: Utc::now()},
                        |a| {
                            a.as_json::<Daily>()
                                .expect("can not serialize todays word kv value")
                        },
                    ));
                }
            }
            return Response::error("Bad request", 404);
        })
        .get_async("/word/previous/:lang", |_req, ctx| async move {
            if let Some(lang) = ctx.param("lang") {
                if lang != "tr" {
                    return Response::error("Language not found", 404);
                } else {
                    let wordle_namespace = ctx.kv("wordle")?;
                    let previous_words = wordle_namespace.get(&format!("previous_{}",lang)).await?;
                    return Response::from_json::<Previous>(&previous_words.map_or(
                        Previous::default(),
                        |a| {
                            a.as_json::<Previous>()
                                .expect("can not serialize previous_words kv value")
                        },
                    ));
                }
            }
            return Response::error("Bad request", 404);
        })
        .run(req, env)
        .await
}
