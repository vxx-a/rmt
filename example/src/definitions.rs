use rmt::{http_context, http_gates};

http_gates!(MyService [
    Msg {
        request: { msg: String },
        response: { msg: String, last_msg: String }
    },
    Ping {
        request: { },
        response: { }
    },
    Hello {
        request: { msg: String },
        response: { msg: String }
    }
]);

pub static SERVICE_CONTEXT: rmt::http::Context<MyService> = http_context![ ::2020 ];