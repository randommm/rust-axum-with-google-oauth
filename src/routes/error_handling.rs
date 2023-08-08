use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};

pub struct AppError {
    code: StatusCode,
    message: String,
    user_message: String,
}

impl AppError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            user_message: "".to_owned(),
            code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    pub fn with_user_message(self, user_message: impl Into<String>) -> Self {
        Self {
            user_message: user_message.into(),
            ..self
        }
    }
    // pub fn with_code(self, code: StatusCode) -> Self {
    //     Self {
    //         code,
    //         ..self
    //     }
    // }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        println!("AppError: {}", self.message);
        (
            self.code,
            Html(format!(
                r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="utf-8">
                    <title>Oops!</title>
                </head>
                <body>
                    <h1>Oops!</h1>
                    <p>Sorry, but something went wrong.</p>
                    <p>{}</p>
                </body>
                </html>
                "#,
                self.user_message
            )),
        )
            .into_response()
    }
}

impl From<minijinja::Error> for AppError {
    fn from(err: minijinja::Error) -> Self {
        AppError::new(format!("Template error: {:#}", err))
    }
}

impl From<dotenvy::Error> for AppError {
    fn from(err: dotenvy::Error) -> Self {
        AppError::new(format!("Dotenv error: {:#}", err))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::new(format!("Database query error: {:#}", err))
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::new(err)
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::new(err)
    }
}
