use crate::runtime::action::builtin::remote::RemoteHttpAction;
use crate::runtime::action::{Impl, ImplRemote};
use crate::runtime::args::{RtArgs, RtArgument, RtValue};
use crate::runtime::blackboard::BlackBoard;
use crate::runtime::builder::ServerPort;
use crate::runtime::context::{TreeContextRef, TreeRemoteContextRef};
use crate::runtime::env::RtEnv;
use crate::runtime::forester::serv::start;
use crate::runtime::TickResult;
use crate::tests::{fb, turn_on_logs};
use crate::tracer::Tracer;
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn smoke_serv() {
    turn_on_logs();
    let bb = Arc::new(Mutex::new(BlackBoard::default()));
    let tr = Arc::new(Mutex::new(Tracer::default()));

    let rt = RtEnv::try_new().unwrap().runtime;

    let info = start(&rt, ServerPort::Static(9999), bb.clone(), tr.clone()).unwrap();
    let stop = info.stop_cmd;

    rt.spawn(async {
        tokio::time::sleep(Duration::from_secs(2)).await;
        stop.send(()).unwrap();
    });

    rt.block_on(async {
        info.status.await.unwrap().unwrap();
    })
}

#[test]
fn remote_smoke() {
    turn_on_logs();
    let action = RemoteHttpAction::new("http://localhost:10000/action".to_string());
    let mut env = RtEnv::try_new().unwrap();
    let result = action.tick(
        RtArgs(vec![
            RtArgument::new("a".to_string(), RtValue::int(1)),
            RtArgument::new("b".to_string(), RtValue::str("a".to_string())),
            RtArgument::new(
                "c".to_string(),
                RtValue::Array(vec![RtValue::int(1), RtValue::int(2)]),
            ),
        ]),
        TreeRemoteContextRef::new(1, 9999, &mut env),
    );

    println!("{:?}", result);
}

#[test]
fn remote_serv() {
    turn_on_logs();

    let mut builder = fb("actions/remote");

    builder.tracer(Tracer::default());

    let action = RemoteHttpAction::new("http://localhost:10001/action".to_string());
    builder.register_remote_action("action", action);
    builder.http_serv(9999);
    let mut f = builder.build().unwrap();

    let result = f.run();

    println!("{:?}", result);
}

#[test]
fn remote() {
    let mut env = RtEnv::try_new().unwrap();

    let port = env.runtime.block_on(async {
        let mock_server = MockServer::start().await;

        let mut resp = ResponseTemplate::new(200);
        let resp = resp.set_body_json(json!("Success"));

        Mock::given(method("POST"))
            .and(path("/action"))
            .respond_with(resp)
            .mount(&mock_server)
            .await;
        mock_server.address().port()
    });

    let action = RemoteHttpAction::new(format!("http://localhost:{}/action", port));
    let mut fb = fb("actions/simple_http");
    fb.rt_env(env);
    fb.register_remote_action("a", action);

    let mut f = fb.build().unwrap();
    let result = f.run();
    assert_eq!(result, Ok(TickResult::success()));
}
