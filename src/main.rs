use std::env;
use axum::response::{Html, Redirect};
use axum::{Router};
use axum::body::Bytes;
use axum::extract::{Multipart, State};
use axum::routing::get;
use base64::prelude::*;
use jiff::Zoned;
use sqlx::migrate::Migrator;
use sqlx::SqlitePool;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap()).await.unwrap();
    MIGRATOR.run(&pool).await.unwrap();

    let app = Router::new().route("/home", get(home).post(submit_new_blog))
        .with_state(pool);

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

async fn home(State(pool): State<SqlitePool>) -> Html<String> {
    const PAGE: &str = include_str!("home.html");

    let mut conn = pool.acquire().await.unwrap();

    let posts = sqlx::query!("SELECT * FROM posts")
        .fetch_all(&mut *conn)
        .await
        .unwrap();

    let posts: String = posts
        .into_iter()
        .map(|post| {
            let blog_image = post
                .image
                .map(|image| format!(r#"<img class="blog-image" src="data:image/png;base64,{}"/>"#, BASE64_STANDARD.encode(image)))
                .unwrap_or_else(|| "".to_string());

            let avatar_image = post
                .avatar
                .map(|image| format!(r#"<img class="avatar" src="data:image/png;base64,{}" />"#, BASE64_STANDARD.encode(image)))
                .unwrap_or_else(|| "".to_string());

            format!(r#"
<div class="post">
    {blog_image}

    <p>
        {body}
    </p>

    by {name} {avatar_image} on {on}
</div>
"#,
                    body = post.body,
                    name = post.user_name,
                    on=jiff::Timestamp::from_millisecond(post.publish_date).unwrap().to_string(),
            )
        })
        .collect();

    Html(PAGE.replace("{{ BLOGS }}", &posts))
}

async fn submit_new_blog(State(pool): State<SqlitePool>, input: Multipart) -> Redirect {
    fn bytes_to_vec(bytes: Bytes) -> Option<Vec<u8>> {
        let vec = bytes.to_vec();

        if vec.is_empty() { None } else { Some(vec) }
    }

    let mut conn = pool.acquire().await.unwrap();

    let Some(data) = BlogFormInput::from_multipart(input).await
    else {
        return Redirect::to("/home");
    };
    let avatar_image_data = reqwest::get(data.avatar_url).await.unwrap().bytes().await.map(bytes_to_vec).unwrap();
    let blog_image = data.image.map(bytes_to_vec).flatten();

    let now = Zoned::now().timestamp().as_millisecond();

    sqlx::query!(r"
INSERT INTO posts(body, image, publish_date, user_name, avatar)
VALUES
(?, ?, ?, ?, ?);",
        data.body, blog_image, now, data.user_name, avatar_image_data,
    )
        .execute(&mut *conn)
        .await
        .unwrap();

    Redirect::to("/home")
}