use openweathermap::*;
use structopt::StructOpt;
use anyhow::{Result, bail};
//use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use chrono::{Utc, Duration, TimeZone, FixedOffset};

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

fn get_latlonloc(lat:f64, lon:f64, loc:String) -> (f64, f64, String, FixedOffset) {
    let mut m_lat = lat;
    let mut m_lon = lon;
    let mut timeoffset = FixedOffset::west(0);

    if loc == "Mickleham" {
        m_lat = 51.268;
        m_lon = -0.321;
        timeoffset = FixedOffset::east(1 * 3600);
    }
    else if loc == "Preveza" { 
        m_lat = 38.95;
        m_lon = 20.73;
        timeoffset = FixedOffset::east(3 * 3600);
    }
    else if loc == "Castlegregory" {
        m_lat = 52.255549;
        m_lon = -10.02099; 
        timeoffset = FixedOffset::east(1 * 3600);
    }
    else if loc == "Casa" { 
        m_lat = 41.895556;
        m_lon = 2.806389;
        timeoffset = FixedOffset::east(2 * 3600);
    }
    else if loc == "Austin" {
        m_lat = 30.267222;
        m_lon = -97.743056; 
        timeoffset = FixedOffset::west(5 * 3600);      
    }
    else if loc == "Cary" {
        m_lat = 35.791667;
        m_lon = -78.781111;     
        timeoffset = FixedOffset::west(4 * 3600);   
    }
    else if loc == "Black_Forest" {
        m_lat = 39.060825;
        m_lon = -104.67525;
        timeoffset = FixedOffset::west(6 * 3600);
    }
    else if loc == "Hoopa" {
        m_lat = 41.050278;
        m_lon = -123.674167;
        timeoffset = FixedOffset::west(7 * 3600);
    }

    return (m_lat, m_lon, format!("{} [{},{}]", loc, m_lat, m_lon), timeoffset);
}

fn print_current(current:Current, location:String, timezone:FixedOffset) {
    println!("Weather for {} on {}", location, Utc.timestamp(current.dt, 0).with_timezone(&timezone));
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
    println!("Sunrise: {}", Utc.timestamp(current.sunrise, 0).with_timezone(&timezone));
    println!("Sunset: {}", Utc.timestamp(current.sunset, 0).with_timezone(&timezone));
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
   let location = opt.loc.unwrap_or_default();
   let days = opt.days;

   if days < 0.0 || days > 5.0 {
        bail!("Day offset '{}' not between one and five", days);
   }

   let now = Utc::now();
   let seconds = days * 24.0 * 60.0 * 60.0;
   let yesterday = now.checked_sub_signed(Duration::seconds(seconds.round() as i64)).unwrap();
   let yesterday_unix = yesterday.timestamp();

   let latlonloc = get_latlonloc(opt.lat.unwrap_or_default(), opt.lon.unwrap_or_default(), location);

   if latlonloc.0 == 0.0 && latlonloc.1 == 0.0 {
        bail!("Location '{}' is not recognized, and both latitude and longitude are zero.", latlonloc.2);
   }

   let api_result = blocking::timemachine(&latlonloc.0, &latlonloc.1, &yesterday_unix, "metric", "en", &opt.api_key).unwrap();
    
   print_current(api_result.current, latlonloc.2, latlonloc.3);

   Ok(())
}
