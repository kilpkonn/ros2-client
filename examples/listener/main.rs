//use log::error;
use mio::{Events, Poll, PollOpt, Ready, Token};
use ros2_client::{
  Context, Node, NodeOptions,
};
use rustdds::{
  policy::{self, Deadline, Lifespan},
  Duration, QosPolicies, QosPolicyBuilder,
};

fn main() {
  pretty_env_logger::init();

  let mut node = create_node();
  let topic_qos = create_qos();

  let chatter_topic = node
    .create_topic(
      "/chatter",
      String::from("std_msgs::msg::dds_::String_"),
      &topic_qos,
    )
    .unwrap();
  let mut chatter_subscription = node
    .create_subscription::<String>(&chatter_topic, None)
    .unwrap();

  let poll = Poll::new().unwrap();

  poll
    .register(&chatter_subscription, Token(1), Ready::readable(), PollOpt::edge())
    .unwrap();
  let mut events = Events::with_capacity(8);

  loop {
    poll.poll(&mut events, None).unwrap();

    for event in events.iter() {
      match event.token() {
        Token(1) => match chatter_subscription.take() {
          Ok(Some((message, _messafe_info))) => println!("message: {:?}", message),
          Ok(None) => println!("No message?!"),
          Err(e) => {
            println!(">>> error with response handling, e: {:?}", e)
          }
        },
        _ => println!(">>> Unknown poll token {:?}", event.token()),
      } // match
    } // for
  } // lopp
} // main

fn create_qos() -> QosPolicies {
  let service_qos: QosPolicies = {
    QosPolicyBuilder::new()
      .history(policy::History::KeepLast { depth: 10 })
      .reliability(policy::Reliability::Reliable {
        max_blocking_time: Duration::from_millis(100),
      })
      .durability(policy::Durability::Volatile)
      .deadline(Deadline(Duration::DURATION_INFINITE))
      .lifespan(Lifespan {
        duration: Duration::DURATION_INFINITE,
      })
      .liveliness(policy::Liveliness::Automatic {
        lease_duration: Duration::DURATION_INFINITE,
      })
      .build()
  };
  service_qos
}

fn create_node() -> Node {
  let context = Context::new().unwrap();
  let node = context
    .new_node(
      "rustdds_listener",
      "/rustdds",
      NodeOptions::new().enable_rosout(true),
    )
    .unwrap();
  node
}
