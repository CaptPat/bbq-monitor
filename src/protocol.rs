// src/protocol.rs
use anyhow::{anyhow, Result};
use uuid::Uuid;

// Combustion Inc (MeatStick) Service UUIDs
pub const COMBUSTION_PROBE_STATUS_SERVICE: Uuid = 
    uuid::uuid!("00000100-CAAB-3792-3D44-97AE51C1407A");
pub const COMBUSTION_UART_SERVICE: Uuid = 
    uuid::uuid!("6E400001-B5A3-F393-E0A9-E50E24DCCA9E");
pub const COMBUSTION_UART_RX_CHAR: Uuid = 
    uuid::uuid!("6E400002-B5A3-F393-E0A9-E50E24DCCA9E");
pub const COMBUSTION_UART_TX_CHAR: Uuid = 
    uuid::uuid!("6E400003-B5A3-F393-E0A9-E50E24DCCA9E");

// Legacy MeatStick Service UUIDs (older firmware)
pub const MEATSTICK_SERVICE: Uuid = 
    uuid::uuid!("8D53DC1D-1DB7-4CD3-868B-8A527460AA84");
pub const MEATSTICK_CHAR: Uuid = 
    uuid::uuid!("DA2E7828-FBCE-4E01-AE9E-261174997C48");

// MEATER Service UUIDs (from reverse engineering)
// Note: MEATER uses standard BLE GATT characteristics
pub const MEATER_SERVICE: Uuid = 
    uuid::uuid!("A75CC7FC-C956-488F-AC2A-2DBC08B63A04");

/// MeatStick (Combustion Inc) protocol parser
/// 
/// Based on official Combustion Inc documentation:
/// https://github.com/combustion-inc/combustion-documentation
pub struct MeatStickProtocol;

impl MeatStickProtocol {
    /// Parse MeatStick temperature data
    /// 
    /// Format (13 bytes total - 104 bits):
    /// - 8 temperature sensors (13 bits each)
    /// - Little-endian packed bit fields
    /// - Temperature = (raw_value * 0.05) - 20 (in Celsius)
    /// - Range: -20°C to 369°C per sensor
    /// 
    /// Sensor layout (Combustion Predictive Probe):
    /// - Sensors T1-T4: Core temperatures (internal)
    /// - Sensors T5-T7: Mid-section temperatures
    /// - Sensor T8: Ambient/surface temperature
    pub fn parse_temperature_data(data: &[u8]) -> Result<Vec<f32>> {
        if data.len() < 13 {
            return Err(anyhow!("Insufficient data: need 13 bytes, got {}", data.len()));
        }
        
        let mut temperatures = Vec::with_capacity(8);
        
        // Parse 8 sensors as 13-bit values packed into 13 bytes (104 bits total)
        let mut bit_offset = 0;
        
        for _sensor_idx in 0..8 {
            // Extract 13-bit value
            let byte_offset = bit_offset / 8;
            let bit_shift = bit_offset % 8;
            
            let raw_temp = if bit_shift == 0 {
                // Aligned case: bits fit within 2 bytes
                let low = data[byte_offset] as u16;
                let high = (data[byte_offset + 1] as u16) & 0x1F; // 5 bits
                (high << 8) | low
            } else {
                // Unaligned case: spans 2-3 bytes
                let bits_in_first = 8 - bit_shift;
                let low = (data[byte_offset] as u16) >> bit_shift;
                let mid = (data[byte_offset + 1] as u16) << bits_in_first;
                let high = if bits_in_first < 5 {
                    ((data[byte_offset + 2] as u16) & ((1 << (13 - 8)) - 1)) << (bits_in_first + 8)
                } else {
                    0
                };
                (high | mid | low) & 0x1FFF // Mask to 13 bits
            };
            
            // Convert to Celsius: Temperature = (raw_value * 0.05) - 20
            let temp_celsius = (raw_temp as f32 * 0.05) - 20.0;
            
            // Convert to Fahrenheit
            let temp_fahrenheit = temp_celsius * 9.0 / 5.0 + 32.0;
            
            // Sanity check: reasonable temperature range
            if (-40.0..=1100.0).contains(&temp_fahrenheit) {
                temperatures.push(temp_fahrenheit);
            } else {
                // Invalid reading - use 0 or skip
                temperatures.push(0.0);
            }
            
            bit_offset += 13;
        }
        
        if temperatures.is_empty() {
            return Err(anyhow!("No valid temperatures parsed"));
        }
        
        Ok(temperatures)
    }
    
    /// Get the internal (meat core) temperature
    /// For Combustion probes, T1-T4 are core sensors
    /// Returns the deepest valid core reading (typically T4)
    pub fn get_internal_temp(temperatures: &[f32]) -> Option<f32> {
        if temperatures.is_empty() {
            return None;
        }
        
        // Try T4 (index 3) as the deepest core sensor
        if temperatures.len() >= 4 && temperatures[3] > 0.0 {
            return Some(temperatures[3]);
        }
        
        // Fallback to other core sensors (T3, T2, T1)
        for i in (0..temperatures.len().min(4)).rev() {
            if temperatures[i] > 0.0 {
                return Some(temperatures[i]);
            }
        }
        
        None
    }
    
