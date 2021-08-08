use openweathermap::*;
use structopt::StructOpt;
use anyhow::{Result, bail};
//use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use chrono::{Utc, Duration, TimeZone};

#[derive(StructOpt)]
struct Opt {

    #[structopt(long)]
    lat: Option<f64>,
    
    #[structopt(long)]
    lon: Option<f64>,

    #[structopt(long)]
    loc: Option<String>,

    #[structopt(long)]
    days: f64,

    #[structopt(long, default_value = "MY_API_KEY")]
    api_key: String
}

fn get_latlonloc(lat:f64, lon:f64, loc:String) -> (f64, f64, String) {
    if loc == "Mickleham" {
        return (51.268, -0.321, loc); 
    }
    else if loc == "Preveza" { 
        return (38.95, 20.73, loc); 
    }
    else if loc == "Castlegregory" { 
        return (52.255549, -10.02099, loc); 
    }
    else if loc == "Casa" { 
        return (41.895556, 2.806389, loc); 
    }
    else {
        return (lat, lon, format!("{}, {}", lat, lon));
    };
}

fn print_current(current:Current, location:String) {
    println!("Weather for {} on {}", location, Utc.timestamp(current.dt, 0));
    for elem in current.weather {
        println!("Short weather: {}", elem.main);
        println!("Weather description: {}", elem.description);
    }
    println!("Temperature: {}ºC", current.temp);
    println!("Humidity: {}%", current.humidity);
    println!("Pressure: {}hPa", current.pressure);
    println!("Cloud cover: {}%", current.clouds);
    println!("Dew Point: {}ºC", current.dew_point);
    println!("Heat Index: {}ºC", current.feels_like);
    match current.snow {
        Some(snow) => {
            match snow.h1 {
                Some(h1) => {
                    println!("Snow one-hour: {}mm", h1);
                }
                None => {}
            }
            match snow.h3 {
                Some(h3) => {
                    println!("Snow three-hour: {}mm", h3);
                }
                None => {}
            }
        }
        None => {}
    };
    match current.rain {
        Some(rain) => {
            match rain.h1 {
                Some(h1) => {
                    println!("Rain one-hour: {}mm", h1);
                }
                None => {}
            }
            match rain.h3 {
                Some(h3) => {
                    println!("Rain three-hour: {}mm", h3);
                }
                None => {}
            }
        }
        None => {}
    }
    println!("Sunrise: {}", Utc.timestamp(current.sunrise, 0));
    println!("Sunset: {}", Utc.timestamp(current.sunset, 0));
    println!("UV Index: {}", current.uvi);
    println!("Visibility: {}m", current.visibility);
    println!("Wind degrees: {}º", current.wind_deg);
    match current.wind_gust {
        Some(gust) => {
            println!("Wind gust: {}m/s", gust); 
        }
        None => {}
    }
    println!("Wind speed: {}m/s", current.wind_speed);
}

fn main() -> Result<()> {
   let opt = Opt::from_args();
   let latitude = opt.lat.unwrap_or_default();
   let longitude = opt.lon.unwrap_or_default();
   let location = opt.loc.unwrap_or_default();
   let days = opt.days;

   if days < 0.0 || days > 5.0 {
        bail!("Day offset '{}' not between one and five", days);
   }

   let now = Utc::now();
   let seconds = days * 24.0 * 60.0 * 60.0;
   let yesterday = now.checked_sub_signed(Duration::seconds(seconds.round() as i64)).unwrap();
   let yesterday_unix = yesterday.timestamp();

   let latlonloc = get_latlonloc(latitude, longitude, location);
   let api_result = blocking::timemachine(&latlonloc.0, &latlonloc.1, &yesterday_unix, "metric", "en", &opt.api_key).unwrap();
    
   print_current(api_result.current, latlonloc.2);

   Ok(())
}
