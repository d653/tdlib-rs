# tdlib-rs
TDLib Rust high level bindings

WIP

```rust
use tdlib::*;

fn main() {
    let mut api = Api::new();

    let params = TdlibParameters::new(
        false,
        "db/".into(),
        "files/".into(),
        true,
        true,
        true,
        true,
        INSERT_API_ID_HERE,
        "INSERT_API_HASH_HERE".into(),
        "it-IT".into(),
        "Desktop".into(),
        "Linux".into(),
        "1.1.0".into(),
        true,
        true,
    );

    loop {
        let r = api.receive(std::time::Duration::from_secs(10));
        if let Some((tag,obj)) = r {
            println!("{:?} {:?}", tag, obj);
            if let TLObject::Update(Update::AuthorizationState(uas)) = obj {
                match uas.authorization_state {
                    AuthorizationState::WaitTdlibParameters(_) => {
                        api.send(SetTdlibParameters::new(params.clone()));
                    }
                    AuthorizationState::WaitEncryptionKey(_) => {
                        api.send(CheckDatabaseEncryptionKey::new("1234".into()));
                    }
                    AuthorizationState::WaitPhoneNumber(_) => {
                        api.send(CheckAuthenticationBotToken::new(
                            "INSERT_BOT_TOKEN_HERE".into(),
                        ));
                    }
                    AuthorizationState::Ready(_) => {
                        api.send_tagged(12345678,Close::new());
                    }
                    AuthorizationState::Closed(_) => {
                        break;
                    }

                    _ => {}
                }
            }
        }
    }
}
```
