use anyhow::Result;
use bbq_monitor::{generate_license_key, PremiumTier};
use chrono::{Duration, Utc};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "generate" => {
            let tier = if args.len() > 2 && args[2].to_lowercase() == "premium" {
                PremiumTier::Premium
            } else {
                PremiumTier::Free
            };

            let expires_at = if args.len() > 3 {
                match args[3].parse::<i64>() {
                    Ok(days) if days > 0 => Some(Utc::now() + Duration::days(days)),
                    Ok(_) | Err(_) => None, // Negative or invalid = lifetime
                }
            } else {
                None // Default: lifetime license
            };

            let key = generate_license_key(tier, expires_at)?;
            
            println!("╔══════════════════════════════════════════════════════╗");
            println!("║           BBQ Monitor License Generator             ║");
            println!("╚══════════════════════════════════════════════════════╝");
            println!();
            println!("Tier: {:?}", tier);
            
            if let Some(expiry) = expires_at {
                println!("Expires: {}", expiry.format("%Y-%m-%d"));
            } else {
                println!("Expires: Never (Lifetime)");
            }
            
            println!();
            println!("License Key:");
            println!("┌────────────────────────────────────────────────────┐");
            println!("│ {}  │", key);
            println!("└────────────────────────────────────────────────────┘");
            println!();
            println!("Add this to config.toml:");
            println!();
            println!("[premium]");
            println!("license_key = \"{}\"", key);
            println!();
        }
        "validate" => {
            if args.len() < 3 {
                eprintln!("Error: Missing license key");
                print_usage();
                return Ok(());
            }

            let key = &args[2];
            let validator = bbq_monitor::LicenseValidator::new();
            
            match validator.validate(key) {
                Ok(license) => {
                    println!("╔══════════════════════════════════════════════════════╗");
                    println!("║           BBQ Monitor License Validator             ║");
                    println!("╚══════════════════════════════════════════════════════╝");
                    println!();
                    println!("✅ License is VALID");
                    println!();
                    println!("Tier: {:?}", license.tier);
                    println!("Valid: {}", license.is_valid());
                    
                    if let Some(expiry) = license.expires_at {
                        println!("Expires: {}", expiry.format("%Y-%m-%d %H:%M:%S UTC"));
                        if let Some(days) = license.days_until_expiry() {
                            println!("Days remaining: {}", days);
                        }
                    } else {
                        println!("Expires: Never (Lifetime)");
                    }
                    
                    println!();
                    println!("Enabled Features:");
                    println!("  • Cloud Sync: {}", if license.features.cloud_sync { "✓" } else { "✗" });
                    println!("  • Unlimited History: {}", if license.features.unlimited_history { "✓" } else { "✗" });
                    println!("  • Cook Profiles: {}", if license.features.cook_profiles { "✓" } else { "✗" });
                    println!("  • Remote Access: {}", if license.features.remote_access { "✓" } else { "✗" });
                    println!("  • Advanced Analytics: {}", if license.features.advanced_analytics { "✓" } else { "✗" });
                    println!("  • Alerts: {}", if license.features.alerts { "✓" } else { "✗" });
                    println!();
                }
                Err(e) => {
                    println!("❌ License validation failed: {}", e);
                }
            }
        }
        "examples" => {
            print_examples();
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("BBQ Monitor License Tool");
    println!();
    println!("USAGE:");
    println!("    license-tool <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    generate <tier> [days]    Generate a new license key");
    println!("                              tier: 'free' or 'premium'");
    println!("                              days: expiry in days (omit for lifetime)");
    println!();
    println!("    validate <key>            Validate an existing license key");
    println!();
    println!("    examples                  Show usage examples");
    println!();
    println!("EXAMPLES:");
    println!("    license-tool generate premium          # Lifetime Premium");
    println!("    license-tool generate premium 365      # Premium for 1 year");
    println!("    license-tool generate premium 30       # Premium for 30 days");
    println!("    license-tool validate \"KEY-HERE\"       # Validate a key");
}

fn print_examples() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           BBQ Monitor License Examples                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    
    println!("1. Generate a LIFETIME Premium license:");
    println!("   $ cargo run --bin license-tool generate premium");
    println!();
    
    println!("2. Generate a 1-YEAR Premium license:");
    println!("   $ cargo run --bin license-tool generate premium 365");
    println!();
    
    println!("3. Generate a 30-DAY trial Premium license:");
    println!("   $ cargo run --bin license-tool generate premium 30");
    println!();
    
    println!("4. Validate a license key:");
    println!("   $ cargo run --bin license-tool validate \"UExFTUlVTXxORVZFUnwyMDI2LTAxLTIwVDE5OjUzOjE5LjQzMzM5NjcwMCswMDowMA==\"");
    println!();
    
    println!("PRICING SUGGESTIONS:");
    println!("  • Free Tier: Local monitoring, 7-day history");
    println!("  • Premium Lifetime: $49 (one-time)");
    println!("  • Premium Annual: $49/year");
    println!("  • Premium Monthly: $4.99/month");
    println!("  • 30-day Trial: Free (for testing)");
    println!();
    
    println!("SALES WORKFLOW:");
    println!("  1. Customer purchases via Gumroad/Stripe");
    println!("  2. Payment webhook triggers license generation");
    println!("  3. Email license key to customer");
    println!("  4. Customer adds key to config.toml");
    println!("  5. Application validates on startup");
    println!();
}
