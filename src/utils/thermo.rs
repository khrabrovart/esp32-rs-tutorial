pub fn ntc_to_celsius(
    adc_voltage: f32,
    resistor_ohms: f32,
    ntc_nominal_resistance_ohms: f32,
    reference_temperature_celsius: f32,
    beta_coefficient: f32,
) -> f32 {
    let clamped_voltage_ratio = adc_voltage.clamp(1e-6, 1.0 - 1e-6);
    let thermistor_resistance_ohms = resistor_ohms * (1.0 / clamped_voltage_ratio - 1.0);
    let reference_temp_kelvin = reference_temperature_celsius + 273.15;
    let inverse_temperature_kelvin = 1.0 / reference_temp_kelvin
        + (1.0 / beta_coefficient)
            * (thermistor_resistance_ohms / ntc_nominal_resistance_ohms).ln();

    1.0 / inverse_temperature_kelvin - 273.15
}
