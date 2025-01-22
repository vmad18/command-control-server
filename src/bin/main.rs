use std::time::SystemTime;

use command_control_server::lib::{
    server::{Channel, CommandControl},
    weather::WeatherParse,
};
fn main() {
    let mut cc = CommandControl::create();
    let weather = WeatherParse::new(
        ("100".to_string(), "1".to_string(), "2".to_string()),
        false,
        1000,
        SystemTime::now(),
        Channel::UDP,
    );

    cc.push_node(weather);

    cc.run();

    CommandControl::curr_network();
}
