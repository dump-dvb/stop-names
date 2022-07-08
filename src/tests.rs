
#[test]
fn test_serialization() {
    let data = TransmissionPosition {
        dhid: Some("dhid".to_string()),
        name: Some("name".to_string()),
        telegram_type: TelegramType::DoorClosed,
        direction: 0,
        lat: 0.0,
        lon: 0.0
    };

    let reference = String::from("{
  \"dhid\": \"dhid\",
  \"name\": \"name\",
  \"telegram_type\": \"3\",
  \"direction\": 0,
  \"lat\": 0.0,
  \"lon\": 0.0
}");
    let json_data = serde_json::to_string_pretty(&data)
        .expect("cannot serialize structs!");
   
    assert_eq!(json_data, reference);
}

