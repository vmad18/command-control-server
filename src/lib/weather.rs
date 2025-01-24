use crate::lib::server::Channel;
use crate::lib::util::{Packet, PACKET_SIZE};
use crate::lib::util::{Context, Node, str_2_bytes};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;


use serde::{Deserialize, Serialize};
use reqwest;
use std::error::Error;
use chrono::{Datelike, Local, NaiveDateTime, TimeZone, Utc}; // Import chrono for time formatting

// weather info json-struct elements
#[derive(Serialize, Deserialize, Debug)]
struct WeatherCondition {
    description: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MainWeather {
    temp: f64,
    feels_like: f64,
    humidity: u8,
    pressure: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct SysInfo {
    country: String,
    sunrise: u64,
    sunset: u64, 
}

#[derive(Serialize, Deserialize, Debug)]
struct WindInfo {
    speed: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct WeatherResponse {
    weather: Vec<WeatherCondition>,
    main: MainWeather,
    wind: WindInfo,
    visibility: u32,
    name: String,
    sys: SysInfo,
}

// forecast json-struct elements
#[derive(Serialize, Deserialize, Debug)]
struct ForecastResponse {
    list: Vec<ForecastItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ForecastItem {
    dt_txt: String,
    main: ForecastMain,
    weather: Vec<WeatherCondition>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ForecastMain {
    temp: f64,
    temp_min: f64,
    temp_max: f64,
}

// weather quality json-struct elements
#[derive(Serialize, Deserialize, Debug)]
struct AQIInfo {
    main: AQIValue,
}

#[derive(Serialize, Deserialize, Debug)]
struct AQIValue {
    aqi: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct AQIResponse {
    list: Vec<AQIInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UVResponse {
    value: f64,
}
/////////////////////////////////////////////

pub struct WeatherParse {
    ctx: Arc<Mutex<Context>>,
    lat: f32,
    lon: f32,
    location: String,
    curr_temp: f32,
    feel_temp: f32,
    sunrise: u64,
    sunset: u64,
    wind: f64,
    humidity: u8,
    pressure: u16,
    curr_weather_cond: String,
    visibility: f32,
    aqi: u8,
    uv: i32,
    next_hrs: [f32; 4],
    next_five_day: Vec<(String, String, f32, f32)>,
    time: String, 
    date: String,
}

impl WeatherParse {
    pub fn new(
        dest: (String, String, String),
        validate: bool,
        wait_time: u128,
        time: SystemTime,
        send_over: Channel,
        lat: f32,
        lon: f32,
    ) -> Box<dyn Node> {
        Box::new(WeatherParse {
            ctx: Arc::new(Mutex::new(Context::new(
                dest, validate, wait_time, time, send_over, true
            ))), 
            lat, 
            lon, 
            location: "".to_string(),
            curr_temp: 0_f32, 
            feel_temp: 0_f32,
            sunrise: 0,
            sunset: 0,
            wind: 0_f64,
            humidity: 0,
            pressure: 0,
            curr_weather_cond: "".to_string(),
            visibility: 0_f32,
            aqi: 0,
            uv: 0,
            next_hrs: [0_f32; 4],
            next_five_day: vec![("".to_string(), "".to_string(), 0_f32, 0_f32); 5],
            time: "".to_string(),
            date: "".to_string()
        })
    }

    async fn populate_weather_data(&mut self, lat: f32, lon: f32) -> Result<(), Box<dyn Error>>  {
        let api_key = "5dcd6ca4510004c571acc72fc8a82847".to_string();

        let weather_url = format!(
            "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}&units=imperial",
            lat, lon, api_key
        );
        
        let weather_response: WeatherResponse = reqwest::Client::new()
            .get(&weather_url)
            .send()
            .await?
            .json()
            .await?;

        let aqi_url = format!(
            "https://api.openweathermap.org/data/2.5/air_pollution?lat={}&lon={}&appid={}",
            lat, lon, api_key
        );

        let aqi_response: AQIResponse = reqwest::Client::new()
            .get(&aqi_url)
            .send()
            .await?
            .json()
            .await?;

        let forecast_url = format!(
            "https://api.openweathermap.org/data/2.5/forecast?lat={}&lon={}&appid={}&units=imperial",
            lat, lon, api_key
        );
        let forecast_response: ForecastResponse = reqwest::Client::new()
            .get(&forecast_url)
            .send()
            .await?
            .json()
            .await?;
    
        for (idx, entry) in forecast_response.list.iter().take(4).enumerate() {
            println!(
                "{} - {:.1}°F, {}",
                entry.dt_txt,
                entry.main.temp,
                entry.weather.get(0).map_or("Unknown", |w| &w.description)
            );

            self.next_hrs[idx] = entry.main.temp as f32;
        }
        
        let mut daily_forecast: Vec<(String, f64, f64, String)> = Vec::new();
        let mut prev_day = String::new();
        for forecast in forecast_response.list.iter() {
            let date = &forecast.dt_txt[0..10];
            
            if date != prev_day {
                daily_forecast.push((
                    date.to_string(),
                    forecast.main.temp_max,
                    forecast.main.temp_min,
                    forecast.weather[0].description.clone(),
                ));
                prev_day = date.to_string();
            }
        }
    
        for (idx, (date, high, low, condition)) in daily_forecast.iter().clone().take(5).enumerate() {
            let parsed_date = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", date), "%Y-%m-%d %H:%M:%S").unwrap();
            let day_of_week = parsed_date.format("%A").to_string(); // Convert date to day of the week
            
            let cond = condition.clone().replace(" ", "_");

            self.next_five_day[idx] = (day_of_week.clone().replace(" ", "_"), cond, *&*high as f32, *&*low as f32);

        }

        // let uv_url = format!(
        //     "https://api.openweathermap.org/data/2.5/air_pollution?lat={}&lon={}&appid={}",
        //     lat, lon, api_key
        // );
        

        // let client = reqwest::Client::new();
        // let uv_resp: UVResponse = client.get(&uv_url).send().await?.json().await?;
        
        let current_moment = Local::now();

        let formatted_date = current_moment.format("%A, %B %d").to_string();
        let curr_day = current_moment.day();
        let day_with_suffix = match curr_day % 10 {
            1  => format!("{}st", curr_day),
            2 => format!("{}nd", curr_day),
            3 => format!("{}rd", curr_day),
            _ => format!("{}th", curr_day)
        };
        
        let final_date = current_moment.format("%A, %B").to_string() + " " + &day_with_suffix;    

        let curr_time = Local::now(); // Get the current local time
        let curr_time_str = curr_time.format("%I: %M").to_string();

        self.location = weather_response.name.replace(" ", "_");
        self.sunrise = weather_response.sys.sunrise;
        self.sunset = weather_response.sys.sunset;
        self.curr_temp = weather_response.main.temp as f32;
        self.feel_temp = weather_response.main.feels_like as f32;
        self.wind = weather_response.wind.speed;
        self.humidity = weather_response.main.humidity as u8;
        self.pressure = weather_response.main.pressure;
        self.curr_weather_cond = weather_response.weather[0].description.clone().replace(" ", "_");
        self.visibility = weather_response.visibility as f32 / 1609.32;
        self.aqi = aqi_response.list[0].main.aqi;
        self.time = curr_time_str.replace(" ", "_");
        self.date = final_date.replace(" ", "_");
        
        // self.uv = uv_resp.value as i32;

        println!("{} uv {} m/s ", self.uv, self.wind);

        Ok(())
    }


}

impl Node for WeatherParse {
    fn gen_packets(&mut self) -> Vec<Packet> {

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let _ = self.populate_weather_data(self.lat, self.lon).await;
        });

        let mut packets: Vec<Packet> = vec![];

        let mut packet_curr_wd = Packet {
            data: [0; PACKET_SIZE],
            size: 0
        };

        let mut datetime = Utc.timestamp_opt(self.sunrise as i64, 0).single().unwrap();
        let sunrise_str = datetime.with_timezone(&Local).format("%I:%M").to_string();
        
        datetime = Utc.timestamp_opt(self.sunset as i64, 0).single().unwrap();
        let sunset_str = datetime.with_timezone(&Local).format("%I:%M").to_string();

        let curr_wd = format!("{} {} {} {} {} {} {} {} {} {} {} {} {} {}", 
                                                                            self.curr_temp as i16, 
                                                                            self.feel_temp as i16, 
                                                                            self.curr_weather_cond, 
                                                                            self.humidity, 
                                                                            self.aqi, 
                                                                            self.visibility, 
                                                                            (self.pressure as f32 * 0.0145037738) as u16, 
                                                                            self.location, 
                                                                            sunrise_str, 
                                                                            sunset_str, 
                                                                            3,
                                                                            self.wind as u32,
                                                                            self.date,
                                                                            self.time);
        
        println!("CW: {}", curr_wd);

        packet_curr_wd.insert(&[0, 0], 2); // data type
        packet_curr_wd.insert(&str_2_bytes(&curr_wd), curr_wd.len());

        packets.push(packet_curr_wd);

        for (idx, info) in self.next_five_day.iter().enumerate() {
            let mut day_packet = Packet {
                data: [0; PACKET_SIZE],
                size: 0
            };

            let curr_id = idx + 1;
            let bytes = [(curr_id >> 8) as u8, (curr_id & 0x00ff) as u8];

            let day_info = format!("{} {} {} {}", info.0, info.1, info.2 as i16, info.3 as i16);

            day_packet.insert(&bytes, 2);
            day_packet.insert(&str_2_bytes(&day_info), day_info.len());

            packets.push(day_packet);
        }

        let mut next_hours_packet = Packet {
            data: [0; PACKET_SIZE],
            size: 0
        };

        let curr_id = 2 + self.next_five_day.len();
        let bytes = [(curr_id >> 8) as u8, (curr_id & 0x00ff) as u8];

        let mut next_hours = String::new();

        for value in self.next_hrs {
            next_hours.push_str(&format!("{} ", value as u8));
        }

        next_hours_packet.insert(&bytes, 2);
        next_hours_packet.insert(&str_2_bytes(&next_hours), next_hours.len());

        packets.push(next_hours_packet);
        packets
    }

    fn get_ctx(&mut self) -> Arc<Mutex<Context>> {
        self.ctx.clone()
    }
}