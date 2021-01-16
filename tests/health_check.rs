use zero2prod::run;
use std::net::TcpListener;

// Launch our application in the background
fn spawn_app()  -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port.");
    // Grab the port assigned by the OS
    let port = listener.local_addr().unwrap().port();
    // New dev dependency - let's add tokio to the party with
    // `cargo add tokio --dev`
    let server = run(listener).expect("Failed to bind address");
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);
    // Return application address
    format!("http://127.0.0.1:{}", port)
}

// `actix_rt::test` is the testing equivalent of `actix_rt::main`.
// It also spares you from having to specify the `#[test]` attribute.
// You can inspect what code gets generated using
// `cargo expand --test health_check` (<- name of the test file
#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();
    // We brought `reqwest` in as a _development_ dependency
    // to perform HTTP requests against our application.
    // Either add it manually under [dev-dependencies] in Cargo.toml
    // or run `cargo add reqwest --dev`
    let client = reqwest::Client::new();

    // Act
    let response = client
    .get(&format!("{}/health_check", &address))
    .send()
    .await
    .expect("Failed to execute request.");

    // Assert 
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let body = "name=Mister%20Potato%20&email=mr.potato.head%40gmail.com";

    // Act
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[actix_rt::test]
async fn subscrive_returns_a_400_when_data_is_missing() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=Mister%20Potato%20Head", "missing the email"),
        ("email=mr.potato.head%40gmail.com","missing the name"),
        ("","missing both the name and email")
    ];

    // Loop through test cases
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_body)
        .send()
        .await
        .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customized error message
            "The API did not fail with 400 Bad Request when {}.",
            error_message
        );
    }
}