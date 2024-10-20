use axum::response::{Html, Redirect};
use axum::{Router};
use axum::body::Bytes;
use axum::extract::Multipart;
use axum::routing::get;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new().route("/home", get(home).post(submit_new_blog));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug)]
struct BlogFormInput {
    body: String,
    image: Option<Bytes>,
    user_name: String,
    avatar_url: String,
}

impl BlogFormInput {
    async fn from_multipart(mut input: Multipart) -> Option<Self> {
        let mut body = None;
        let mut image = None;
        let mut user_name = None;
        let mut avatar_url = None;

        while let Some(field) = input.next_field().await.unwrap() {
            let name = field.name().unwrap().to_string();

            match name.as_str() {
                "body" => {
                    body = Some(field.text().await.unwrap());
                }
                "image" => {
                    image = Some(field.bytes().await.unwrap());
                }
                "user_name" => {
                    user_name = Some(field.text().await.unwrap());
                }
                "avatar" => {
                    avatar_url = Some(field.text().await.unwrap());
                }
                unknown => tracing::info!("Unknown field: '{unknown}', skipping..."),
            }
        }

        Some(Self {
            body: body?,
            image,
            user_name: user_name?,
            avatar_url: avatar_url?,
        })
    }
}

async fn home() -> Html<String> {
    const PAGE: &str = include_str!("home.html");

    Html(PAGE.replace("{{ BLOGS }}", ""))
}

async fn submit_new_blog(input: Multipart) -> Redirect {
    let Some(data) = BlogFormInput::from_multipart(input).await
    else {
        return Redirect::to("/home");
    };

    tracing::debug!("Got {data:?}");

    let avatar_image_data = reqwest::get(data.avatar_url).await.unwrap();

    tracing::debug!("Got avatar data: {}", avatar_image_data.status());

    Redirect::to("/home")
}