use std::env;
use std::fs;
use std::io::{self, BufRead};

#[derive(Debug, Default)]
struct RangeMap {
    ranges: Vec<(usize, usize, usize)>
}

impl RangeMap {
    
    fn get(&self, value: usize) -> Option<usize> {
        for (destination, source, size) in self.ranges.iter() {
            if *source <= value && value < source + size {
                return Some(destination + value - source);
            }
        }

        None
    }

    /// This function returns a vector of new ranges from the original range passed in as a parameter.
    fn get_ranges(&self, start: usize, size: usize) -> Vec<(usize, usize)> {
        self.ranges
            .iter()
            .filter_map(|(destination, source, s_size)| {
                let intersection_start = start.max(*source);
                let intersection_end = (start + size).min(source + s_size);
                if intersection_start < intersection_end {
                    Some((
                        destination + intersection_start - *source,
                        intersection_end - intersection_start,
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

}

struct RangeMapChain {
    range_maps: Vec<(String, RangeMap)>
}

impl RangeMapChain {
    fn resolve(&self, value: usize, label: &str) -> Option<usize> {
        let mut mapped = value;
        for (range_map_label, range_map) in self.range_maps.iter() {
            if let Some(output) = range_map.get(mapped) {
                mapped = output;
                if label == range_map_label {
                    return Some(mapped)
                }
            } else {
                return None
            }
        }
        None
    }

    fn resolve_ranges(&self, ranges: &[(usize, usize)], label: &str) -> Vec<(usize, usize)> {
        let mut mapped: Vec<(usize, usize)> = ranges.into();
        for (range_map_label, range_map) in self.range_maps.iter() {
            mapped = mapped.into_iter().flat_map(|(start, size)| range_map.get_ranges(start, size)).collect();
            if label == range_map_label {
                return mapped
            }
        }
        mapped
    }
}

#[derive(Debug)]
enum CapturingStatus {
    NoStatus,
    SeedToSoil,
    SoilToFertilizer,
    FertilizerToWater,
    WaterToLight,
    LightToTemperature,
    TemperatureToHumidity,
    HumidityToLocation
}


fn main() {
    let path = env::args().nth(1).expect("Missing required parameter path!");

    let mut data = io::BufReader::new(
        fs::File::open(path).expect("Could not open file!"))
        .lines();

    let seeds: Vec<usize> = data
        .next()
        .expect("Unexpected EOF!")
        .expect("Could not read line!")
        .trim_start_matches("seeds: ")
        .trim()
        .split_whitespace()
        .filter_map(|n| n.parse::<usize>().ok())
        .collect();

    let mut capturing: CapturingStatus = CapturingStatus::NoStatus;
    let mut seed_to_soil = RangeMap::default();
    let mut soil_to_fertilizer = RangeMap::default();
    let mut fertilizer_to_water = RangeMap::default();
    let mut water_to_light = RangeMap::default();
    let mut light_to_temperature = RangeMap::default();
    let mut temperature_to_humidity = RangeMap::default();
    let mut humidity_to_location = RangeMap::default();

    for line in data {
        let text = line.expect("Could not read line!");
        match text.as_str() {
            "" => continue,
            "seed-to-soil map:" => capturing = CapturingStatus::SeedToSoil,
            "soil-to-fertilizer map:" => capturing = CapturingStatus::SoilToFertilizer,
            "fertilizer-to-water map:" => capturing = CapturingStatus::FertilizerToWater,
            "water-to-light map:" => capturing = CapturingStatus::WaterToLight,
            "light-to-temperature map:" => capturing = CapturingStatus::LightToTemperature,
            "temperature-to-humidity map:" => capturing = CapturingStatus::TemperatureToHumidity,
            "humidity-to-location map:" => capturing = CapturingStatus::HumidityToLocation,
            _ => {
                let split: Vec<usize> = text.trim().split_whitespace().filter_map(|n| n.parse::<usize>().ok()).collect();
                let range = (split[0], split[1], split[2]);
                match capturing {
                    CapturingStatus::SeedToSoil => seed_to_soil.ranges.push(range),
                    CapturingStatus::SoilToFertilizer => soil_to_fertilizer.ranges.push(range),
                    CapturingStatus::FertilizerToWater => fertilizer_to_water.ranges.push(range),
                    CapturingStatus::WaterToLight => water_to_light.ranges.push(range),
                    CapturingStatus::LightToTemperature => light_to_temperature.ranges.push(range),
                    CapturingStatus::TemperatureToHumidity => temperature_to_humidity.ranges.push(range),
                    CapturingStatus::HumidityToLocation => humidity_to_location.ranges.push(range),
                    _ => ()
                }
            }
        }
    }

    let chain = RangeMapChain{
        range_maps: vec![
            (
                String::from("soil"),
                seed_to_soil
            ),
            (
                String::from("fertilizer"),
                soil_to_fertilizer
            ),
            (
                String::from("water"),
                fertilizer_to_water
            ),
            (
                String::from("light"),
                water_to_light
            ),
            (
                String::from("temperature"),
                light_to_temperature
            ),
            (
                String::from("humidity"),
                temperature_to_humidity
            ),
            (
                String::from("location"),
                humidity_to_location
            )
        ]
    };

println!(
    "Minimum location for seeds: {}",
    chain.resolve_ranges(
        &seeds
            .chunks(2)
            .into_iter()
            .map(|s| (s[0], s[1]))
            .collect::<Vec<(usize, usize)>>(), 
        "location")
        .into_iter()
        .map(|x| x.0)
        .min()
        .expect("Could not map any seeds!")
)

// println!(
//     "Minimum location for seeds: {}", 
//     seeds
//         .chunks(2)
//         .into_iter()
//         .flat_map(|range| {
//             let start = range[0];
//             let size = range[1];
//             (start..(start+size))
//                 .filter_map(|s| chain.resolve(s, "location"))
//         })
//         .min()
//         .expect("Could not map any seeds!")
// )
}


#[test]
fn test_resolve_range() {
    let range_map = RangeMap {
        ranges: vec![(100, 0, 50), (200, 50, 50), (500, 100, 100)]
    };

    assert_eq!(vec![(125, 25), (200, 25)], range_map.get_ranges(25, 50));
    assert_eq!(vec![(110, 10)], range_map.get_ranges(10, 10));
}