    /// Get the ambient temperature
    /// For Combustion probes, T8 (index 7) is the ambient sensor
    pub fn get_ambient_temp(temperatures: &[f32]) -> Option<f32> {
        if temperatures.len() >= 8 && temperatures[7] > 0.0 {
            Some(temperatures[7])
        } else if temperatures.len() >= 6 {
            // Fallback to T6 or T7 if T8 not available
            temperatures[temperatures.len() - 1..]
                .iter()
                .rev()
                .find(|&&t| t > 0.0)
                .copied()
        } else {
            None
        }
    }
}

/// MEATER protocol parser
/// 
/// Based on reverse engineering by Nathan Faber:
/// https://github.com/nathanfaber/meaterble
pub struct MeaterProtocol;

impl MeaterProtocol {
    /// Parse MEATER temperature data
    /// 
    /// Format (8 bytes total):
    /// - Bytes 0-1: Tip temperature (little-endian u16)
    /// - Bytes 2-3: RA ambient reading (little-endian u16)
    /// - Bytes 4-5: OA ambient reading (little-endian u16)
    /// - Bytes 6-7: Unknown/reserved
    /// 
    /// Temperature conversion:
    /// - Tip: direct value / 10.0 = Celsius
    /// - Ambient: calculated from RA and OA using formula
    pub fn parse_temperature_data(data: &[u8]) -> Result<Vec<f32>> {
        if data.len() < 8 {
            return Err(anyhow!("Insufficient data for MEATER format: need 8 bytes, got {}", data.len()));
        }
        
        let mut temperatures = Vec::new();
        
        // Parse tip temperature (bytes 0-1)
        let tip_raw = u16::from_le_bytes([data[0], data[1]]);
        let tip_celsius = tip_raw as f32 / 10.0;
        let tip_fahrenheit = tip_celsius * 9.0 / 5.0 + 32.0;
        
        if (-40.0..=600.0).contains(&tip_fahrenheit) {
            temperatures.push(tip_fahrenheit);
        }
        
        // Parse ambient temperature components
        let ra_raw = u16::from_le_bytes([data[2], data[3]]);
        let oa_raw = u16::from_le_bytes([data[4], data[5]]);
        
        // Calculate ambient using MEATER formula (from Nathan Faber's work)
        // ambient = tip + max(0, ((ra - min(48, oa)) * 16 * 589) / 1487)
        let ambient_raw = tip_raw as i32 + 
            ((((ra_raw as i32 - oa_raw.min(48) as i32) * 16 * 589) / 1487).max(0));
        
        let ambient_celsius = ambient_raw as f32 / 10.0;
        let ambient_fahrenheit = ambient_celsius * 9.0 / 5.0 + 32.0;
        
        if (-40.0..=600.0).contains(&ambient_fahrenheit) {
            temperatures.push(ambient_fahrenheit);
        }
        
        Ok(temperatures)
    }
    
    /// Get internal/tip temperature (first sensor)
    pub fn get_internal_temp(temperatures: &[f32]) -> Option<f32> {
        temperatures.first().copied()
    }
    
    /// Get ambient temperature (second sensor)
    pub fn get_ambient_temp(temperatures: &[f32]) -> Option<f32> {
        if temperatures.len() >= 2 {
            Some(temperatures[1])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_meatstick_parsing() {
        // Simulate room temperature readings (72°F = 22.2°C)
        // Using Combustion format: (temp_c + 20) / 0.05 = raw
        // 22.2°C: (22.2 + 20) / 0.05 = 844
        let raw_value = 844u16;
        
        // Create 13-byte packed data for 8 sensors (13 bits each)
        // Simplified: just putting same value in first few sensors
        let mut data = vec![0u8; 13];
        data[0] = (raw_value & 0xFF) as u8;
        data[1] = ((raw_value >> 8) & 0x1F) as u8;
        
        let temps = MeatStickProtocol::parse_temperature_data(&data).unwrap();
        assert!(!temps.is_empty());
        
        // Should be close to 72°F
        let temp_f = temps[0];
        assert!((temp_f - 72.0).abs() < 1.0, "Expected ~72°F, got {}", temp_f);
    }
    
    #[test]
    fn test_meater_parsing() {
        // Simulate MEATER data: tip at 72°F (22.2°C = 222 raw)
        // ambient at 80°F (26.7°C)
        let data = vec![
            0xDE, 0x00, // Tip: 222 (22.2°C = 72°F)
            0x00, 0x01, // RA: 256
            0x00, 0x01, // OA: 256
            0x00, 0x00, // Reserved
        ];
        
        let temps = MeaterProtocol::parse_temperature_data(&data).unwrap();
        assert_eq!(temps.len(), 2);
        
        // Check tip temperature
        assert!((temps[0] - 72.0).abs() < 1.0);
    }
}
