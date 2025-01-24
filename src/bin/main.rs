use std::time::SystemTime;

use command_control_server::lib::{
    server::{Channel, CommandControl},
    weather::WeatherParse,
};


fn main() {
    let mut cc = CommandControl::create();
    let dest = ("192.168.4.1".to_string(), "10000".to_string(), "xX_uwu_pico_ap_uwu_Xx".to_string());
    let weather = WeatherParse::new(
        dest.clone(),
        false,
        20000,
        SystemTime::now(),
        Channel::UDP,
        100.0, 
        100.0
    );

    cc.push_node(weather);

    if false { 
        println!("Connecting first time");
        CommandControl::connect_ap_first_time(&dest.2, &"uwu_pico_ap_i_wonder_who_connects_123_uwu".to_string(), &"192.168.4.1".to_string(), &"192.168.4.2/24".to_string());
    }

    cc.run();
}
