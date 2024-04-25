/// IGNORE THIS WHOLE FILE, IT'S USELESS, BUT JUST KEEPING IT HERE FOR NOW.
use std::borrow::Cow;

use image::ExtendedColorType;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn preprocess_invalid_json(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Regex to match the keys and remove type declarations in values
    let re_key = Regex::new(r"(?m)^\s*([a-zA-Z]+):")?;
    let re_type_value = Regex::new(r"\b([a-zA-Z]+)\(([^)]+)\),")?;

    // // Add double quotes around keys
    // let mut corrected_json = re_key.replace_all(input, |caps: &regex::Captures| format!("\"{}\":", &caps[1]));

    // // Remove type declarations from values
    // corrected_json = re_type_value.replace_all(&corrected_json, "$2,");

    // // Replace any single quotes with double quotes
    // corrected_json = corrected_json.replace("'", "\"");

    // // Modify the last elements to remove the trailing comma
    // corrected_json = corrected_json.replace("},\n", "}\n").replace("],\n", "]\n");

    // Ensure that we're working with Strings here, as `replace` returns a String.
    let mut corrected_json: String = re_key
        .replace_all(input, |caps: &regex::Captures<'_>| format!(r#""{}":"#, &caps[1]))
        .into_owned();

    // Using replace with a String instead of Cow<'_, str>
    corrected_json = corrected_json.replace(r#"'"#, r#"""#);

    // Fix this line similarly by using the result as a String
    corrected_json = corrected_json.replace("},\n", "}\n").replace("],\n", "]\n");

    // Wrap the transformed string in curly braces to form a complete JSON object
    let parent_name = "FrameRequestData ".to_string();
    let mut final_json = parent_name;
    final_json.push_str(corrected_json.as_str());
    // let final_json = format!("Corrected Json: \n{:?}", corrected_json);
    println!("Corrected Json: \n{}", final_json);

    // Verify that the generated string is valid JSON by attempting to parse it
    // let _: Value = serde_json::from_str(&final_json)?;

    // Return the verified and corrected JSON
    Ok(final_json)
}

#[derive(Debug, Deserialize)]
pub struct ExposureTime {
    pub value: i32,
}

#[derive(Debug, Deserialize)]
pub struct AnalogueGain {
    pub value: f32,
}

#[derive(Debug, Deserialize)]
pub struct ColourCorrectionMatrix {
    pub values: Vec<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SensorTimestamp {
    pub value: i32,
}

#[derive(Debug, Deserialize)]
pub struct AwbLocked {
    pub values: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct AeLocked {
    pub value: bool,
}

#[derive(Debug, Deserialize)]
pub struct SensorTemperature {
    pub value: f32,
}

#[derive(Debug, Deserialize)]
pub struct SensorBlackLevels {
    pub values: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct Sharpness {
    pub values: Vec<i32>,
}

#[derive(Debug, Deserialize)]
pub struct DigitalGain {
    pub values: Vec<i32>,
}

#[derive(Debug, Deserialize)]
pub struct FrameDuration {
    pub values: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Deserialize)]
// struct FrameDurationLimits {
#[serde(rename = "FrameDurationLimits")] // This line renames the struct for deserialization
pub struct FrameDurationL {
    pub rectangles: Vec<Rectangle>,
}

#[derive(Debug, Deserialize)]
pub struct AfSpeed {
    pub values: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct FrameRequestData {
    pub exposure_time: ExposureTime,
    pub analogue_gain: AnalogueGain,
    pub colour_correction_matrix: ColourCorrectionMatrix,
    pub sensor_timestamp: SensorTimestamp,
    pub awb_locked: AwbLocked,
    pub ae_locked: AeLocked,
    pub sensor_temperature: SensorTemperature,
    pub sensor_black_levels: SensorBlackLevels,
    pub sharpness: Sharpness,
    pub digital_gain: DigitalGain,
    pub frame_duration: FrameDuration,
    pub frame_duration_limits: FrameDurationL,
    pub af_speed: AfSpeed,
}

pub fn test_my_struct() {
    let json_string = r#"
    {
        "ExposureTime": {
            "value": 66653
        },
        "AnalogueGain": {
            "value": 8.0
        },
        "ColourCorrectionMatrix": {
            "values": [
                4096,
                4096,
                4096,
                4096
            ]
        },
        "SensorTimestamp": {
            "value": 66729
        },
        "AwbLocked": {
            "values": [
                1.063743
            ]
        },
        "AeLocked": {
            "value": false
        },
        "SensorTemperature": {
            "value": 1.0001835
        },
        "SensorBlackLevels": {
            "values": [
                1.217393,
                2.0870757
            ]
        },
        "Sharpness": {
            "values": [
                2988
            ]
        },
        "DigitalGain": {
            "values": [
                328
            ]
        },
        "FrameDuration": {
            "values": [
                2.2660978,
                -0.5489229,
                -0.7171845,
                -0.7698631,
                2.6086154,
                -0.8387422,
                -0.2611642,
                -1.4788433,
                2.739998
            ]
        },
        "FrameDurationLimits": {
            "rectangles": [
                {
                    "x": 0,
                    "y": 2,
                    "width": 3280,
                    "height": 2460
                }
            ]
        },
        "AfSpeed": {
            "values": [
                1554095633441000
            ]
        }
    }
    "#;

    let frame_request_data: FrameRequestData = serde_json::from_str(&json_string).expect("Invalid JSON");
    println!("{:?}", frame_request_data);

    // Get the width from the FrameDurationLimits
    let width = frame_request_data.frame_duration_limits.rectangles[0].width;
    println!("Width: {}", width);
}
