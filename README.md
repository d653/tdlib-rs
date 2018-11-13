# tdlib-rs
TDLib Rust high level bindings

WIP

The specifications of the tdlib json API are terrible. They are designed with OOP in mind. Types are defined using inheritance. Pointers may be null, and one needs to read the //comments to see if it is the case. Parameters that may be null are not specified at all. Some fields are just omitted, when they have some default value. Some lines of the original .tl file contain errors, like unexpected tokens. Also, in the .tl file, some functions are just described wrongly. This library tries to model these objects using Rust enums, using a manually fixed .tl file that gets automatically converted to Rust code.

Currently, the deserialization of the incoming updates may be slow. This library uses the serde untagged attribute in order to deserialize automatically to enums. The untagged attribute is described as follows. "There is no explicit tag identifying which variant the data contains. Serde will try to match the data against each variant in order and the first one that deserializes successfully is the one returned."

Anyway, this is a working example:

```rust
use tdlib::*;

fn main() {

    let mut api = Api::new();


    loop {
        let r = api.receive(std::time::Duration::from_secs(10));
        if let Some((tag,obj)) = r {
            println!("{:?} {:?}", tag, obj);
            match obj {
                TLObject::Update(Update::AuthorizationState(uas)) => handle_auth(&mut api, uas),
                TLObject::Update(Update::NewMessage(msg)) => handle_msg(&mut api, msg),
                _ => {}
            }
        }
    }
}


fn handle_auth(api:&mut Api, uas : UpdateAuthorizationState){
    match uas.authorization_state {
        AuthorizationState::WaitTdlibParameters(_) => {
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
            //api.send_tagged(123,Close::new());
        }
        AuthorizationState::Closed(_) => {
            break;
        }

        _ => {}
    }
}

fn handle_msg(api:&mut Api, msg : UpdateNewMessage) {
    let msg = msg.message;
    let uid = msg.sender_user_id;
    let cid = msg.chat_id;
    match msg.content {
        MessageContent::MessageText(MessageText{text:FormattedText{text,..},..}) => {
            println!("{} wrote {}", uid, text);
            if text == "/test" {
                let txt = InputMessageText::new(FormattedText::new(format!("hello!"),vec![]),false,true);
                api.send(SendMessage::new(cid,0,false,false,None,txt));
            }
        },
        _ => {}
    }
}
```
