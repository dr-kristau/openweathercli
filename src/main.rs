use openweathermap::*;
use structopt::StructOpt;
use anyhow::{Result, bail};
//use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use chrono::{Utc, Duration, TimeZone, FixedOffset};
use serde_derive::{Deserialize};

#[derive(Debug,Deserialize)]
struct Zone {
    zone_id:i64,
    country_code:String,
    zone_name:String
}

#[derive(Debug,Deserialize)]
struct TimeZoneCSV {
    zone_id:i64,
    abbreviation:String,
    time_start:i64,
    gmt_offset:i32,
    dst:i32
}

#[derive(Debug,Deserialize)]
struct WordCities {
    city: String,
    city_ascii: String,
    lat: f64,
    lng: f64,
    country:String,
    iso2:String,
    iso3:String,
    admin_name:String,
    capital:String,
    population:Option<f64>,
    id:i64
}


#[derive(StructOpt)]
struct Opt {

    #[structopt(long)]
    lat: Option<f64>,
    
    #[structopt(long)]
    lon: Option<f64>,

    #[structopt(long)]
    loc: Option<String>,

    #[structopt(long)]
    utc: Option<i32>,

    #[structopt(long)]
    days: f64,
    
    #[structopt(long, default_value = "MY_API_KEY")]
    api_key: String
}

fn get_latlonloc(lat:f64, lon:f64, loc:&String, time:i32, unix:i64) -> Result<(f64, f64, String, Option<FixedOffset>)> {
    let mut m_lat = lat;
    let mut m_lon = lon;
    let mut timeoffset = if time < 0 {
        FixedOffset::west((time * -1) * 3600)
    }
    else {
        FixedOffset::east(time * 3600)
    };

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
    else if m_lat == 0.0 && m_lon == 0.0 {
        match find_latlong(loc) {
            Ok(l) => {
                match l {
                    Some(latlon) => {
                        m_lat = latlon.0;
                        m_lon = latlon.1;
                    }
                    None => {}
                } 
            }
            Err(e) => bail!("Error {} loading file", e)
        }
        if time == 0 {
            match find_timezone(loc, unix) {
                Ok(tz) => {
                    match tz {
                        Some(timezone) => {
                            if timezone < 0 {
                                timeoffset = FixedOffset::west((timezone * -1) * 3600);
                            }
                            else {
                                timeoffset = FixedOffset::east(timezone * 3600);
                            }
                            
                        }
                        None => {
                            return Ok((m_lat, m_lon, format!("{} [{},{}]", loc, m_lat, m_lon), None));
                        }
                    } 
                }
                Err(e) => bail!("Error {} loading file", e)
            }
        }
    }

    return Ok((m_lat, m_lon, format!("{} [{},{}]", loc, m_lat, m_lon), Some(timeoffset)));
}

fn print_current(current:Current, location:String, timezone:Option<FixedOffset>) {
    match timezone {
        Some(timezone) => {
            println!("Weather for {} on {}", location, Utc.timestamp(current.dt, 0).with_timezone(&timezone));
        }
        None => {
            println!("Weather for {} on {}", location, Utc.timestamp(current.dt, 0));
        }
    }
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
    match timezone {
        Some(tz) => {
            match current.sunrise {
                Some(sunrise) => {
                    println!("Sunrise: {}", Utc.timestamp(sunrise, 0).with_timezone(&tz));
                }
                None => {}
            }
            
            match current.sunset {
                Some(sunset) => {
                    println!("Sunrise: {}", Utc.timestamp(sunset, 0).with_timezone(&tz));
                }
                None => {}
            }
        }
        None => {
            match current.sunrise {
                Some(sunrise) => {
                    println!("Sunrise: {}", Utc.timestamp(sunrise, 0));
                }
                None => {}
            }
            
            match current.sunset {
                Some(sunset) => {
                    println!("Sunrise: {}", Utc.timestamp(sunset, 0));
                }
                None => {}
            }
        }
    }
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

fn load_zone() -> Result<Vec<Zone>> {
    let bytes = std::include_bytes!("data/zone.csv");
    let mut vec = Vec::new();
    
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(bytes.as_ref()); 

    for result in rdr.deserialize() {
        let record: Zone = result?; 
        vec.push(record);
    }

    Ok(vec)
}

fn load_timezone() -> Result<Vec<TimeZoneCSV>> {
    let bytes = std::include_bytes!("data/timezone.csv");
    let mut vec = Vec::new();
    
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(bytes.as_ref()); 

    for result in rdr.deserialize() {
        let record: TimeZoneCSV = result?; 
        vec.push(record);
    }

    Ok(vec)
}

fn load_cities() -> Result<Vec<WordCities>> {
    let bytes = std::include_bytes!("data/worldcities.csv");
    let mut vec = Vec::new();
    
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes.as_ref()); 

    for result in rdr.deserialize() {
        let record: WordCities = result?; 
        vec.push(record);
    }

    Ok(vec)
}

fn find_timezone(city:&String, unix_time:i64) -> Result<Option<i32>> {
    let no_space_city = city.replace(" ", "_");
    match load_zone() {
        Ok(v) => {
            let ii = v.into_iter().find(|y| y.zone_name.ends_with(&no_space_city));
            match ii {
                Some(ci) => {
                    match load_timezone() {
                        Ok(v) => {
                            let mut ww: Vec<TimeZoneCSV> = v.into_iter()
                                .filter(|y| y.zone_id == ci.zone_id)
                                .collect();

                            ww.sort_by(|a, b| b.time_start.cmp(&a.time_start));
                            let uu = ww.iter().find(|z| z.time_start <= unix_time);
                            match uu {
                                Some(ci) => {
                                    return Ok(Some(ci.gmt_offset / 3600));
                                }
                                None => {  
                                    return Ok(None);
                                }
                            }
                        }
                        Err(e) => bail!("Error {} loading file", e), 
                    }
                }
                None => {  
                    return Ok(None);
                }
            }
        }
        Err(e) => bail!("Error {} loading file", e), 
    }
} 

fn find_latlong(city:&String) -> Result<Option<(f64, f64)>> {
    match load_cities() {
        Ok(v) => {
            let uu = v.into_iter().find(|y| &y.city == city);
            match uu {
                Some(ci) => {
                    return Ok(Some((ci.lat, ci.lng)));
                }
                None => { return Ok(None); }
            }
        }
        Err(e) => bail!("Error {} loading file", e), 
    }
}


fn main() -> Result<()> {
   let opt = Opt::from_args();
   let location = &opt.loc.unwrap_or_default();
   let days = opt.days;

   if days < 0.0 || days > 5.0 {
        bail!("Day offset '{}' not between one and five", days);
   }

   let now = Utc::now();
   let seconds = days * 24.0 * 60.0 * 60.0;
   let yesterday = now.checked_sub_signed(Duration::seconds(seconds.round() as i64)).unwrap();
   let yesterday_unix = yesterday.timestamp();

   match get_latlonloc(opt.lat.unwrap_or_default(), opt.lon.unwrap_or_default(), location, opt.utc.unwrap_or_default(), yesterday_unix) {
       Ok(latlonloc) => {
            if latlonloc.0 == 0.0 && latlonloc.1 == 0.0 {
                bail!("Location '{}' is not recognized, and both latitude and longitude are zero.", latlonloc.2);
            }
    
            let api_result = blocking::timemachine(&latlonloc.0, &latlonloc.1, &yesterday_unix, "metric", "en", &opt.api_key).unwrap();
        
            print_current(api_result.current, latlonloc.2, latlonloc.3);
       }
       Err(e) => {
            bail!("Error {}", e);
       }
   }

   Ok(())
}
